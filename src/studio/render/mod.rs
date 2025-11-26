//! Rendering modules for Iris Studio
//!
//! Splits rendering logic by mode for maintainability.

mod changelog;
mod commit;
mod explore;
mod modals;
mod pr;
mod release_notes;
mod review;

pub use changelog::render_changelog_panel;
pub use commit::render_commit_panel;
pub use explore::render_explore_panel;
pub use modals::render_modal;
pub use pr::render_pr_panel;
pub use release_notes::render_release_notes_panel;
pub use review::render_review_panel;
