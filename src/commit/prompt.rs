//! Commit message post-processing utilities

use crate::gitmoji::apply_gitmoji;

/// Post-processes a commit message, applying gitmoji if enabled
pub fn process_commit_message(message: String, use_gitmoji: bool) -> String {
    if use_gitmoji {
        apply_gitmoji(&message)
    } else {
        message
    }
}
