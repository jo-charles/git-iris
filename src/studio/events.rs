//! Event handling for Iris Studio
//!
//! Keyboard input processing and action dispatch.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::{Modal, Mode, StudioState};

// ═══════════════════════════════════════════════════════════════════════════════
// Action Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of processing an input event
#[derive(Debug, Clone)]
pub enum Action {
    /// No action, continue running
    None,
    /// Quit the application
    Quit,
    /// Request redraw
    Redraw,
    /// Iris query request
    IrisQuery(IrisQueryRequest),
    /// Perform a commit
    Commit(String),
}

/// Request to query Iris agent
#[derive(Debug, Clone)]
pub enum IrisQueryRequest {
    /// Semantic blame for a code location
    SemanticBlame {
        file: std::path::PathBuf,
        start_line: usize,
        end_line: usize,
    },
    /// Generate commit message
    GenerateCommit { instructions: Option<String> },
    /// Chat with Iris
    Chat { message: String },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Event Handler
// ═══════════════════════════════════════════════════════════════════════════════

/// Process a key event and return the resulting action
pub fn handle_key_event(state: &mut StudioState, key: KeyEvent) -> Action {
    // Handle modals first
    if state.modal.is_some() {
        return handle_modal_key(state, key);
    }

    // Global keybindings (work in all modes)
    if let Some(action) = handle_global_key(state, key) {
        return action;
    }

    // Mode-specific keybindings
    match state.active_mode {
        Mode::Explore => handle_explore_key(state, key),
        Mode::Commit => handle_commit_key(state, key),
        Mode::Review => handle_review_key(state, key),
        Mode::PR => handle_pr_key(state, key),
        Mode::Changelog => handle_changelog_key(state, key),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Modal Key Handling
// ═══════════════════════════════════════════════════════════════════════════════

fn handle_modal_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match &state.modal {
        Some(Modal::Help) => {
            // Any key closes help
            state.close_modal();
            Action::Redraw
        }
        Some(Modal::Search { .. }) => {
            match key.code {
                KeyCode::Esc => {
                    state.close_modal();
                    Action::Redraw
                }
                KeyCode::Enter => {
                    // TODO: Handle search selection
                    state.close_modal();
                    Action::Redraw
                }
                KeyCode::Char(c) => {
                    // TODO: Update search query
                    let _ = c;
                    Action::Redraw
                }
                KeyCode::Backspace => {
                    // TODO: Update search query
                    Action::Redraw
                }
                _ => Action::None,
            }
        }
        Some(Modal::Confirm { .. }) => {
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    // TODO: Execute confirmed action
                    state.close_modal();
                    Action::Redraw
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    state.close_modal();
                    Action::Redraw
                }
                _ => Action::None,
            }
        }
        Some(Modal::Instructions { input }) => {
            match key.code {
                KeyCode::Esc => {
                    state.close_modal();
                    Action::Redraw
                }
                KeyCode::Enter => {
                    // Generate commit with instructions
                    let instructions = if input.is_empty() {
                        None
                    } else {
                        Some(input.clone())
                    };
                    state.close_modal();
                    state.set_iris_thinking("Generating commit message...");
                    Action::IrisQuery(IrisQueryRequest::GenerateCommit { instructions })
                }
                KeyCode::Char(c) => {
                    // Update input - need to get mutable reference
                    if let Some(Modal::Instructions { input }) = &mut state.modal {
                        input.push(c);
                    }
                    state.mark_dirty();
                    Action::Redraw
                }
                KeyCode::Backspace => {
                    if let Some(Modal::Instructions { input }) = &mut state.modal {
                        input.pop();
                    }
                    state.mark_dirty();
                    Action::Redraw
                }
                _ => Action::None,
            }
        }
        Some(Modal::Chat(chat_state)) => {
            match key.code {
                KeyCode::Esc => {
                    state.close_modal();
                    Action::Redraw
                }
                KeyCode::Enter => {
                    // Send message if not empty and not already responding
                    if !chat_state.input.is_empty() && !chat_state.is_responding {
                        let message = chat_state.input.clone();
                        if let Some(Modal::Chat(chat)) = &mut state.modal {
                            chat.add_user_message(&message);
                            chat.is_responding = true;
                        }
                        state.mark_dirty();
                        Action::IrisQuery(IrisQueryRequest::Chat { message })
                    } else {
                        Action::None
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(Modal::Chat(chat)) = &mut state.modal {
                        chat.input.push(c);
                    }
                    state.mark_dirty();
                    Action::Redraw
                }
                KeyCode::Backspace => {
                    if let Some(Modal::Chat(chat)) = &mut state.modal {
                        chat.input.pop();
                    }
                    state.mark_dirty();
                    Action::Redraw
                }
                KeyCode::Up => {
                    // Scroll up in chat history
                    if let Some(Modal::Chat(chat)) = &mut state.modal {
                        chat.scroll_offset = chat.scroll_offset.saturating_add(1);
                    }
                    state.mark_dirty();
                    Action::Redraw
                }
                KeyCode::Down => {
                    // Scroll down in chat history
                    if let Some(Modal::Chat(chat)) = &mut state.modal {
                        chat.scroll_offset = chat.scroll_offset.saturating_sub(1);
                    }
                    state.mark_dirty();
                    Action::Redraw
                }
                _ => Action::None,
            }
        }
        None => Action::None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global Key Handling
// ═══════════════════════════════════════════════════════════════════════════════

fn handle_global_key(state: &mut StudioState, key: KeyEvent) -> Option<Action> {
    match key.code {
        // Quit
        KeyCode::Char('q') if !is_editing(state) => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),

        // Help
        KeyCode::Char('?') if !is_editing(state) => {
            state.show_help();
            Some(Action::Redraw)
        }

        // Chat with Iris
        KeyCode::Char('/') if !is_editing(state) => {
            state.show_chat();
            Some(Action::Redraw)
        }

        // Mode switching
        KeyCode::Char('E') if key.modifiers.contains(KeyModifiers::SHIFT) || !is_editing(state) => {
            state.switch_mode(Mode::Explore);
            Some(Action::Redraw)
        }
        KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            state.switch_mode(Mode::Commit);
            Some(Action::Redraw)
        }
        KeyCode::Char('R') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            state.switch_mode(Mode::Review);
            Some(Action::Redraw)
        }
        KeyCode::Char('P') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            state.switch_mode(Mode::PR);
            Some(Action::Redraw)
        }
        KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            state.switch_mode(Mode::Changelog);
            Some(Action::Redraw)
        }

        // Panel navigation
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.focus_prev_panel();
            } else {
                state.focus_next_panel();
            }
            Some(Action::Redraw)
        }

        // Search (global)
        KeyCode::Char('/') if !is_editing(state) => {
            state.modal = Some(Modal::Search {
                query: String::new(),
                results: Vec::new(),
            });
            Some(Action::Redraw)
        }

        // Escape closes modals or cancels current operation
        KeyCode::Esc => {
            if state.modal.is_some() {
                state.close_modal();
                Some(Action::Redraw)
            } else {
                // Mode-specific escape handling
                None
            }
        }

        _ => None,
    }
}

