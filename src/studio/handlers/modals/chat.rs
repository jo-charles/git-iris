//! Chat modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, StudioState};

use super::super::spawn_chat_task;

/// Handle key events in chat modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Verify chat modal is open
    if !matches!(state.modal, Some(Modal::Chat)) {
        return vec![];
    }

    // Get state needed before potential mutation
    let current_input = state.chat_state.input.clone();
    let is_responding = state.chat_state.is_responding;
    let mode = state.active_mode;

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Send message if not empty and not already responding
            if !current_input.is_empty() && !is_responding {
                state.chat_state.add_user_message(&current_input);
                state.chat_state.is_responding = true;
                state.mark_dirty();
                vec![spawn_chat_task(current_input, mode)]
            } else {
                vec![]
            }
        }
        KeyCode::Char(c) => {
            state.chat_state.input.push(c);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            state.chat_state.input.pop();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Up => {
            // Scroll up in chat history
            state.chat_state.scroll_up(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down => {
            // Scroll down in chat history
            let max_scroll = state.chat_state.estimated_max_scroll();
            state.chat_state.scroll_down(1, max_scroll);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp => {
            // Scroll up faster
            state.chat_state.scroll_up(10);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown => {
            // Scroll down faster
            let max_scroll = state.chat_state.estimated_max_scroll();
            state.chat_state.scroll_down(10, max_scroll);
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
