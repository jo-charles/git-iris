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

/// Render the currently active modal, if any
pub fn render_modal(state: &StudioState, frame: &mut Frame, last_render: Instant) {
    let Some(modal) = &state.modal else {
        return;
    };
    let area = frame.area();

    // Center modal in screen - chat takes most of the screen
    let (modal_width, modal_height) = match modal {
        Modal::Chat => (
            (area.width * 4 / 5)
                .max(100)
                .min(area.width.saturating_sub(4)),
            (area.height * 4 / 5).min(area.height.saturating_sub(4)),
        ),
        Modal::Help => (70.min(area.width.saturating_sub(4)), 30),
        Modal::Instructions { .. } => (60.min(area.width.saturating_sub(4)), 8),
        Modal::Search { .. } => (60.min(area.width.saturating_sub(4)), 15),
        Modal::Confirm { .. } => (60.min(area.width.saturating_sub(4)), 6),
        Modal::RefSelector { .. } => (50.min(area.width.saturating_sub(4)), 15),
        Modal::PresetSelector { .. } => (70.min(area.width.saturating_sub(4)), 24),
        Modal::EmojiSelector { .. } => (55.min(area.width.saturating_sub(4)), 24),
        Modal::Settings(_) => (60.min(area.width.saturating_sub(4)), 18),
    };
    let modal_height = modal_height.min(area.height.saturating_sub(4));

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
