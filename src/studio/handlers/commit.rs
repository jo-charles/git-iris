//! Commit mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::state::{Modal, PanelId, StudioState};

use super::{Action, IrisQueryRequest};

/// Handle key events in Commit mode
pub fn handle_commit_key(state: &mut StudioState, key: KeyEvent) -> Action {
    // If editing message, handle text input
    if state.modes.commit.editing_message {
        return handle_editing_key(state, key);
    }

    match state.focused_panel {
        PanelId::Left => handle_files_key(state, key),
        PanelId::Center => handle_message_key(state, key),
        PanelId::Right => handle_diff_key(state, key),
    }
}

fn handle_editing_key(state: &mut StudioState, key: KeyEvent) -> Action {
    // Forward to message editor - it handles Esc internally
    if state.modes.commit.message_editor.handle_key(key) {
        // Sync editing state from component
        state.modes.commit.editing_message = state.modes.commit.message_editor.is_editing();
        state.mark_dirty();
        Action::Redraw
    } else {
        Action::None
    }
}

/// Sync file tree selection with diff view
fn sync_file_selection(state: &mut StudioState) {
    if let Some(path) = state.modes.commit.file_tree.selected_path() {
        state.modes.commit.diff_view.select_file_by_path(&path);
    }
}

fn handle_files_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.commit.file_tree.select_next();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.commit.file_tree.select_prev();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('h') | KeyCode::Left => {
            state.modes.commit.file_tree.collapse();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.modes.commit.file_tree.expand();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('g') => {
            state.modes.commit.file_tree.select_first();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('G') => {
            state.modes.commit.file_tree.select_last();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.file_tree.page_down(10);
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.file_tree.page_up(10);
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }

        // Stage/unstage
        KeyCode::Char('s') => {
            // Stage selected file
            // TODO: Implement git staging
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('U') => {
            // Unstage selected file (capital U to avoid conflict with page up)
            // TODO: Implement git unstaging
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('a') => {
            // Stage all
            // TODO: Implement git stage all
            state.mark_dirty();
            Action::Redraw
        }

        KeyCode::Enter => {
            // Toggle expand for directories, or select file and move to diff view
            if let Some(entry) = state.modes.commit.file_tree.selected_entry() {
                if entry.is_dir {
                    state.modes.commit.file_tree.toggle_expand();
                } else {
                    // Sync diff view and move focus to diff panel (right)
                    sync_file_selection(state);
                    state.focused_panel = PanelId::Right;
                }
            }
            state.mark_dirty();
            Action::Redraw
        }

        _ => Action::None,
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation - scroll by line
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.commit.diff_view.scroll_down(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.commit.diff_view.scroll_up(1);
            state.mark_dirty();
            Action::Redraw
        }
        // Page scrolling
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.diff_view.scroll_down(20);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.diff_view.scroll_up(20);
            state.mark_dirty();
            Action::Redraw
        }
        // Hunk navigation
        KeyCode::Char(']') => {
            state.modes.commit.diff_view.next_hunk();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('[') => {
            state.modes.commit.diff_view.prev_hunk();
            state.mark_dirty();
            Action::Redraw
        }
        // File navigation within diff
        KeyCode::Char('n') => {
            state.modes.commit.diff_view.next_file();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('p') => {
            state.modes.commit.diff_view.prev_file();
            state.mark_dirty();
            Action::Redraw
        }

        _ => Action::None,
    }
}

fn handle_message_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Edit message
        KeyCode::Char('e') => {
            state.modes.commit.message_editor.enter_edit_mode();
            state.modes.commit.editing_message = true;
            state.mark_dirty();
            Action::Redraw
        }

        // Regenerate message
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating commit message...");
            state.modes.commit.generating = true;
            Action::IrisQuery(IrisQueryRequest::GenerateCommit {
                instructions: if state.modes.commit.custom_instructions.is_empty() {
                    None
                } else {
                    Some(state.modes.commit.custom_instructions.clone())
                },
            })
        }

        // Reset to original message
        KeyCode::Char('R') => {
            state.modes.commit.message_editor.reset();
            state.mark_dirty();
            Action::Redraw
        }

        // Custom instructions - open input modal
        KeyCode::Char('i') => {
            state.modal = Some(Modal::Instructions {
                input: state.modes.commit.custom_instructions.clone(),
            });
            state.mark_dirty();
            Action::Redraw
        }

        // Commit - use message from editor (may have been modified)
        KeyCode::Enter => {
            let message = state.modes.commit.message_editor.get_message();
            if message.is_empty() {
                Action::None
            } else {
                Action::Commit(message)
            }
        }

        // Navigate between generated messages
        KeyCode::Char('n') | KeyCode::Right => {
            state.modes.commit.message_editor.next_message();
            // Sync index for backward compat
            state.modes.commit.current_index = state.modes.commit.message_editor.selected_index();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('p') | KeyCode::Left => {
            state.modes.commit.message_editor.prev_message();
            state.modes.commit.current_index = state.modes.commit.message_editor.selected_index();
            state.mark_dirty();
            Action::Redraw
        }

        _ => Action::None,
    }
}
