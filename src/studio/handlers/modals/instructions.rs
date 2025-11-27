//! Instructions modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, StudioState};

use super::super::spawn_commit_task;

/// Handle key events in instructions modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Get current input for Enter handling
    let current_input = if let Some(Modal::Instructions { input }) = &state.modal {
        input.clone()
    } else {
        return vec![];
    };

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Generate commit with instructions
            let instructions = if current_input.is_empty() {
                None
            } else {
                Some(current_input)
            };
            // Store custom instructions for future use
            if let Some(ref instr) = instructions {
                state.modes.commit.custom_instructions.clone_from(instr);
            }
            state.close_modal();
            state.set_iris_thinking("Generating commit message...");
            state.modes.commit.generating = true;
            vec![spawn_commit_task(state)]
        }
        KeyCode::Char(c) => {
            if let Some(Modal::Instructions { input }) = &mut state.modal {
                input.push(c);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::Instructions { input }) = &mut state.modal {
                input.pop();
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
