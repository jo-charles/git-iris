//! Confirm modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, StudioState};

/// Handle key events in confirm modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        KeyCode::Char('y' | 'Y') | KeyCode::Enter => {
            // Get the action from the modal before closing
            let action = if let Some(Modal::Confirm { action, .. }) = &state.modal {
                action.clone()
            } else {
                return vec![];
            };

            state.close_modal();

            // Execute the confirmed action
            match action.as_str() {
                "commit" => {
                    // Get the commit message and execute
                    if let Some(msg) = state
                        .modes
                        .commit
                        .messages
                        .get(state.modes.commit.current_index)
                    {
                        let message = crate::types::format_commit_message(msg);
                        vec![SideEffect::ExecuteCommit { message }]
                    } else {
                        vec![]
                    }
                }
                "amend" => {
                    // Get the commit message and execute amend
                    if let Some(msg) = state
                        .modes
                        .commit
                        .messages
                        .get(state.modes.commit.current_index)
                    {
                        let message = crate::types::format_commit_message(msg);
                        vec![SideEffect::ExecuteAmend { message }]
                    } else {
                        vec![]
                    }
                }
                "quit" => vec![SideEffect::Quit],
                _ => vec![],
            }
        }
        KeyCode::Char('n' | 'N') | KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        _ => vec![],
    }
}
