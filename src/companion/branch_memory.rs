//! Branch memory for Iris Companion
//!
//! Remembers context per branch across sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Focus state - where the user was last working
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFocus {
    /// Path to the focused file
    pub path: PathBuf,
    /// Line number in the file
    pub line: usize,
    /// When this focus was recorded
    pub timestamp: DateTime<Utc>,
}

impl FileFocus {
    /// Create a new file focus
    pub fn new(path: PathBuf, line: usize) -> Self {
        Self {
            path,
            line,
            timestamp: Utc::now(),
        }
    }
}

/// Per-branch persistent memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMemory {
    /// Branch name
    pub branch_name: String,
    /// When this branch was first visited
    pub created_at: DateTime<Utc>,
    /// When this branch was last visited
    pub last_visited: DateTime<Utc>,
    /// Last focused file and line
    pub last_focus: Option<FileFocus>,
    /// User notes for this branch
    pub notes: Vec<String>,
    /// Number of sessions on this branch
    pub session_count: u32,
    /// Number of commits made on this branch (across sessions)
    pub total_commits: u32,
}

impl BranchMemory {
    /// Create new branch memory
    pub fn new(branch_name: String) -> Self {
        let now = Utc::now();
        Self {
            branch_name,
            created_at: now,
            last_visited: now,
            last_focus: None,
            notes: Vec::new(),
            session_count: 1,
            total_commits: 0,
        }
    }

    /// Record a new session visit
    pub fn record_visit(&mut self) {
        self.last_visited = Utc::now();
        self.session_count += 1;
    }

    /// Update last focus
    pub fn set_focus(&mut self, path: PathBuf, line: usize) {
        self.last_focus = Some(FileFocus::new(path, line));
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        self.last_focus = None;
    }

    /// Add a note
    pub fn add_note(&mut self, note: String) {
        self.notes.push(note);
    }

    /// Record a commit
    pub fn record_commit(&mut self) {
        self.total_commits += 1;
    }

    /// Time since last visit
    pub fn time_since_last_visit(&self) -> chrono::Duration {
        Utc::now() - self.last_visited
    }

    /// Check if this is a returning visit (visited before more than 5 minutes ago)
    pub fn is_returning_visit(&self) -> bool {
        self.session_count > 1
            && self.time_since_last_visit() > chrono::Duration::minutes(5)
    }

    /// Generate a welcome message if returning
    pub fn welcome_message(&self) -> Option<String> {
        if !self.is_returning_visit() {
            return None;
        }

        let duration = self.time_since_last_visit();
        let time_str = if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} minutes ago", duration.num_minutes())
        };

        Some(format!("Welcome back to {}! Last here {}", self.branch_name, time_str))
    }
}
