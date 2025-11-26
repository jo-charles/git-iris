//! Explore mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::state::{PanelId, StudioState};

use super::{Action, IrisQueryRequest};

/// Handle key events in Explore mode
pub fn handle_explore_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match state.focused_panel {
        PanelId::Left => handle_file_tree_key(state, key),
        PanelId::Center => handle_code_view_key(state, key),
        PanelId::Right => handle_context_key(state, key),
    }
}

fn handle_file_tree_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.explore.file_tree.select_next();
            // Update current file from selection
            if let Some(entry) = state.modes.explore.file_tree.selected_entry()
                && !entry.is_dir
            {
                state.modes.explore.current_file = Some(entry.path);
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.explore.file_tree.select_prev();
            if let Some(entry) = state.modes.explore.file_tree.selected_entry()
                && !entry.is_dir
            {
                state.modes.explore.current_file = Some(entry.path);
            }
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
                    state.modes.explore.current_file = Some(entry.path);
                    state.focus_next_panel(); // Move to code view
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('g') => {
            // Go to first
            state.modes.explore.file_tree.select_first();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('G') => {
            // Go to last
            state.modes.explore.file_tree.select_last();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.explore.file_tree.page_down(10);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.explore.file_tree.page_up(10);
            state.mark_dirty();
            Action::Redraw
        }

        _ => Action::None,
    }
}

fn handle_code_view_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.explore.current_line = state.modes.explore.current_line.saturating_add(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.explore.current_line = state.modes.explore.current_line.saturating_sub(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('g') => {
            // Go to line (TODO: show input)
            Action::None
        }
        KeyCode::Char('G') => {
            // Go to end
            // TODO: Set to last line
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
                let line = state.modes.explore.current_line;
                let (start, end) = state.modes.explore.selection.unwrap_or((line, line));

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
