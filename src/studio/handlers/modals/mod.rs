//! Modal key handlers for Iris Studio
//!
//! Each modal type has its own handler module for maintainability.

mod chat;
mod confirm;
mod emoji_selector;
mod instructions;
mod preset_selector;
mod ref_selector;
mod search;
mod settings;
mod theme_selector;

use crossterm::event::KeyEvent;

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, StudioState};

/// Handle key events when a modal is open
pub fn handle_modal_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match &state.modal {
        Some(Modal::Help) => {
            // Any key closes help
            state.close_modal();
            vec![]
        }
        Some(Modal::Search { .. }) => search::handle(state, key),
        Some(Modal::Confirm { .. }) => confirm::handle(state, key),
        Some(Modal::Instructions { .. }) => instructions::handle(state, key),
        Some(Modal::Chat) => chat::handle(state, key),
        Some(Modal::RefSelector { .. }) => ref_selector::handle(state, key),
        Some(Modal::PresetSelector { .. }) => preset_selector::handle(state, key),
        Some(Modal::EmojiSelector { .. }) => emoji_selector::handle(state, key),
        Some(Modal::Settings(_)) => settings::handle(state, key),
        Some(Modal::ThemeSelector { .. }) => theme_selector::handle(state, key),
        None => vec![],
    }
}
