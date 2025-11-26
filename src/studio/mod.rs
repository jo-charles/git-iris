//! Iris Studio - Unified TUI for git-iris
//!
//! A single, context-aware interface for all git-iris capabilities:
//! - **Explore**: Semantic code understanding with AI-powered blame
//! - **Commit**: Generate and refine commit messages
//! - **Review**: AI-powered code review (future)
//! - **PR**: Pull request creation (future)
//! - **Changelog**: Release documentation (future)

mod app;
mod events;
mod layout;
mod state;
mod theme;

// Submodules
pub mod components;
pub mod modals;
pub mod modes;

// Re-exports
pub use app::{ExitResult, StudioApp, run_studio};
pub use state::{Mode, StudioState};
