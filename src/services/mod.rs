//! Services module for Git-Iris
//!
//! This module provides focused service layers for specific operations:
//! - `GitCommitService` - Git commit operations (create commits, hooks)

pub mod git_commit;

pub use git_commit::GitCommitService;
