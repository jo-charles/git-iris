//! Event handlers for Iris Studio
//!
//! Keyboard input processing split by mode for maintainability.

mod changelog;
mod commit;
mod explore;
mod modal;
mod pr;
mod release_notes;
mod review;

use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::state::{Modal, Mode, Notification, StudioState};

pub use changelog::handle_changelog_key;
pub use commit::handle_commit_key;
pub use explore::handle_explore_key;
pub use modal::handle_modal_key;
pub use pr::handle_pr_key;
pub use release_notes::handle_release_notes_key;
pub use review::handle_review_key;

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
    /// Switch mode (triggers mode-specific data loading)
    SwitchMode(Mode),
    /// Reload PR data (after ref selection changes)
    ReloadPrData,
    /// Reload Review data (after ref selection changes)
    ReloadReviewData,
    /// Reload Changelog data (after ref selection changes)
    ReloadChangelogData,
    /// Reload Release Notes data (after ref selection changes)
    ReloadReleaseNotesData,
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
    GenerateCommit {
        instructions: Option<String>,
        preset: String,
        use_gitmoji: bool,
    },
    /// Generate code review
    GenerateReview,
    /// Generate PR description
    GeneratePR,
    /// Generate changelog between refs
    GenerateChangelog { from_ref: String, to_ref: String },
    /// Generate release notes between refs
    GenerateReleaseNotes { from_ref: String, to_ref: String },
    /// Chat with Iris
    Chat { message: String },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Event Handler
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
        Mode::ReleaseNotes => handle_release_notes_key(state, key),
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

        // Mode switching (Shift+letter)
        KeyCode::Char('E') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Action::SwitchMode(Mode::Explore))
        }
        KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Action::SwitchMode(Mode::Commit))
        }
        KeyCode::Char('R') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Action::SwitchMode(Mode::Review))
        }
        KeyCode::Char('P') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Action::SwitchMode(Mode::PR))
        }
        KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Action::SwitchMode(Mode::Changelog))
        }
        KeyCode::Char('N') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Action::SwitchMode(Mode::ReleaseNotes))
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
pub fn is_editing(state: &StudioState) -> bool {
    match state.active_mode {
        Mode::Commit => state.modes.commit.editing_message,
        _ => false,
    }
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

// ═══════════════════════════════════════════════════════════════════════════════
// Clipboard Utilities
// ═══════════════════════════════════════════════════════════════════════════════

/// Copy text to the system clipboard and notify the user
pub fn copy_to_clipboard(state: &mut StudioState, content: &str, description: &str) -> Action {
    match Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(content) {
            Ok(()) => {
                state.notify(Notification::success(format!(
                    "{description} copied to clipboard"
                )));
            }
            Err(e) => {
                state.notify(Notification::error(format!("Failed to copy: {e}")));
            }
        },
        Err(e) => {
            state.notify(Notification::error(format!("Clipboard unavailable: {e}")));
        }
    }
    state.mark_dirty();
    Action::Redraw
}
