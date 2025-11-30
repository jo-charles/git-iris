//! Agent response event handlers
//!
//! Handles events from the agent: progress updates, completions, errors, streaming.

use super::super::events::{AgentResult, EventSource, TaskType};
use super::super::history::{ChatRole, ContentData, History};
use super::super::state::{Mode, Notification, StudioState};
use crate::studio::events::ContentType;

/// Handle `AgentStarted` event
pub fn agent_started(state: &mut StudioState, history: &mut History, task_type: TaskType) {
    state.set_iris_thinking(format!("Working on {}...", task_type));
    history.record_agent_start(task_type);
}

/// Handle `AgentProgress` event
pub fn agent_progress(state: &mut StudioState, tool_name: &str, message: &str) {
    state.set_iris_thinking(format!("{tool_name}: {message}"));
}

/// Handle `AgentComplete` event
pub fn agent_complete(
    state: &mut StudioState,
    history: &mut History,
    task_type: TaskType,
    result: AgentResult,
) {
    // Only set fallback completion if not already set (agent may have set a better one)
    if !state.iris_status.is_complete() {
        let completion_msg = match task_type {
            TaskType::Commit => "Ready.",
            TaskType::Review => "Review ready.",
            TaskType::PR => "PR ready.",
            TaskType::Changelog => "Changelog ready.",
            TaskType::ReleaseNotes => "Release notes ready.",
            TaskType::Chat => "Done.",
            TaskType::SemanticBlame => "Blame ready.",
        };
        state.set_iris_complete(completion_msg);
    }
    history.record_agent_complete(task_type, true);

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
                    EventSource::Agent,
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
                EventSource::Agent,
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
                EventSource::Agent,
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
                EventSource::Agent,
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
                EventSource::Agent,
                "generation_complete",
            );
        }

        AgentResult::ChatResponse(response) => {
            // Add Iris response to history
            history.add_chat_message(ChatRole::Iris, &response);

            // Update chat state
            state.chat_state.add_iris_response(&response);
        }

        AgentResult::SemanticBlame(result) => {
            state.modes.explore.semantic_blame = Some(result);
            state.modes.explore.blame_loading = false;
            state.notify(Notification::success("Blame analysis complete"));
        }
    }

    state.mark_dirty();
}

/// Handle `AgentError` event
pub fn agent_error(
    state: &mut StudioState,
    history: &mut History,
    task_type: TaskType,
    error: &str,
) {
    state.set_iris_error(error);
    history.record_agent_complete(task_type, false);

    // Reset generating flags
    match task_type {
        TaskType::Commit => state.modes.commit.generating = false,
        TaskType::Review => state.modes.review.generating = false,
        TaskType::PR => state.modes.pr.generating = false,
        TaskType::Changelog => state.modes.changelog.generating = false,
        TaskType::ReleaseNotes => state.modes.release_notes.generating = false,
        TaskType::Chat => {
            state.chat_state.is_responding = false;
        }
        TaskType::SemanticBlame => {
            state.modes.explore.blame_loading = false;
        }
    }

    state.notify(Notification::error(format!(
        "{} failed: {}",
        task_type, error
    )));
}

/// Handle `StreamingChunk` event
pub fn streaming_chunk(state: &mut StudioState, task_type: TaskType, aggregated: String) {
    match task_type {
        TaskType::Review => {
            state.modes.review.streaming_content = Some(aggregated);
        }
        TaskType::PR => {
            state.modes.pr.streaming_content = Some(aggregated);
        }
        TaskType::Changelog => {
            state.modes.changelog.streaming_content = Some(aggregated);
        }
        TaskType::ReleaseNotes => {
            state.modes.release_notes.streaming_content = Some(aggregated);
        }
        TaskType::Chat => {
            state.chat_state.streaming_response = Some(aggregated);
        }
        TaskType::SemanticBlame => {
            state.modes.explore.streaming_blame = Some(aggregated);
        }
        TaskType::Commit => {
            // Commit doesn't stream (structured JSON)
        }
    }
    state.mark_dirty();
}

/// Handle `StreamingComplete` event
pub fn streaming_complete(state: &mut StudioState, task_type: TaskType) {
    match task_type {
        TaskType::Review => {
            state.modes.review.streaming_content = None;
        }
        TaskType::PR => {
            state.modes.pr.streaming_content = None;
        }
        TaskType::Changelog => {
            state.modes.changelog.streaming_content = None;
        }
        TaskType::ReleaseNotes => {
            state.modes.release_notes.streaming_content = None;
        }
        TaskType::Chat => {
            state.chat_state.streaming_response = None;
            // Move final current_tool to history before clearing
            if let Some(tool) = state.chat_state.current_tool.take() {
                state.chat_state.add_tool_to_history(tool);
            }
        }
        TaskType::SemanticBlame => {
            state.modes.explore.streaming_blame = None;
        }
        TaskType::Commit => {}
    }
    state.mark_dirty();
}
