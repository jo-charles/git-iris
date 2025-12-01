//! File watcher service for Iris Companion
//!
//! Monitors the repository for file changes using the `notify` crate
//! with debouncing and gitignore filtering.

use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Events emitted by the companion file watcher
#[derive(Debug, Clone)]
pub enum CompanionEvent {
    /// A file was created
    FileCreated(PathBuf),
    /// A file was modified
    FileModified(PathBuf),
    /// A file was deleted
    FileDeleted(PathBuf),
    /// A file was renamed (old path, new path)
    FileRenamed(PathBuf, PathBuf),
    /// Git ref changed (branch switch, commit, etc.)
    GitRefChanged,
    /// Watcher error occurred
    WatcherError(String),
}

/// File watcher service that monitors repository changes
pub struct FileWatcherService {
    /// The debounced watcher
    _watcher: Debouncer<RecommendedWatcher, RecommendedCache>,
    /// Repository root path
    repo_path: PathBuf,
}

impl FileWatcherService {
    /// Create a new file watcher for the given repository
    pub fn new(repo_path: &Path, event_tx: mpsc::UnboundedSender<CompanionEvent>) -> Result<Self> {
        let repo_path = repo_path.to_path_buf();
        let repo_path_clone = repo_path.clone();

        // Build gitignore matcher
        let gitignore = Self::build_gitignore(&repo_path);

        // Create debouncer with 500ms delay
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: DebounceEventResult| {
                Self::handle_events(result, &repo_path_clone, &gitignore, &event_tx);
            },
        )
        .context("Failed to create file watcher debouncer")?;

        // Watch the repository recursively
        debouncer
            .watch(&repo_path, RecursiveMode::Recursive)
            .context("Failed to start watching repository")?;

        Ok(Self {
            _watcher: debouncer,
            repo_path,
        })
    }

    /// Build a gitignore matcher from repo's .gitignore files
    fn build_gitignore(repo_path: &Path) -> Arc<Gitignore> {
        let mut builder = GitignoreBuilder::new(repo_path);

        // Add root .gitignore
        let gitignore_path = repo_path.join(".gitignore");
        if gitignore_path.exists() {
            let _ = builder.add(&gitignore_path);
        }

        // Add global gitignore if available
        if let Some(home) = dirs::home_dir() {
            let global_ignore = home.join(".gitignore_global");
            if global_ignore.exists() {
                let _ = builder.add(&global_ignore);
            }
        }

        // Always ignore .git directory
        let _ = builder.add_line(None, ".git/");

        Arc::new(builder.build().unwrap_or_else(|_| {
            // Fallback: just ignore .git
            let mut fallback = GitignoreBuilder::new(repo_path);
            let _ = fallback.add_line(None, ".git/");
            fallback
                .build()
                .expect("Failed to build fallback gitignore")
        }))
    }

    /// Handle debounced file events
    fn handle_events(
        result: DebounceEventResult,
        repo_path: &Path,
        gitignore: &Gitignore,
        event_tx: &mpsc::UnboundedSender<CompanionEvent>,
    ) {
        match result {
            Ok(events) => {
                for event in events {
                    // Check for git ref changes (HEAD, refs, index)
                    let is_git_ref_change = event.paths.iter().any(|p| {
                        p.strip_prefix(repo_path)
                            .map(|rel| {
                                let rel_str = rel.to_string_lossy();
                                rel_str == ".git/HEAD"
                                    || rel_str.starts_with(".git/refs/")
                                    || rel_str == ".git/index"
                            })
                            .unwrap_or(false)
                    });

                    if is_git_ref_change {
                        let _ = event_tx.send(CompanionEvent::GitRefChanged);
                        continue;
                    }

                    // Convert notify event kind to our event type
                    use notify::EventKind;
                    for path in &event.paths {
                        // Skip gitignored files (including .git/)
                        if Self::is_ignored(path, repo_path, gitignore) {
                            continue;
                        }

                        let companion_event = match event.kind {
                            EventKind::Create(_) => Some(CompanionEvent::FileCreated(path.clone())),
                            EventKind::Modify(_) => {
                                Some(CompanionEvent::FileModified(path.clone()))
                            }
                            EventKind::Remove(_) => Some(CompanionEvent::FileDeleted(path.clone())),
                            _ => None,
                        };

                        if let Some(e) = companion_event {
                            let _ = event_tx.send(e);
                        }
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    let _ = event_tx.send(CompanionEvent::WatcherError(error.to_string()));
                }
            }
        }
    }

    /// Check if a path should be ignored (gitignored or .git internal)
    fn is_ignored(path: &Path, repo_path: &Path, gitignore: &Gitignore) -> bool {
        // Get relative path
        let Ok(rel_path) = path.strip_prefix(repo_path) else {
            return false;
        };

        // Check if it's a directory (for gitignore matching)
        let is_dir = path.is_dir();

        // Check gitignore
        gitignore.matched(rel_path, is_dir).is_ignore()
    }

    /// Get the repository path being watched
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }
}
