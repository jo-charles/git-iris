//! Search modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, Notification, StudioState};

/// Handle key events in search modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Get current query for filtering
    let query = if let Some(Modal::Search { query, .. }) = &state.modal {
        query.clone()
    } else {
        return vec![];
    };

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Select the current file and jump to it
            if let Some(Modal::Search {
                results, selected, ..
            }) = &state.modal
            {
                // Filter results by query
                let filtered: Vec<_> = results
                    .iter()
                    .filter(|r| {
                        query.is_empty() || r.to_lowercase().contains(&query.to_lowercase())
                    })
                    .collect();

                if let Some(file_path) = filtered.get(*selected) {
                    let path_str = (*file_path).clone();
                    let path = std::path::Path::new(&path_str);
                    state.close_modal();
                    // Try to select file in current mode's diff view
                    match state.active_mode {
                        crate::studio::state::Mode::Commit => {
                            state.modes.commit.diff_view.select_file_by_path(path);
                        }
                        crate::studio::state::Mode::Review => {
                            state.modes.review.diff_view.select_file_by_path(path);
                        }
                        crate::studio::state::Mode::PR => {
                            state.modes.pr.diff_view.select_file_by_path(path);
                        }
                        _ => {}
                    }
                    state.notify(Notification::info(format!("Jumped to {}", path_str)));
                    return vec![];
                }
            }
            state.close_modal();
            vec![]
        }
        KeyCode::Up => {
            if let Some(Modal::Search { selected, .. }) = &mut state.modal {
                *selected = selected.saturating_sub(1);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down => {
            if let Some(Modal::Search {
                results,
                selected,
                query,
            }) = &mut state.modal
            {
                let filtered_len = results
                    .iter()
                    .filter(|r| {
                        query.is_empty() || r.to_lowercase().contains(&query.to_lowercase())
                    })
                    .count();
                if *selected + 1 < filtered_len {
                    *selected += 1;
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char(c) => {
            if let Some(Modal::Search {
                query, selected, ..
            }) = &mut state.modal
            {
                query.push(c);
                *selected = 0; // Reset selection on filter change
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::Search {
                query, selected, ..
            }) = &mut state.modal
            {
                query.pop();
                *selected = 0; // Reset selection on filter change
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