/// Check if we're in an editing state (text input mode)
fn is_editing(state: &StudioState) -> bool {
    match state.active_mode {
        Mode::Commit => state.modes.commit.editing_message,
        _ => false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Explore Mode Key Handling
// ═══════════════════════════════════════════════════════════════════════════════

fn handle_explore_key(state: &mut StudioState, key: KeyEvent) -> Action {
    use super::state::PanelId;

    match state.focused_panel {
        PanelId::Left => handle_explore_file_tree_key(state, key),
        PanelId::Center => handle_explore_code_view_key(state, key),
        PanelId::Right => handle_explore_context_key(state, key),
    }
}

fn handle_explore_file_tree_key(state: &mut StudioState, key: KeyEvent) -> Action {
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

fn handle_explore_code_view_key(state: &mut StudioState, key: KeyEvent) -> Action {
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

fn handle_explore_context_key(state: &mut StudioState, key: KeyEvent) -> Action {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Commit Mode Key Handling
// ═══════════════════════════════════════════════════════════════════════════════

fn handle_commit_key(state: &mut StudioState, key: KeyEvent) -> Action {
    use super::state::PanelId;

    // If editing message, handle text input
    if state.modes.commit.editing_message {
        return handle_commit_editing_key(state, key);
    }

    match state.focused_panel {
        PanelId::Left => handle_commit_files_key(state, key),
        PanelId::Center => handle_commit_message_key(state, key),
        PanelId::Right => handle_commit_diff_key(state, key),
    }
}

fn handle_commit_editing_key(state: &mut StudioState, key: KeyEvent) -> Action {
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
fn sync_commit_file_selection(state: &mut StudioState) {
    if let Some(path) = state.modes.commit.file_tree.selected_path() {
        state.modes.commit.diff_view.select_file_by_path(&path);
    }
}

fn handle_commit_files_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.commit.file_tree.select_next();
            sync_commit_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.commit.file_tree.select_prev();
            sync_commit_file_selection(state);
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
            sync_commit_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('G') => {
            state.modes.commit.file_tree.select_last();
            sync_commit_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.file_tree.page_down(10);
            sync_commit_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.commit.file_tree.page_up(10);
            sync_commit_file_selection(state);
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
                    sync_commit_file_selection(state);
                    state.focused_panel = super::state::PanelId::Right;
                }
            }
            state.mark_dirty();
            Action::Redraw
        }

        _ => Action::None,
    }
}

fn handle_commit_diff_key(state: &mut StudioState, key: KeyEvent) -> Action {
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

fn handle_commit_message_key(state: &mut StudioState, key: KeyEvent) -> Action {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Placeholder Mode Handlers
// ═══════════════════════════════════════════════════════════════════════════════

fn handle_review_key(_state: &mut StudioState, _key: KeyEvent) -> Action {
    Action::None
}

fn handle_pr_key(_state: &mut StudioState, _key: KeyEvent) -> Action {
    Action::None
}

fn handle_changelog_key(_state: &mut StudioState, _key: KeyEvent) -> Action {
    Action::None
}

// ═══════════════════════════════════════════════════════════════════════════════
// Keybinding Descriptions
// ═══════════════════════════════════════════════════════════════════════════════

/// Get keybinding descriptions for help display
#[allow(dead_code)] // Will be used for dynamic help overlay
pub fn get_keybindings(mode: Mode) -> Vec<(&'static str, &'static str)> {
    let mut bindings = vec![
        // Global
        ("q", "Quit"),
        ("?", "Help"),
        ("Tab", "Next panel"),
        ("S-Tab", "Previous panel"),
        ("/", "Search"),
        ("E", "Explore mode"),
        ("C", "Commit mode"),
    ];

    // Mode-specific
    match mode {
        Mode::Explore => {
            bindings.extend([
                ("j/k", "Navigate up/down"),
                ("h/l", "Collapse/expand"),
                ("g/G", "First/last"),
                ("Enter", "Open/select"),
                ("w", "Ask why"),
                ("H", "Toggle heat map"),
                ("o", "Open in editor"),
            ]);
        }
        Mode::Commit => {
            bindings.extend([
                ("j/k", "Navigate/scroll"),
                ("h/l", "Collapse/expand"),
                ("[/]", "Prev/next hunk"),
                ("n/p", "Cycle messages"),
                ("s", "Stage file"),
                ("U", "Unstage file"),
                ("a", "Stage all"),
                ("e", "Edit message"),
                ("r", "Regenerate"),
                ("R", "Reset message"),
                ("Enter", "Commit/select"),
            ]);
        }
        _ => {}
    }

    bindings
}
