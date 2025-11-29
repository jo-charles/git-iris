//! Git-related event handlers
//!
//! Handles file staging, unstaging, git status refresh, and file selection.

use std::path::{Path, PathBuf};

use super::super::events::SideEffect;
use super::super::state::{Mode, Notification, StudioState};

/// Handle `FileStaged` event
pub fn file_staged(state: &mut StudioState, path: &Path) -> Vec<SideEffect> {
    state.notify(Notification::success(format!("Staged: {}", path.display())));
    vec![SideEffect::RefreshGitStatus]
}

/// Handle `FileUnstaged` event
pub fn file_unstaged(state: &mut StudioState, path: &Path) -> Vec<SideEffect> {
    state.notify(Notification::info(format!("Unstaged: {}", path.display())));
    vec![SideEffect::RefreshGitStatus]
}

/// Handle `RefreshGitStatus` event
pub fn refresh_git_status() -> Vec<SideEffect> {
    vec![SideEffect::RefreshGitStatus]
}

/// Handle `GitStatusRefreshed` event
pub fn git_status_refreshed(state: &mut StudioState) {
    state.mark_dirty();
}

/// Handle `SelectFile` event
pub fn select_file(state: &mut StudioState, path: PathBuf) {
    match state.active_mode {
        Mode::Explore => {
            state.modes.explore.current_file = Some(path);
        }
        Mode::Commit => {
            // Find index of file in staged files
            if let Some(idx) = state
                .git_status
                .staged_files
                .iter()
                .position(|f| f == &path)
            {
                state.modes.commit.selected_file_index = idx;
            }
        }
        _ => {}
    }
    state.mark_dirty();
}
