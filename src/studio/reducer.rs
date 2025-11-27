//! Pure reducer for Iris Studio
//!
//! All state transitions happen here. This function is pure:
//! - Takes current state + event
//! - Returns new state + side effects
//! - No I/O, no async, no side effects inside
//!
//! Side effects are returned for the app to execute after state update.

use crossterm::event::MouseEventKind;

use super::events::{
    AgentResult, AgentTask, ChatContext, ContentPayload, ContentType, DataType, ModalType,
    NotificationLevel, RefField, ScrollDirection, SideEffect, StudioEvent, TaskType,
};
use super::history::{ChatRole, ContentData, History};
use super::state::{
    EmojiMode, Modal, Mode, Notification, PanelId, RefSelectorTarget, SettingsState, StudioState,
};

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
        } => {
            state.modes.commit.generating = true;
            state.set_iris_thinking("Generating commit message...");
            history.record_agent_start(TaskType::Commit);

            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::Commit {
                    instructions,
                    preset,
                    use_gitmoji,
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
                message.clone(),
                state.active_mode,
                get_current_content(state),
            );

            // Update chat state
            if let Some(Modal::Chat(ref mut chat)) = state.modal {
                chat.add_user_message(&message);
                chat.is_responding = true;
            }
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
            state.set_iris_thinking(format!("Working on {}...", task_type));
            history.record_agent_start(task_type);
        }

        StudioEvent::AgentProgress {
            task_type: _,
            tool_name,
            message,
        } => {
            // Update status with tool progress
            state.set_iris_thinking(format!("{}: {}", tool_name, message));
        }

        StudioEvent::AgentComplete { task_type, result } => {
            state.set_iris_idle();
            history.record_agent_complete(task_type.clone(), true);

            match result {
                AgentResult::CommitMessages(messages) => {
                    state.modes.commit.messages.clone_from(&messages);
                    state.modes.commit.current_index = 0;
                    state.modes.commit.generating = false;
                    state
                        .modes
                        .commit
                        .message_editor
                        .set_messages(messages.clone());

                    // Record in history
                    if let Some(msg) = messages.first() {
                        history.record_content(
                            Mode::Commit,
                            ContentType::CommitMessage,
                            &ContentData::Commit(msg.clone()),
                            super::events::EventSource::Agent,
                            "generation_complete",
                        );
                    }
                }

                AgentResult::ReviewContent(content) => {
                    state.modes.review.review_content.clone_from(&content);
                    state.modes.review.generating = false;

                    history.record_content(
                        Mode::Review,
                        ContentType::CodeReview,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Agent,
                        "generation_complete",
                    );
                }

                AgentResult::PRContent(content) => {
                    state.modes.pr.pr_content.clone_from(&content);
                    state.modes.pr.generating = false;

                    history.record_content(
                        Mode::PR,
                        ContentType::PRDescription,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Agent,
                        "generation_complete",
                    );
                }

                AgentResult::ChangelogContent(content) => {
                    state.modes.changelog.changelog_content.clone_from(&content);
                    state.modes.changelog.generating = false;

                    history.record_content(
                        Mode::Changelog,
                        ContentType::Changelog,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Agent,
                        "generation_complete",
                    );
                }

                AgentResult::ReleaseNotesContent(content) => {
                    state
                        .modes
                        .release_notes
                        .release_notes_content
                        .clone_from(&content);
                    state.modes.release_notes.generating = false;

                    history.record_content(
                        Mode::ReleaseNotes,
                        ContentType::ReleaseNotes,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Agent,
                        "generation_complete",
                    );
                }

                AgentResult::ChatResponse(response) => {
                    // Add Iris response to history
                    history.add_chat_message(ChatRole::Iris, response.clone());

                    // Update chat state
                    if let Some(Modal::Chat(ref mut chat)) = state.modal {
                        chat.add_iris_response(&response);
                    }
                    state.chat_state.add_iris_response(&response);
                }
            }

            state.mark_dirty();
        }

        StudioEvent::AgentError { task_type, error } => {
            state.set_iris_error(&error);
            history.record_agent_complete(task_type.clone(), false);

            // Reset generating flags
            match task_type {
                TaskType::Commit => state.modes.commit.generating = false,
                TaskType::Review => state.modes.review.generating = false,
                TaskType::PR => state.modes.pr.generating = false,
                TaskType::Changelog => state.modes.changelog.generating = false,
                TaskType::ReleaseNotes => state.modes.release_notes.generating = false,
                TaskType::Chat => {
                    if let Some(Modal::Chat(ref mut chat)) = state.modal {
                        chat.is_responding = false;
                    }
                    state.chat_state.is_responding = false;
                }
            }

            state.notify(Notification::error(format!(
                "{} failed: {}",
                task_type, error
            )));
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Tool-Triggered Events (agent controls UI)
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::UpdateContent {
            content_type,
            content,
        } => {
            match (content_type, content) {
                (ContentType::CommitMessage, ContentPayload::Commit(msg)) => {
                    // Update current message
                    if state.modes.commit.messages.is_empty() {
                        state.modes.commit.messages = vec![msg.clone()];
                        state
                            .modes
                            .commit
                            .message_editor
                            .set_messages(vec![msg.clone()]);
                    } else {
                        let idx = state.modes.commit.current_index;
                        state.modes.commit.messages[idx] = msg.clone();
                        state
                            .modes
                            .commit
                            .message_editor
                            .set_messages(state.modes.commit.messages.clone());
                    }

                    history.record_content(
                        Mode::Commit,
                        content_type,
                        &ContentData::Commit(msg),
                        super::events::EventSource::Tool,
                        "tool_update",
                    );
                }

                (ContentType::PRDescription, ContentPayload::Markdown(content)) => {
                    state.modes.pr.pr_content.clone_from(&content);

                    history.record_content(
                        Mode::PR,
                        content_type,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Tool,
                        "tool_update",
                    );
                }

                (ContentType::CodeReview, ContentPayload::Markdown(content)) => {
                    state.modes.review.review_content.clone_from(&content);

                    history.record_content(
                        Mode::Review,
                        content_type,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Tool,
                        "tool_update",
                    );
                }

                (ContentType::Changelog, ContentPayload::Markdown(content)) => {
                    state.modes.changelog.changelog_content.clone_from(&content);

                    history.record_content(
                        Mode::Changelog,
                        content_type,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Tool,
                        "tool_update",
                    );
                }

                (ContentType::ReleaseNotes, ContentPayload::Markdown(content)) => {
                    state
                        .modes
                        .release_notes
                        .release_notes_content
                        .clone_from(&content);

                    history.record_content(
                        Mode::ReleaseNotes,
                        content_type,
                        &ContentData::Markdown(content),
                        super::events::EventSource::Tool,
                        "tool_update",
                    );
                }

                _ => {
                    // Mismatched content type and payload
                    state.notify(Notification::warning("Received mismatched content update"));
                }
            }

            state.mark_dirty();
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
            effects.push(SideEffect::GitStage(path));
        }

        StudioEvent::UnstageFile(path) => {
            effects.push(SideEffect::GitUnstage(path));
        }

        // ─────────────────────────────────────────────────────────────────────────
        // File & Git Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::FileStaged(path) => {
            state.notify(Notification::success(format!("Staged: {}", path.display())));
            effects.push(SideEffect::RefreshGitStatus);
        }

        StudioEvent::FileUnstaged(path) => {
            state.notify(Notification::info(format!("Unstaged: {}", path.display())));
            effects.push(SideEffect::RefreshGitStatus);
        }

        StudioEvent::RefreshGitStatus => {
            effects.push(SideEffect::RefreshGitStatus);
        }

        StudioEvent::GitStatusRefreshed => {
            state.mark_dirty();
        }

        StudioEvent::SelectFile(path) => {
            // Update selected file based on current mode
            match state.active_mode {
                Mode::Explore => {
                    state.modes.explore.current_file = Some(path);
                }
                Mode::Commit => {
                    // Find index of file in staged files
                    if let Some(idx) = state
                        .git_status
                        .staged_files
                        .iter()
                        .position(|f| f == &path)
                    {
                        state.modes.commit.selected_file_index = idx;
                    }
                }
                _ => {}
            }
            state.mark_dirty();
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Modal Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::OpenModal(modal_type) => {
            state.modal = Some(create_modal(state, modal_type));
            state.mark_dirty();
        }

        StudioEvent::CloseModal => {
            // Sync chat state before closing
            if let Some(Modal::Chat(chat)) = &state.modal {
                state.chat_state = chat.clone();
            }
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
                ModalType::RefSelector { field } => {
                    if let Some(ref_value) = data {
                        apply_ref_selection(state, field, ref_value);
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
            let notification = match level {
                NotificationLevel::Info => Notification::info(message),
                NotificationLevel::Success => Notification::success(message),
                NotificationLevel::Warning => Notification::warning(message),
                NotificationLevel::Error => Notification::error(message),
            };
            state.notify(notification);
        }

        StudioEvent::Scroll { direction, amount } => {
            apply_scroll(state, direction, amount);
        }

        StudioEvent::ToggleEditMode => {
            if state.active_mode == Mode::Commit {
                state.modes.commit.editing_message = !state.modes.commit.editing_message;
                if state.modes.commit.editing_message {
                    state.modes.commit.message_editor.enter_edit_mode();
                } else {
                    state.modes.commit.message_editor.exit_edit_mode();
                }
            }
            state.mark_dirty();
        }

        StudioEvent::NextMessageVariant => {
            let commit = &mut state.modes.commit;
            if !commit.messages.is_empty() {
                commit.current_index = (commit.current_index + 1) % commit.messages.len();
                // Use the editor's built-in navigation which syncs everything
                commit.message_editor.next_message();
            }
            state.mark_dirty();
        }

        StudioEvent::PrevMessageVariant => {
            let commit = &mut state.modes.commit;
            if !commit.messages.is_empty() {
                commit.current_index = if commit.current_index == 0 {
                    commit.messages.len() - 1
                } else {
                    commit.current_index - 1
                };
                // Use the editor's built-in navigation which syncs everything
                commit.message_editor.prev_message();
            }
            state.mark_dirty();
        }

        StudioEvent::CopyToClipboard(content) => {
            effects.push(SideEffect::CopyToClipboard(content));
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Settings Events
        // ─────────────────────────────────────────────────────────────────────────
        StudioEvent::SetPreset(preset) => {
            state.modes.commit.preset = preset;
            state.mark_dirty();
        }

        StudioEvent::ToggleGitmoji => {
            state.modes.commit.use_gitmoji = !state.modes.commit.use_gitmoji;
            state.modes.commit.emoji_mode = if state.modes.commit.use_gitmoji {
                EmojiMode::Auto
            } else {
                EmojiMode::None
            };
            state.mark_dirty();
        }

        StudioEvent::SetEmoji(emoji) => {
            state.modes.commit.emoji_mode = if emoji.is_empty() {
                EmojiMode::None
            } else {
                EmojiMode::Custom(emoji)
            };
            state.mark_dirty();
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

/// Create a modal from a modal type
fn create_modal(state: &StudioState, modal_type: ModalType) -> Modal {
    match modal_type {
        ModalType::Help => Modal::Help,
        ModalType::Chat => {
            // Use persistent chat state
            Modal::Chat(state.chat_state.clone())
        }
        ModalType::Settings => Modal::Settings(SettingsState::from_config(&state.config)),
        ModalType::PresetSelector => {
            let presets = state.get_commit_presets();
            Modal::PresetSelector {
                input: String::new(),
                presets,
                selected: 0,
                scroll: 0,
            }
        }
        ModalType::EmojiSelector => {
            let emojis = state.get_emoji_list();
            Modal::EmojiSelector {
                input: String::new(),
                emojis,
                selected: 0,
                scroll: 0,
            }
        }
        ModalType::RefSelector { field } => {
            let refs = state.get_branch_refs();
            let target = match field {
                RefField::From => match state.active_mode {
                    Mode::Review => RefSelectorTarget::ReviewFrom,
                    Mode::Changelog => RefSelectorTarget::ChangelogFrom,
                    Mode::ReleaseNotes => RefSelectorTarget::ReleaseNotesFrom,
                    _ => RefSelectorTarget::ReviewFrom,
                },
                RefField::To => match state.active_mode {
                    Mode::Review => RefSelectorTarget::ReviewTo,
                    Mode::Changelog => RefSelectorTarget::ChangelogTo,
                    Mode::ReleaseNotes => RefSelectorTarget::ReleaseNotesTo,
                    _ => RefSelectorTarget::ReviewTo,
                },
                RefField::Base => RefSelectorTarget::PrFrom,
            };
            Modal::RefSelector {
                input: String::new(),
                refs,
                selected: 0,
                target,
            }
        }
        ModalType::ConfirmCommit => {
            if let Some(msg) = state
                .modes
                .commit
                .messages
                .get(state.modes.commit.current_index)
            {
                Modal::Confirm {
                    message: format!("Commit with message:\n\n{}", msg.title),
                    action: "commit".to_string(),
                }
            } else {
                Modal::Confirm {
                    message: "No commit message to commit".to_string(),
                    action: "cancel".to_string(),
                }
            }
        }
        ModalType::ConfirmQuit => Modal::Confirm {
            message: "Quit Iris Studio?".to_string(),
            action: "quit".to_string(),
        },
    }
}

/// Apply ref selection to the appropriate mode
fn apply_ref_selection(state: &mut StudioState, field: RefField, value: String) {
    match (state.active_mode, field) {
        (Mode::Review, RefField::From) => {
            state.modes.review.from_ref = value;
        }
        (Mode::Review, RefField::To) => {
            state.modes.review.to_ref = value;
        }
        (Mode::PR, RefField::Base) => {
            state.modes.pr.base_branch = value;
        }
        (Mode::PR, RefField::To) => {
            state.modes.pr.to_ref = value;
        }
        (Mode::Changelog, RefField::From) => {
            state.modes.changelog.from_ref = value;
        }
        (Mode::Changelog, RefField::To) => {
            state.modes.changelog.to_ref = value;
        }
        (Mode::ReleaseNotes, RefField::From) => {
            state.modes.release_notes.from_ref = value;
        }
        (Mode::ReleaseNotes, RefField::To) => {
            state.modes.release_notes.to_ref = value;
        }
        _ => {}
    }
}

/// Apply scroll to the current focused panel
fn apply_scroll(state: &mut StudioState, direction: ScrollDirection, amount: usize) {
    match state.active_mode {
        Mode::Commit => match state.focused_panel {
            PanelId::Left => {
                // File tree navigation (uses selection, not scroll directly)
                match direction {
                    ScrollDirection::Up => {
                        for _ in 0..amount {
                            state.modes.commit.file_tree.select_prev();
                        }
                    }
                    ScrollDirection::Down => {
                        for _ in 0..amount {
                            state.modes.commit.file_tree.select_next();
                        }
                    }
                    ScrollDirection::PageUp => {
                        state.modes.commit.file_tree.page_up(amount);
                    }
                    ScrollDirection::PageDown => {
                        state.modes.commit.file_tree.page_down(amount);
                    }
                    ScrollDirection::Top => {
                        state.modes.commit.file_tree.select_first();
                    }
                    ScrollDirection::Bottom => {
                        state.modes.commit.file_tree.select_last();
                    }
                }
            }
            PanelId::Center => {
                // Diff view scroll
                match direction {
                    ScrollDirection::Up => {
                        state.modes.commit.diff_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.commit.diff_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
            PanelId::Right => {
                // Message editor - navigation handled by textarea
                // No direct scroll control needed
            }
        },
        Mode::Review => {
            if state.focused_panel == PanelId::Center {
                match direction {
                    ScrollDirection::Up => {
                        state.modes.review.review_scroll =
                            state.modes.review.review_scroll.saturating_sub(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.review.review_scroll += amount;
                    }
                    _ => {}
                }
            }
        }
        Mode::PR => {
            if state.focused_panel == PanelId::Center {
                match direction {
                    ScrollDirection::Up => {
                        state.modes.pr.pr_scroll = state.modes.pr.pr_scroll.saturating_sub(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.pr.pr_scroll += amount;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    state.mark_dirty();
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

    match mouse.kind {
        MouseEventKind::ScrollUp => {
            apply_scroll(state, ScrollDirection::Up, 3);
        }
        MouseEventKind::ScrollDown => {
            apply_scroll(state, ScrollDirection::Down, 3);
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
