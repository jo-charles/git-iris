//! Commit mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::events::SideEffect;
use crate::studio::state::{EmojiMode, Modal, PanelId, StudioState};

use super::{copy_to_clipboard, spawn_commit_task};

/// Handle key events in Commit mode
pub fn handle_commit_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
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

fn handle_editing_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Forward to message editor - it handles Esc internally
    if state.modes.commit.message_editor.handle_key(key) {
        // Sync editing state from component
        state.modes.commit.editing_message = state.modes.commit.message_editor.is_editing();
        state.mark_dirty();
    }
    vec![]
}

/// Sync file tree selection with diff view
fn sync_file_selection(state: &mut StudioState) {
    if let Some(path) = state.modes.commit.file_tree.selected_path() {
        state.modes.commit.diff_view.select_file_by_path(&path);
    }
}

fn handle_files_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.commit.file_tree.select_next();
            sync_file_selection(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.commit.file_tree.select_prev();
            sync_file_selection(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('h') | KeyCode::Left => {
            state.modes.commit.file_tree.collapse();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.modes.commit.file_tree.expand();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('g') => {
            state.modes.commit.file_tree.select_first();
            sync_file_selection(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('G') => {
            state.modes.commit.file_tree.select_last();
            sync_file_selection(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.file_tree.page_down(10);
            sync_file_selection(state);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.file_tree.page_up(10);
            sync_file_selection(state);
            state.mark_dirty();
            vec![]
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
            vec![]
        }

        // Stage selected file
        KeyCode::Char('s') => {
            if let Some(path) = state.modes.commit.file_tree.selected_path() {
                vec![SideEffect::GitStage(path)]
            } else {
                vec![]
            }
        }

        // Unstage selected file
        KeyCode::Char('u') => {
            if let Some(path) = state.modes.commit.file_tree.selected_path() {
                vec![SideEffect::GitUnstage(path)]
            } else {
                vec![]
            }
        }

        // Stage all files
        KeyCode::Char('a') => vec![SideEffect::GitStageAll],

        // Unstage all files
        KeyCode::Char('U') => vec![SideEffect::GitUnstageAll],

        _ => vec![],
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Navigation - scroll by line
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.commit.diff_view.scroll_down(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.commit.diff_view.scroll_up(1);
            state.mark_dirty();
            vec![]
        }
        // Page scrolling
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.diff_view.scroll_down(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.diff_view.scroll_up(20);
            state.mark_dirty();
            vec![]
        }
        // Hunk navigation
        KeyCode::Char(']') => {
            state.modes.commit.diff_view.next_hunk();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('[') => {
            state.modes.commit.diff_view.prev_hunk();
            state.mark_dirty();
            vec![]
        }
        // File navigation within diff
        KeyCode::Char('n') => {
            state.modes.commit.diff_view.next_file();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('p') => {
            state.modes.commit.diff_view.prev_file();
            state.mark_dirty();
            vec![]
        }

        _ => vec![],
    }
}

fn handle_message_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Edit message
        KeyCode::Char('e') => {
            state.modes.commit.message_editor.enter_edit_mode();
            state.modes.commit.editing_message = true;
            state.mark_dirty();
            vec![]
        }

        // Open preset selector
        KeyCode::Char('p') => {
            let presets = state.get_commit_presets();
            state.modal = Some(Modal::PresetSelector {
                input: String::new(),
                presets,
                selected: 0,
                scroll: 0,
            });
            state.mark_dirty();
            vec![]
        }

        // Regenerate message
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating commit message...");
            state.modes.commit.generating = true;
            vec![spawn_commit_task(state)]
        }

        // Reset to original message
        KeyCode::Char('R') => {
            state.modes.commit.message_editor.reset();
            state.mark_dirty();
            vec![]
        }

        // Custom instructions - open input modal
        KeyCode::Char('i') => {
            state.modal = Some(Modal::Instructions {
                input: state.modes.commit.custom_instructions.clone(),
            });
            state.mark_dirty();
            vec![]
        }

        // Open emoji selector
        KeyCode::Char('g') => {
            let emojis = state.get_emoji_list();
            // Find current selection index
            let selected = match &state.modes.commit.emoji_mode {
                EmojiMode::None => 0,
                EmojiMode::Auto => 1,
                EmojiMode::Custom(emoji) => {
                    emojis.iter().position(|e| e.emoji == *emoji).unwrap_or(1)
                }
            };
            state.modal = Some(Modal::EmojiSelector {
                input: String::new(),
                emojis,
                selected,
                scroll: 0,
            });
            state.mark_dirty();
            vec![]
        }

        // Quick toggle emoji between None and Auto
        KeyCode::Char('E') => {
            state.modes.commit.emoji_mode = match state.modes.commit.emoji_mode {
                EmojiMode::None => EmojiMode::Auto,
                _ => EmojiMode::None,
            };
            // Sync legacy flag
            state.modes.commit.use_gitmoji = state.modes.commit.emoji_mode != EmojiMode::None;
            let status = match &state.modes.commit.emoji_mode {
                EmojiMode::None => "off",
                EmojiMode::Auto => "auto",
                EmojiMode::Custom(e) => e,
            };
            state.notify(crate::studio::state::Notification::info(format!(
                "Emoji: {}",
                status
            )));
            state.mark_dirty();
            vec![]
        }

        // Commit - use message from editor (may have been modified)
        KeyCode::Enter => {
            let message = state.modes.commit.message_editor.get_message();
            if message.is_empty() {
                vec![]
            } else {
                vec![SideEffect::ExecuteCommit { message }]
            }
        }

        // Navigate between generated messages (arrow keys only, n/p reserved for other uses)
        KeyCode::Right => {
            state.modes.commit.message_editor.next_message();
            // Sync index for backward compat
            state.modes.commit.current_index = state.modes.commit.message_editor.selected_index();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Left => {
            state.modes.commit.message_editor.prev_message();
            state.modes.commit.current_index = state.modes.commit.message_editor.selected_index();
            state.mark_dirty();
            vec![]
        }

        // Copy to clipboard
        KeyCode::Char('y') => {
            let message = state.modes.commit.message_editor.get_message();
            if !message.is_empty() {
                copy_to_clipboard(state, &message, "Commit message");
            }
            vec![]
        }

        _ => vec![],
    }
}
