//! Commit count picker modal key handler
//!
//! Quick picker for "last N commits" - sets `from_ref` to HEAD~N

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{CommitCountTarget, Modal, Notification, StudioState};

use super::super::{
    reload_changelog_data, reload_pr_data, reload_release_notes_data, reload_review_data,
};

/// Handle key events in commit count modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Get current state
    let (input, target) = if let Some(Modal::CommitCount { input, target }) = &state.modal {
        (input.clone(), *target)
    } else {
        return vec![];
    };

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Parse the number and set HEAD~N
            let count: usize = input.parse().unwrap_or(0);
            if count == 0 {
                state.notify(Notification::error("Please enter a number > 0"));
                return vec![];
            }

            let ref_value = format!("HEAD~{count}");
            let (label, effect) = match target {
                CommitCountTarget::Pr => {
                    state.modes.pr.base_branch.clone_from(&ref_value);
                    ("PR base", reload_pr_data(state))
                }
                CommitCountTarget::Review => {
                    state.modes.review.from_ref.clone_from(&ref_value);
                    ("Review from", reload_review_data(state))
                }
                CommitCountTarget::Changelog => {
                    state.modes.changelog.from_ref.clone_from(&ref_value);
                    ("Changelog from", reload_changelog_data(state))
                }
                CommitCountTarget::ReleaseNotes => {
                    state.modes.release_notes.from_ref.clone_from(&ref_value);
                    ("Release Notes from", reload_release_notes_data(state))
                }
            };

            state.notify(Notification::info(format!(
                "{label} set to last {count} commits"
            )));
            state.close_modal();
            vec![effect]
        }
        // Quick presets: 1-9 for immediate selection
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if let Some(Modal::CommitCount { input, .. }) = &mut state.modal {
                input.push(c);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::CommitCount { input, .. }) = &mut state.modal {
                input.pop();
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
