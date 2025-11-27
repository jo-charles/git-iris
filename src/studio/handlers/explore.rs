//! Explore mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::events::SideEffect;
use crate::studio::state::{Notification, PanelId, StudioState};

/// Default visible height for code view navigation (will be adjusted by actual render)
const DEFAULT_VISIBLE_HEIGHT: usize = 30;

/// Handle key events in Explore mode
pub fn handle_explore_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
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

fn handle_file_tree_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.explore.file_tree.select_next();
            load_selected_file(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.explore.file_tree.select_prev();
            load_selected_file(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('h') | KeyCode::Left => {
            state.modes.explore.file_tree.collapse();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.modes.explore.file_tree.expand();
            state.mark_dirty();
            vec![]
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
            vec![]
        }
        KeyCode::Char('g') => {
            // Go to first
            state.modes.explore.file_tree.select_first();
            load_selected_file(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('G') => {
            // Go to last
            state.modes.explore.file_tree.select_last();
            load_selected_file(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.explore.file_tree.page_down(10);
            load_selected_file(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.explore.file_tree.page_up(10);
            load_selected_file(state);
            state.mark_dirty();
            vec![]
        }

        _ => vec![],
    }
}

fn handle_code_view_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
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
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state
                .modes
                .explore
                .code_view
                .move_up(1, DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            vec![]
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
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state
                .modes
                .explore
                .code_view
                .move_up(20, DEFAULT_VISIBLE_HEIGHT);
            state.modes.explore.current_line = state.modes.explore.code_view.selected_line();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('g') => {
            // Go to first line
            state.modes.explore.code_view.goto_first();
            state.modes.explore.current_line = 1;
            state.mark_dirty();
            vec![]
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
            vec![]
        }

        // Heat map toggle
        KeyCode::Char('H') => {
            state.modes.explore.show_heat_map = !state.modes.explore.show_heat_map;
            state.mark_dirty();
            vec![]
        }

        // Ask "why" about current line - semantic blame
        KeyCode::Char('w') => {
            let file = state.modes.explore.current_file.clone();
            if let Some(file) = file {
                if state.modes.explore.blame_loading {
                    state.notify(Notification::info("Already analyzing..."));
                    state.mark_dirty();
                    vec![]
                } else {
                    let line = state.modes.explore.current_line;
                    let end_line = state.modes.explore.selection.map_or(line, |(_, end)| end);

                    // Set loading state
                    state.modes.explore.blame_loading = true;
                    state.set_iris_thinking("Analyzing code history...");
                    state.mark_dirty();

                    vec![SideEffect::GatherBlameAndSpawnAgent {
                        file,
                        start_line: line,
                        end_line,
                    }]
                }
            } else {
                state.notify(Notification::warning("No file selected"));
                state.mark_dirty();
                vec![]
            }
        }

        // Copy current line to clipboard
        KeyCode::Char('y') => {
            let line_idx = state
                .modes
                .explore
                .code_view
                .selected_line()
                .saturating_sub(1);
            let line_content = state.modes.explore.code_view.lines().get(line_idx).cloned();
            if let Some(content) = line_content {
                state.notify(Notification::success("Line copied to clipboard"));
                state.mark_dirty();
                vec![SideEffect::CopyToClipboard(content)]
            } else {
                state.notify(Notification::warning("Nothing to copy"));
                state.mark_dirty();
                vec![]
            }
        }

        // Copy entire file content
        KeyCode::Char('Y') => {
            let content = state.modes.explore.code_view.lines().join("\n");
            if content.is_empty() {
                state.notify(Notification::warning("No content to copy"));
                state.mark_dirty();
                vec![]
            } else {
                state.notify(Notification::success("File content copied to clipboard"));
                state.mark_dirty();
                vec![SideEffect::CopyToClipboard(content)]
            }
        }

        // Open in $EDITOR
        KeyCode::Char('o') => {
            if state.modes.explore.current_file.is_some() {
                // TODO: Full implementation needs terminal suspend/restore
                // For now, show a message with the command that would be run
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
                state.notify(Notification::info(format!(
                    "Open in editor: use '{} +{} <file>' outside TUI",
                    editor, state.modes.explore.current_line
                )));
            } else {
                state.notify(Notification::warning("No file selected"));
            }
            state.mark_dirty();
            vec![]
        }

        _ => vec![],
    }
}

fn handle_context_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            // Scroll context panel down
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Scroll context panel up
            state.mark_dirty();
            vec![]
        }
        KeyCode::Enter => {
            // Drill into selected context item
            vec![]
        }

        _ => vec![],
    }
}
