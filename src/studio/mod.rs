//! Iris Studio - Unified TUI for git-iris
//!
//! A single, context-aware interface for all git-iris capabilities:
//! - **Explore**: Semantic code understanding with AI-powered blame
//! - **Commit**: Generate and refine commit messages
//! - **Review**: AI-powered code review (future)
//! - **PR**: Pull request creation (future)
//! - **Changelog**: Release documentation (future)

// TUI code commonly uses these patterns - allow at module level
#![allow(clippy::cast_possible_truncation)] // u16 terminal dimensions
#![allow(clippy::cast_sign_loss)] // scroll delta conversions
#![allow(clippy::as_conversions)] // ratatui uses u16 extensively
#![allow(clippy::too_many_lines)] // render functions are naturally long
#![allow(clippy::match_same_arms)] // icon mappings share defaults
#![allow(clippy::trivially_copy_pass_by_ref)] // consistency with ratatui APIs

mod app;
mod events;
mod handlers;
mod history;
mod layout;
mod reducer;
mod render;
mod state;
mod theme;

// Submodules
pub mod components;

#[cfg(test)]
mod tests;

// Re-exports
pub use app::{ExitResult, StudioApp, run_studio};
pub use state::{Mode, StudioState};
