//! Modal rendering for Iris Studio
//!
//! Each modal type has its own module for maintainability.

mod chat_modal;
mod confirm;
mod emoji_selector;
mod help;
mod instructions;
mod preset_selector;
mod ref_selector;
mod search;
mod settings;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;
use std::time::Instant;

use crate::studio::state::{Modal, StudioState};

/// Calculate responsive modal size based on terminal dimensions and modal type
fn modal_size(modal: &Modal, area: Rect) -> (u16, u16) {
    let max_width = area.width.saturating_sub(4);
    let max_height = area.height.saturating_sub(4);

    match modal {
        // Chat modal takes most of the screen
        Modal::Chat => (
            (area.width * 4 / 5).max(80).min(max_width),
            (area.height * 4 / 5).min(max_height),
        ),
        // Help modal uses available height (40 lines or less)
        Modal::Help => (70.min(max_width), 40.min(max_height)),
        // Instructions modal is compact
        Modal::Instructions { .. } => (60.min(max_width), 8.min(max_height)),
        // Search modal with results
        Modal::Search { .. } => (60.min(max_width), 15.min(max_height)),
        // Confirm modal is minimal
        Modal::Confirm { .. } => (60.min(max_width), 6.min(max_height)),
        // RefSelector sizes based on content when possible
        Modal::RefSelector { refs, .. } => {
            let needed_width = refs
                .iter()
                .map(std::string::String::len)
                .max()
                .unwrap_or(30) as u16
                + 10;
            let list_height = (refs.len() as u16 + 4).min(20);
            (
                needed_width.max(40).min(max_width),
                list_height.min(max_height),
            )
        }
        // Preset selector with scrollable list
        Modal::PresetSelector { presets, .. } => {
            let list_height = (presets.len() as u16 + 5).min(30);
            (70.min(max_width), list_height.min(max_height))
        }
        // Emoji selector grid
        Modal::EmojiSelector { .. } => (55.min(max_width), 26.min(max_height)),
        // Settings modal
        Modal::Settings(_) => (60.min(max_width), 20.min(max_height)),
    }
}

/// Render the currently active modal, if any
pub fn render_modal(state: &StudioState, frame: &mut Frame, last_render: Instant) {
    let Some(modal) = &state.modal else {
        return;
    };
    let area = frame.area();

    // Calculate responsive modal size
    let (modal_width, modal_height) = modal_size(modal, area);

    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    // Clear the area first
    frame.render_widget(Clear, modal_area);

    match modal {
        Modal::Help => help::render(frame, modal_area),
        Modal::Instructions { input } => instructions::render(frame, modal_area, input),
        Modal::Search {
            query,
            results,
            selected,
        } => {
            search::render(frame, modal_area, query, results, *selected);
        }
        Modal::Confirm { message, .. } => confirm::render(frame, modal_area, message),
        Modal::Chat => chat_modal::render(frame, modal_area, &state.chat_state, last_render),
        Modal::RefSelector {
            input,
            refs,
            selected,
            target,
        } => ref_selector::render(frame, modal_area, input, refs, *selected, *target),
        Modal::PresetSelector {
            input,
            presets,
            selected,
            scroll,
        } => preset_selector::render(frame, modal_area, input, presets, *selected, *scroll),
        Modal::EmojiSelector {
            input,
            emojis,
            selected,
            scroll,
        } => emoji_selector::render(frame, modal_area, input, emojis, *selected, *scroll),
        Modal::Settings(settings_state) => settings::render(frame, modal_area, settings_state),
    }
}
