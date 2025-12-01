//! Iris Companion - Ambient awareness for Git workflows
//!
//! Provides session tracking, branch memory, and live file watching
//! to transform Studio into an always-aware development companion.

mod branch_memory;
mod session;
mod storage;
mod watcher;

pub use branch_memory::{BranchMemory, FileFocus};
pub use session::{FileActivity, SessionState};
pub use storage::CompanionStorage;
pub use watcher::{CompanionEvent, FileWatcherService};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Main companion service that coordinates all subsystems
pub struct CompanionService {
    /// Repository path being watched
    repo_path: PathBuf,
    /// Current session state
    session: Arc<parking_lot::RwLock<SessionState>>,
    /// Storage backend for persistence
    storage: CompanionStorage,
    /// File watcher service (optional - may fail to start)
    watcher: Option<FileWatcherService>,
    /// Channel for receiving companion events
    event_rx: mpsc::UnboundedReceiver<CompanionEvent>,
    /// Channel sender (held to keep channel alive)
    _event_tx: mpsc::UnboundedSender<CompanionEvent>,
}

impl CompanionService {
    /// Create a new companion service for the given repository
    pub fn new(repo_path: PathBuf, branch: &str) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Initialize storage
        let storage = CompanionStorage::new(&repo_path)?;

        // Try to load existing session or create new one
        let session = storage
            .load_session()?
            .filter(|s| s.branch == branch) // Only restore if same branch
            .unwrap_or_else(|| SessionState::new(repo_path.clone(), branch.to_owned()));

        let session = Arc::new(parking_lot::RwLock::new(session));

        // Try to start file watcher (non-fatal if it fails)
        let watcher = match FileWatcherService::new(&repo_path, event_tx.clone()) {
            Ok(w) => {
                tracing::info!("Companion file watcher started");
                Some(w)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to start file watcher: {}. Companion will run without live updates.",
                    e
                );
                None
            }
        };

        Ok(Self {
            repo_path,
            session,
            storage,
            watcher,
            event_rx,
            _event_tx: event_tx,
        })
    }

    /// Get the current session state
    pub fn session(&self) -> &Arc<parking_lot::RwLock<SessionState>> {
        &self.session
    }

    /// Load branch memory for the given branch
    pub fn load_branch_memory(&self, branch: &str) -> Result<Option<BranchMemory>> {
        self.storage.load_branch_memory(branch)
    }

    /// Save branch memory
    pub fn save_branch_memory(&self, memory: &BranchMemory) -> Result<()> {
        self.storage.save_branch_memory(memory)
    }

    /// Save current session state
    pub fn save_session(&self) -> Result<()> {
        let session = self.session.read();
        self.storage.save_session(&session)
    }

    /// Record a file touch (opened/modified)
    pub fn touch_file(&self, path: PathBuf) {
        let mut session = self.session.write();
        session.touch_file(path);
    }

    /// Record a commit was made
    pub fn record_commit(&self, hash: String) {
        let mut session = self.session.write();
        session.record_commit(hash);
    }

    /// Try to receive the next companion event (non-blocking)
    pub fn try_recv_event(&mut self) -> Option<CompanionEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Check if file watcher is active
    pub fn has_watcher(&self) -> bool {
        self.watcher.is_some()
    }

    /// Get repository path
    pub fn repo_path(&self) -> &PathBuf {
        &self.repo_path
    }
}

impl Drop for CompanionService {
    fn drop(&mut self) {
        // Try to save session on shutdown
        if let Err(e) = self.save_session() {
            tracing::warn!("Failed to save session on shutdown: {}", e);
        }
    }
}
