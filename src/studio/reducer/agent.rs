//! Agent-related reducer functions
//!
//! Handles: `AgentStarted`, `AgentProgress`, `AgentComplete`, `AgentError`, `StreamingChunk`, `StreamingComplete`

use super::super::events::{AgentResult, ContentType, SideEffect, StudioEvent, TaskType};
use super::super::history::{ChatRole, ContentData, History};
use super::super::state::{Mode, Notification, StudioState};

/// Reduce agent-related events
pub fn reduce(
    state: &mut StudioState,
    event: StudioEvent,
    history: &mut History,
) -> Vec<SideEffect> {
    let effects = Vec::new();

    match event {
        StudioEvent::AgentStarted { task_type } => {
            state.set_iris_thinking(format!("Working on {}...", task_type));
            history.record_agent_start(task_type);
        }

        StudioEvent::AgentProgress {
            task_type: _,
            tool_name,
            message,
        } => {
            state.set_iris_thinking(format!("{}: {}", tool_name, message));
        }

        StudioEvent::AgentComplete { task_type, result } => {
            reduce_agent_complete(state, history, task_type, result);
            state.mark_dirty();
        }

        StudioEvent::AgentError { task_type, error } => {
            reduce_agent_error(state, history, task_type, error);
        }

        StudioEvent::StreamingChunk {
            task_type,
            chunk: _,
            aggregated,
        } => {
            reduce_streaming_chunk(state, task_type, aggregated);
            state.mark_dirty();
        }

        StudioEvent::StreamingComplete { task_type } => {
            reduce_streaming_complete(state, task_type);
            state.mark_dirty();
        }

        _ => {}
    }

    effects
}

fn reduce_agent_complete(
    state: &mut StudioState,
    history: &mut History,
    task_type: TaskType,
    result: AgentResult,
) {
    state.set_iris_idle();
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

            if let Some(msg) = messages.first() {
                history.record_content(
                    Mode::Commit,
                    ContentType::CommitMessage,
                    &ContentData::Commit(msg.clone()),
                    super::super::events::EventSource::Agent,
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
                super::super::events::EventSource::Agent,
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
                super::super::events::EventSource::Agent,
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
                super::super::events::EventSource::Agent,
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
                super::super::events::EventSource::Agent,
                "generation_complete",
            );
        }

        AgentResult::ChatResponse(response) => {
            history.add_chat_message(ChatRole::Iris, &response);
            state.chat_state.add_iris_response(&response);
        }

        AgentResult::SemanticBlame(result) => {
            state.modes.explore.semantic_blame = Some(result);
            state.modes.explore.blame_loading = false;
            state.notify(Notification::success("Blame analysis complete"));
        }
    }
}

fn reduce_agent_error(
    state: &mut StudioState,
    history: &mut History,
    task_type: TaskType,
    error: String,
) {
    state.set_iris_error(&error);
    history.record_agent_complete(task_type.clone(), false);

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

fn reduce_streaming_chunk(state: &mut StudioState, task_type: TaskType, aggregated: String) {
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
        TaskType::Commit => {}
    }
}

fn reduce_streaming_complete(state: &mut StudioState, task_type: TaskType) {
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
            if let Some(tool) = state.chat_state.current_tool.take() {
                state.chat_state.add_tool_to_history(tool);
            }
        }
        TaskType::SemanticBlame => {
            state.modes.explore.streaming_blame = None;
        }
        TaskType::Commit => {}
    }
}
