//! Chat modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, StudioState};

use super::super::spawn_chat_task;

/// Handle key events in chat modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Get current state for Enter handling
    let (current_input, is_responding, mode) = if let Some(Modal::Chat(chat)) = &state.modal {
        (chat.input.clone(), chat.is_responding, state.active_mode)
    } else {
        return vec![];
    };

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Send message if not empty and not already responding
            if !current_input.is_empty() && !is_responding {
                let message = current_input;
                if let Some(Modal::Chat(chat)) = &mut state.modal {
                    chat.add_user_message(&message);
                    chat.is_responding = true;
                }
                state.mark_dirty();
                vec![spawn_chat_task(message, mode)]
            } else {
                vec![]
            }
        }
        KeyCode::Char(c) => {
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.input.push(c);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.input.pop();
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Up => {
            // Scroll up in chat history
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.scroll_up(1);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down => {
            // Scroll down in chat history
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                // We don't know max_scroll here, so just increment and let render clamp
                chat.scroll_offset = chat.scroll_offset.saturating_add(1);
                // Don't re-enable auto_scroll here - let render handle it when at bottom
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp => {
            // Scroll up faster
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.scroll_up(10);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown => {
            // Scroll down faster
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.scroll_offset = chat.scroll_offset.saturating_add(10);
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
