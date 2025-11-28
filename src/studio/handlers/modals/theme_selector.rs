//! Theme selector modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, Notification, StudioState};
use crate::theme;

/// Visible items in the list (modal height - header - footer - variant separators)
const VISIBLE_ITEMS: usize = 16;

/// Handle key events in theme selector modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Verify we're in the right modal
    let Some(Modal::ThemeSelector {
        themes, selected, ..
    }) = &state.modal
    else {
        return vec![];
    };
    let themes = themes.clone();
    let selected = *selected;

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Apply selected theme
            if let Some(theme_info) = themes.get(selected) {
                // Load the theme
                if theme::load_theme_by_name(&theme_info.id).is_ok() {
                    // Theme has been applied (done above)
                    state.notify(Notification::success(format!(
                        "Theme: {}",
                        theme_info.display_name
                    )));
                } else {
                    state.notify(Notification::error(format!(
                        "Failed to load theme: {}",
                        theme_info.id
                    )));
                }
            }
            state.close_modal();
            vec![]
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(Modal::ThemeSelector {
                selected,
                scroll,
                themes,
                input,
            }) = &mut state.modal
            {
                // Get filtered indices
                let filtered_indices: Vec<usize> = themes
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| {
                        input.is_empty()
                            || t.display_name
                                .to_lowercase()
                                .contains(&input.to_lowercase())
                            || t.author.to_lowercase().contains(&input.to_lowercase())
                    })
                    .map(|(i, _)| i)
                    .collect();

                // Find current position in filtered list
                if let Some(pos) = filtered_indices.iter().position(|&i| i == *selected)
                    && pos > 0
                {
                    *selected = filtered_indices[pos - 1];
                    // Scroll up if selection goes above visible area
                    if pos - 1 < *scroll {
                        *scroll = pos - 1;
                    }
                    // Apply live preview
                    if let Some(theme_info) = themes.get(*selected) {
                        let _ = theme::load_theme_by_name(&theme_info.id);
                    }
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(Modal::ThemeSelector {
                selected,
                scroll,
                themes,
                input,
            }) = &mut state.modal
            {
                // Get filtered indices
                let filtered_indices: Vec<usize> = themes
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| {
                        input.is_empty()
                            || t.display_name
                                .to_lowercase()
                                .contains(&input.to_lowercase())
                            || t.author.to_lowercase().contains(&input.to_lowercase())
                    })
                    .map(|(i, _)| i)
                    .collect();

                // Find current position in filtered list
                if let Some(pos) = filtered_indices.iter().position(|&i| i == *selected)
                    && pos + 1 < filtered_indices.len()
                {
                    *selected = filtered_indices[pos + 1];
                    // Scroll down if selection goes below visible area
                    if pos + 1 >= *scroll + VISIBLE_ITEMS {
                        *scroll = pos + 1 - VISIBLE_ITEMS + 1;
                    }
                    // Apply live preview
                    if let Some(theme_info) = themes.get(*selected) {
                        let _ = theme::load_theme_by_name(&theme_info.id);
                    }
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char(c) => {
            if let Some(Modal::ThemeSelector {
                input,
                selected,
                scroll,
                themes,
            }) = &mut state.modal
            {
                input.push(c);
                *scroll = 0;
                // Select first matching theme
                let first_match = themes
                    .iter()
                    .enumerate()
                    .find(|(_, t)| {
                        input.is_empty()
                            || t.display_name
                                .to_lowercase()
                                .contains(&input.to_lowercase())
                            || t.author.to_lowercase().contains(&input.to_lowercase())
                    })
                    .map(|(i, _)| i);
                *selected = first_match.unwrap_or(0);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::ThemeSelector {
                input,
                selected,
                scroll,
                themes,
            }) = &mut state.modal
            {
                input.pop();
                *scroll = 0;
                // Select first matching theme
                let first_match = themes
                    .iter()
                    .enumerate()
                    .find(|(_, t)| {
                        input.is_empty()
                            || t.display_name
                                .to_lowercase()
                                .contains(&input.to_lowercase())
                            || t.author.to_lowercase().contains(&input.to_lowercase())
                    })
                    .map(|(i, _)| i);
                *selected = first_match.unwrap_or(0);
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
