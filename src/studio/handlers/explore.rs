//! Explore mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::state::{Notification, PanelId, StudioState};

use super::{Action, IrisQueryRequest};

/// Default visible height for code view navigation (will be adjusted by actual render)
const DEFAULT_VISIBLE_HEIGHT: usize = 30;

/// Handle key events in Explore mode
pub fn handle_explore_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match state.focused_panel {
        PanelId::Left => handle_file_tree_key(state, key),
        PanelId::Center => handle_code_view_key(state, key),
        PanelId::Right => handle_context_key(state, key),
    }
}

/// Load the selected file into the code view
fn load_selected_file(state: &mut StudioState) {
    if let Some(entry) = state.modes.explore.file_tree.selected_entry()
        && !entry.is_dir
    {
        state.modes.explore.current_file = Some(entry.path.clone());
        if let Err(e) = state.modes.explore.code_view.load_file(&entry.path) {
            state.notify(Notification::warning(format!("Could not load file: {}", e)));
        }
    }
}

fn handle_file_tree_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.explore.file_tree.select_next();
            load_selected_file(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.explore.file_tree.select_prev();
            load_selected_file(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('h') | KeyCode::Left => {
            state.modes.explore.file_tree.collapse();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.modes.explore.file_tree.expand();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Toggle expand for directories, or select file and move to code view
            if let Some(entry) = state.modes.explore.file_tree.selected_entry() {
                if entry.is_dir {
                    state.modes.explore.file_tree.toggle_expand();
                } else {
                    load_selected_file(state);
                    state.focus_next_panel(); // Move to code view
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('g') => {
            // Go to first
            state.modes.explore.file_tree.select_first();
            load_selected_file(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('G') => {
            // Go to last
            state.modes.explore.file_tree.select_last();
            load_selected_file(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.explore.file_tree.page_down(10);
            load_selected_file(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.explore.file_tree.page_up(10);
            load_selected_file(state);
            state.mark_dirty();
            Action::Redraw
        }

        _ => Action::None,
    }
}

fn handle_code_view_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation - single line
        KeyCode::Char('j') | KeyCode::Down => {
            state
                .modes
                .explore
                .code_view
                .move_down(1, DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state
                .modes
                .explore
                .code_view
                .move_up(1, DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            Action::Redraw
        }
        // Page navigation
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state
                .modes
                .explore
                .code_view
                .move_down(20, DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state
                .modes
                .explore
                .code_view
                .move_up(20, DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('g') => {
            // Go to first line
            state.modes.explore.code_view.goto_first();
            state.modes.explore.current_line = 1;
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('G') => {
            // Go to last line
            state
                .modes
                .explore
                .code_view
                .goto_last(DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            Action::Redraw
        }

        // Heat map toggle
        KeyCode::Char('H') => {
            state.modes.explore.show_heat_map = !state.modes.explore.show_heat_map;
            state.mark_dirty();
            Action::Redraw
        }

        // Ask "why" about current line
        KeyCode::Char('w') => {
            if let Some(file) = &state.modes.explore.current_file {
                let line = state.modes.explore.code_view.selected_line();
                let (start, end) = state
                    .modes
                    .explore
                    .code_view
                    .selection()
                    .unwrap_or((line, line));

                return Action::IrisQuery(IrisQueryRequest::SemanticBlame {
                    file: file.clone(),
                    start_line: start,
                    end_line: end,
                });
            }
            Action::None
        }

        // Open in $EDITOR
        KeyCode::Char('o') => {
            // TODO: Open in external editor
            Action::None
        }

        _ => Action::None,
    }
}

fn handle_context_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            // Scroll context panel down
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Scroll context panel up
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Drill into selected context item
            Action::None
        }

        _ => Action::None,
    }
}
