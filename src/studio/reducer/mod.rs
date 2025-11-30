//! Pure reducer for Iris Studio
//!
//! All state transitions happen here. This function is pure:
//! - Takes current state + event
//! - Returns new state + side effects
//! - No I/O, no async, no side effects inside
//!
//! Side effects are returned for the app to execute after state update.

mod agent;
mod content;
mod git;
mod modal;
mod navigation;
mod settings;
mod ui;

use crossterm::event::MouseEventKind;

use super::events::{
    AgentTask, ChatContext, DataType, ModalType, ScrollDirection, SideEffect, StudioEvent, TaskType,
};
use super::history::{ChatRole, History};
use super::state::{EmojiMode, Modal, Mode, StudioState};

// ═══════════════════════════════════════════════════════════════════════════════
// Reducer Function
// ═══════════════════════════════════════════════════════════════════════════════

/// Reducer: (state, event) → effects
///
/// This is the single source of truth for all state transitions.
/// The app calls this function which mutates state and returns effects.
#[allow(clippy::cognitive_complexity)]
pub fn reduce(
    state: &mut StudioState,
    event: StudioEvent,
    history: &mut History,
) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    match event {
        // ─────────────────────────────────────────────────────────────────────────
        // User Input Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::KeyPressed(key) => {
            // Delegate to key handler, which returns events
            // For now, we'll handle key events directly here
            // This will be refactored when handlers return events
            let key_effects = reduce_key_event(state, key, history);
            effects.extend(key_effects);
        }

        StudioEvent::Mouse(mouse) => {
            let mouse_effects = reduce_mouse_event(state, mouse);
            effects.extend(mouse_effects);
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Navigation Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::SwitchMode(new_mode) => {
            let old_mode = state.active_mode;
            if old_mode != new_mode {
                history.record_mode_switch(old_mode, new_mode);
                state.switch_mode(new_mode);

                // Trigger data loading for the new mode
                match new_mode {
                    Mode::Commit => {
                        effects.push(SideEffect::LoadData {
                            data_type: DataType::CommitDiff,
                            from_ref: None,
                            to_ref: None,
                        });
                    }
                    Mode::Review => {
                        let from = state.modes.review.from_ref.clone();
                        let to = state.modes.review.to_ref.clone();
                        effects.push(SideEffect::LoadData {
                            data_type: DataType::ReviewDiff,
                            from_ref: Some(from),
                            to_ref: Some(to),
                        });
                    }
                    Mode::PR => {
                        let base = state.modes.pr.base_branch.clone();
                        let to = state.modes.pr.to_ref.clone();
                        effects.push(SideEffect::LoadData {
                            data_type: DataType::PRDiff,
                            from_ref: Some(base),
                            to_ref: Some(to),
                        });
                    }
                    Mode::Changelog => {
                        let from = state.modes.changelog.from_ref.clone();
                        let to = state.modes.changelog.to_ref.clone();
                        effects.push(SideEffect::LoadData {
                            data_type: DataType::ChangelogCommits,
                            from_ref: Some(from),
                            to_ref: Some(to),
                        });
                    }
                    Mode::ReleaseNotes => {
                        let from = state.modes.release_notes.from_ref.clone();
                        let to = state.modes.release_notes.to_ref.clone();
                        effects.push(SideEffect::LoadData {
                            data_type: DataType::ReleaseNotesCommits,
                            from_ref: Some(from),
                            to_ref: Some(to),
                        });
                    }
                    Mode::Explore => {
                        // Explore mode loads files on demand
                    }
                }
            }
        }

        StudioEvent::FocusPanel(panel) => {
            state.focused_panel = panel;
            state.mark_dirty();
        }

        StudioEvent::FocusNext => {
            state.focus_next_panel();
        }

        StudioEvent::FocusPrev => {
            state.focus_prev_panel();
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Content Generation Events (user-triggered)
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::GenerateCommit {
            instructions,
            preset,
            use_gitmoji,
            amend,
        } => {
            state.modes.commit.generating = true;
            let thinking_msg = if amend {
                "Generating amended commit message..."
            } else {
                "Generating commit message..."
            };
            state.set_iris_thinking(thinking_msg);
            history.record_agent_start(TaskType::Commit);

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::Commit {
                    instructions,
                    preset,
                    use_gitmoji,
                    amend,
                },
            });
        }

        StudioEvent::GenerateReview { from_ref, to_ref } => {
            state.modes.review.generating = true;
            state.set_iris_thinking("Reviewing code changes...");
            history.record_agent_start(TaskType::Review);

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::Review { from_ref, to_ref },
            });
        }

        StudioEvent::GeneratePR {
            base_branch,
            to_ref,
        } => {
            state.modes.pr.generating = true;
            state.set_iris_thinking("Generating PR description...");
            history.record_agent_start(TaskType::PR);

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::PR {
                    base_branch,
                    to_ref,
                },
            });
        }

        StudioEvent::GenerateChangelog { from_ref, to_ref } => {
            state.modes.changelog.generating = true;
            state.set_iris_thinking("Generating changelog...");
            history.record_agent_start(TaskType::Changelog);

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::Changelog { from_ref, to_ref },
            });
        }

        StudioEvent::GenerateReleaseNotes { from_ref, to_ref } => {
            state.modes.release_notes.generating = true;
            state.set_iris_thinking("Generating release notes...");
            history.record_agent_start(TaskType::ReleaseNotes);

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::ReleaseNotes { from_ref, to_ref },
            });
        }

        StudioEvent::ChatMessage(message) => {
            // Add user message to history
            history.add_chat_message_with_context(
                ChatRole::User,
                &message,
                state.active_mode,
                get_current_content(state),
            );

            // Update chat state
            state.chat_state.add_user_message(&message);
            state.chat_state.is_responding = true;

            // Build context for agent
            let context = ChatContext {
                mode: state.active_mode,
                current_content: get_current_content(state),
                diff_summary: get_diff_summary(state),
            };

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::Chat { message, context },
            });
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Agent Response Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::AgentStarted { task_type } => {
            agent::agent_started(state, history, task_type);
        }

        StudioEvent::AgentProgress {
            task_type: _,
            tool_name,
            message,
        } => {
            agent::agent_progress(state, &tool_name, &message);
        }

        StudioEvent::AgentComplete { task_type, result } => {
            agent::agent_complete(state, history, task_type, result);
        }

        StudioEvent::AgentError { task_type, error } => {
            agent::agent_error(state, history, task_type, &error);
        }

        StudioEvent::StreamingChunk {
            task_type,
            chunk: _,
            aggregated,
        } => {
            agent::streaming_chunk(state, task_type, aggregated);
        }

        StudioEvent::StreamingComplete { task_type } => {
            agent::streaming_complete(state, task_type);
        }

        StudioEvent::StatusMessage(message) => {
            tracing::info!("Processing StatusMessage event: {:?}", message.message);
            state.add_status_message(message);
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Tool-Triggered Events (agent controls UI)
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::UpdateContent {
            content_type,
            content: payload,
        } => {
            content::update_content(state, history, content_type, payload);
        }

        StudioEvent::LoadData {
            data_type,
            from_ref,
            to_ref,
        } => {
            effects.push(SideEffect::LoadData {
                data_type,
                from_ref,
                to_ref,
            });
        }

        StudioEvent::StageFile(path) => {
            effects.push(content::stage_file(path));
        }

        StudioEvent::UnstageFile(path) => {
            effects.push(content::unstage_file(path));
        }

        // ─────────────────────────────────────────────────────────────────────────
        // File & Git Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::FileStaged(ref path) => {
            effects.extend(git::file_staged(state, path));
        }

        StudioEvent::FileUnstaged(ref path) => {
            effects.extend(git::file_unstaged(state, path));
        }

        StudioEvent::RefreshGitStatus => {
            effects.extend(git::refresh_git_status());
        }

        StudioEvent::GitStatusRefreshed => {
            git::git_status_refreshed(state);
        }

        StudioEvent::SelectFile(path) => {
            git::select_file(state, path);
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Modal Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::OpenModal(modal_type) => {
            state.modal = Some(modal::create_modal(state, modal_type));
            state.mark_dirty();
        }

        StudioEvent::CloseModal => {
            state.modal = None;
            state.mark_dirty();
        }

        StudioEvent::ModalConfirmed { modal_type, data } => {
            match modal_type {
                ModalType::ConfirmCommit => {
                    if let Some(msg) = state
                        .modes
                        .commit
                        .messages
                        .get(state.modes.commit.current_index)
                    {
                        let message = crate::types::format_commit_message(msg);
                        effects.push(SideEffect::ExecuteCommit { message });
                    }
                }
                ModalType::ConfirmAmend => {
                    if let Some(msg) = state
                        .modes
                        .commit
                        .messages
                        .get(state.modes.commit.current_index)
                    {
                        let message = crate::types::format_commit_message(msg);
                        effects.push(SideEffect::ExecuteAmend { message });
                    }
                }
                ModalType::RefSelector { field } => {
                    if let Some(ref_value) = data {
                        modal::apply_ref_selection(state, field, ref_value);
                    }
                }
                ModalType::PresetSelector => {
                    if let Some(preset) = data {
                        state.modes.commit.preset = preset;
                    }
                }
                ModalType::EmojiSelector => {
                    if let Some(emoji) = data {
                        if emoji == "none" {
                            state.modes.commit.emoji_mode = EmojiMode::None;
                            state.modes.commit.use_gitmoji = false;
                        } else if emoji == "auto" {
                            state.modes.commit.emoji_mode = EmojiMode::Auto;
                            state.modes.commit.use_gitmoji = true;
                        } else {
                            state.modes.commit.emoji_mode = EmojiMode::Custom(emoji);
                        }
                    }
                }
                _ => {}
            }
            state.modal = None;
            state.mark_dirty();
        }

        // ─────────────────────────────────────────────────────────────────────────
        // UI Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::Notify { level, message } => {
            ui::notify(state, level, message);
        }

        StudioEvent::Scroll { direction, amount } => {
            ui::scroll(state, direction, amount);
        }

        StudioEvent::ToggleEditMode => {
            ui::toggle_edit_mode(state);
        }

        StudioEvent::NextMessageVariant => {
            ui::next_message_variant(state);
        }

        StudioEvent::PrevMessageVariant => {
            ui::prev_message_variant(state);
        }

        StudioEvent::CopyToClipboard(content) => {
            effects.push(ui::copy_to_clipboard(content));
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Settings Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::SetPreset(preset) => {
            settings::set_preset(state, preset);
        }

        StudioEvent::ToggleGitmoji => {
            settings::toggle_gitmoji(state);
        }

        StudioEvent::SetEmoji(emoji) => {
            settings::set_emoji(state, emoji);
        }

        StudioEvent::ToggleAmendMode => {
            settings::toggle_amend_mode(state);
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Lifecycle Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::Quit => {
            effects.push(SideEffect::Quit);
        }

        StudioEvent::Tick => {
            state.tick();
        }
    }

    effects
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Get current content for the active mode (for chat context)
fn get_current_content(state: &StudioState) -> Option<String> {
    match state.active_mode {
        Mode::Commit => state
            .modes
            .commit
            .messages
            .get(state.modes.commit.current_index)
            .map(crate::types::format_commit_message),
        Mode::Review => {
            if state.modes.review.review_content.is_empty() {
                None
            } else {
                Some(state.modes.review.review_content.clone())
            }
        }
        Mode::PR => {
            if state.modes.pr.pr_content.is_empty() {
                None
            } else {
                Some(state.modes.pr.pr_content.clone())
            }
        }
        Mode::Changelog => {
            if state.modes.changelog.changelog_content.is_empty() {
                None
            } else {
                Some(state.modes.changelog.changelog_content.clone())
            }
        }
        Mode::ReleaseNotes => {
            if state.modes.release_notes.release_notes_content.is_empty() {
                None
            } else {
                Some(state.modes.release_notes.release_notes_content.clone())
            }
        }
        Mode::Explore => state
            .modes
            .explore
            .current_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
    }
}

/// Get a summary of the current diff/changes for chat context
fn get_diff_summary(state: &StudioState) -> Option<String> {
    let git = &state.git_status;

    // Build a summary of changes
    let mut parts = Vec::new();

    if git.staged_count > 0 {
        parts.push(format!(
            "{} staged file{}",
            git.staged_count,
            if git.staged_count == 1 { "" } else { "s" }
        ));
    }

    if git.modified_count > 0 {
        parts.push(format!(
            "{} modified file{}",
            git.modified_count,
            if git.modified_count == 1 { "" } else { "s" }
        ));
    }

    if git.untracked_count > 0 {
        parts.push(format!(
            "{} untracked file{}",
            git.untracked_count,
            if git.untracked_count == 1 { "" } else { "s" }
        ));
    }

    if parts.is_empty() {
        None
    } else {
        // Include file names for context
        let mut summary = format!("Changes: {}", parts.join(", "));

        if !git.staged_files.is_empty() {
            let files: Vec<_> = git
                .staged_files
                .iter()
                .take(5)
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect();
            summary.push_str(&format!("\nStaged: {}", files.join(", ")));
            if git.staged_files.len() > 5 {
                summary.push_str(&format!(" (+{} more)", git.staged_files.len() - 5));
            }
        }

        Some(summary)
    }
}

/// Handle key events - delegates to handlers which return effects directly
fn reduce_key_event(
    state: &mut StudioState,
    key: crossterm::event::KeyEvent,
    _history: &mut History,
) -> Vec<SideEffect> {
    use super::handlers::handle_key_event;

    // Handlers now return Vec<SideEffect> directly - no conversion needed!
    handle_key_event(state, key)
}

/// Handle mouse events
fn reduce_mouse_event(
    state: &mut StudioState,
    mouse: crossterm::event::MouseEvent,
) -> Vec<SideEffect> {
    let effects = Vec::new();

    // Check if chat modal is open - scroll it instead of the main view
    if matches!(state.modal, Some(Modal::Chat)) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                state.chat_state.scroll_up(3);
                state.mark_dirty();
            }
            MouseEventKind::ScrollDown => {
                let max_scroll = state.chat_state.estimated_max_scroll();
                state.chat_state.scroll_down(3, max_scroll);
                state.mark_dirty();
            }
            MouseEventKind::Down(_) => {
                state.mark_dirty();
            }
            _ => {}
        }
        return effects;
    }

    // Normal scroll handling for main views
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            navigation::apply_scroll(state, ScrollDirection::Up, 3);
        }
        MouseEventKind::ScrollDown => {
            navigation::apply_scroll(state, ScrollDirection::Down, 3);
        }
        MouseEventKind::Down(_) => {
            // Click handling - would need terminal position context
            // For now, just mark dirty
            state.mark_dirty();
        }
        _ => {}
    }

    effects
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::studio::events::NotificationLevel;
    use crate::studio::state::PanelId;

    fn test_state() -> StudioState {
        StudioState::new(Config::default(), None)
    }

    #[test]
    fn test_mode_switch() {
        let mut state = test_state();
        let mut history = History::new();

        let effects = reduce(
            &mut state,
            StudioEvent::SwitchMode(Mode::Commit),
            &mut history,
        );

        assert_eq!(state.active_mode, Mode::Commit);
        assert!(!effects.is_empty()); // Should have LoadData effect
    }

    #[test]
    fn test_focus_panel() {
        let mut state = test_state();
        let mut history = History::new();

        let _ = reduce(
            &mut state,
            StudioEvent::FocusPanel(PanelId::Right),
            &mut history,
        );

        assert_eq!(state.focused_panel, PanelId::Right);
    }

    #[test]
    fn test_notify() {
        let mut state = test_state();
        let mut history = History::new();

        let _ = reduce(
            &mut state,
            StudioEvent::Notify {
                level: NotificationLevel::Success,
                message: "Test notification".to_string(),
            },
            &mut history,
        );

        assert!(!state.notifications.is_empty());
    }

    #[test]
    fn test_quit_produces_effect() {
        let mut state = test_state();
        let mut history = History::new();

        let effects = reduce(&mut state, StudioEvent::Quit, &mut history);

        assert!(effects.iter().any(|e| matches!(e, SideEffect::Quit)));
    }
}
