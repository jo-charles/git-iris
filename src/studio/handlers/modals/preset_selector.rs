//! Preset selector modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, Notification, StudioState};

use super::super::spawn_commit_task;

/// Visible items in the list (modal height - header - footer)
const VISIBLE_ITEMS: usize = 18;

/// Handle key events in preset selector modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Get current state for filtering
    let (input, presets, selected) = if let Some(Modal::PresetSelector {
        input,
        presets,
        selected,
        ..
    }) = &state.modal
    {
        (input.clone(), presets.clone(), *selected)
    } else {
        return vec![];
    };

    // Filter presets based on input
    let filtered: Vec<_> = presets
        .iter()
        .filter(|p| {
            input.is_empty()
                || p.name.to_lowercase().contains(&input.to_lowercase())
                || p.key.to_lowercase().contains(&input.to_lowercase())
        })
        .collect();

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Apply selection and auto-regenerate
            if let Some(preset) = filtered.get(selected) {
                state.modes.commit.preset.clone_from(&preset.key);
                state.notify(Notification::info(format!(
                    "Preset: {} {} - regenerating...",
                    preset.emoji, preset.name
                )));
                state.close_modal();
                state.set_iris_thinking("Generating commit message...");
                state.modes.commit.generating = true;
                return vec![spawn_commit_task(state)];
            }
            state.close_modal();
            vec![]
        }
        KeyCode::Up => {
            if let Some(Modal::PresetSelector {
                selected, scroll, ..
            }) = &mut state.modal
            {
                *selected = selected.saturating_sub(1);
                // Scroll up if selection goes above visible area
                if *selected < *scroll {
                    *scroll = *selected;
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down => {
            if let Some(Modal::PresetSelector {
                selected,
                scroll,
                presets,
                input,
            }) = &mut state.modal
            {
                let filtered_len = presets
                    .iter()
                    .filter(|p| {
                        input.is_empty()
                            || p.name.to_lowercase().contains(&input.to_lowercase())
                            || p.key.to_lowercase().contains(&input.to_lowercase())
                    })
                    .count();
                if *selected + 1 < filtered_len {
                    *selected += 1;
                    // Scroll down if selection goes below visible area
                    if *selected >= *scroll + VISIBLE_ITEMS {
                        *scroll = *selected - VISIBLE_ITEMS + 1;
                    }
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char(c) => {
            if let Some(Modal::PresetSelector {
                input,
                selected,
                scroll,
                ..
            }) = &mut state.modal
            {
                input.push(c);
                *selected = 0; // Reset selection on filter change
                *scroll = 0;
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::PresetSelector {
                input,
                selected,
                scroll,
                ..
            }) = &mut state.modal
            {
                input.pop();
                *selected = 0;
                *scroll = 0;
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
