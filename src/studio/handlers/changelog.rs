//! Changelog mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::state::{Modal, RefSelectorTarget, StudioState};

use super::Action;

/// Handle key events in Changelog mode
pub fn handle_changelog_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Select from ref
        KeyCode::Char('f') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::ChangelogFrom,
            });
            state.mark_dirty();
            Action::Redraw
        }
        // Select to ref
        KeyCode::Char('t') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::ChangelogTo,
            });
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}
