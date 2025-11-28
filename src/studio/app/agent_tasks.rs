//! Agent task spawning for Iris Studio
//!
//! Contains all async task spawning functions for Iris agent operations.

use crate::types::GeneratedMessage;

use super::{ChatUpdateType, IrisTaskResult, StudioApp};
use crate::studio::events::{BlameInfo, SemanticBlameResult, TaskType};

impl StudioApp {
    // ═══════════════════════════════════════════════════════════════════════════════
    // Chat Query
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Spawn a task for chat query - uses Iris agent with chat capability
    pub(super) fn spawn_chat_query(
        &self,
        message: String,
        context: crate::studio::events::ChatContext,
    ) {
        use crate::agents::StructuredResponse;
        use crate::agents::status::IRIS_STATUS;
        use crate::agents::tools::{ContentUpdate, create_content_update_channel};
        use crate::studio::state::{ChatMessage, ChatRole};
        use tokio_util::sync::CancellationToken;

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::ChatResponse(
                "Agent service not available".to_string(),
            ));
            return;
        };

        // Create bounded content update channel for tool-based updates
        let (content_tx, mut content_rx) = create_content_update_channel();

        // Capture context before spawning async task
        let tx = self.iris_result_tx.clone();
        let tx_status = self.iris_result_tx.clone();
        let tx_updates = self.iris_result_tx.clone();
        let mode = context.mode;

        // Extract conversation history (convert VecDeque → Vec)
        let chat_history: Vec<ChatMessage> =
            self.state.chat_state.messages.iter().cloned().collect();

        // Use context content if provided, otherwise extract from state
        let current_content = context
            .current_content
            .or_else(|| self.get_current_content_for_chat());

        // Cancellation token to signal when the main task is done
        let cancel_token = CancellationToken::new();
        let cancel_status = cancel_token.clone();
        let cancel_updates = cancel_token.clone();

        // Spawn a status polling task (polls global state, so still uses interval)
        tokio::spawn(async move {
            use crate::agents::status::IrisPhase;
            let mut last_tool: Option<String> = None;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

            loop {
                tokio::select! {
                    () = cancel_status.cancelled() => break,
                    _ = interval.tick() => {
                        let status = IRIS_STATUS.get_current();

                        // Check if we're in a tool execution phase
                        if let IrisPhase::ToolExecution {
                            ref tool_name,
                            ref reason,
                        } = status.phase
                        {
                            // Only send if it's a new tool
                            if last_tool.as_ref() != Some(tool_name) {
                                let _ = tx_status.send(IrisTaskResult::ToolStatus {
                                    tool_name: tool_name.clone(),
                                    message: reason.clone(),
                                });
                                last_tool = Some(tool_name.clone());
                            }
                        }
                    }
                }
            }
        });

        // Spawn a task to listen for content updates from tools (uses select! for zero latency)
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    () = cancel_updates.cancelled() => break,
                    update = content_rx.recv() => {
                        let Some(update) = update else { break };
                        let chat_update = match update {
                            ContentUpdate::Commit {
                                emoji,
                                title,
                                message,
                            } => {
                                tracing::info!("Content update tool: commit - {}", title);
                                ChatUpdateType::CommitMessage(GeneratedMessage {
                                    emoji,
                                    title,
                                    message,
                                })
                            }
                            ContentUpdate::PR { content } => {
                                tracing::info!("Content update tool: PR");
                                ChatUpdateType::PRDescription(content)
                            }
                            ContentUpdate::Review { content } => {
                                tracing::info!("Content update tool: review");
                                ChatUpdateType::Review(content)
                            }
                        };
                        let _ = tx_updates.send(IrisTaskResult::ChatUpdate(chat_update));
                    }
                }
            }
        });

        tokio::spawn(async move {
            // Build comprehensive context (universal chat across all modes)
            let mode_context = format!(
                "Current Mode: {:?}\nYou are Iris, a helpful git assistant. You have access to all generated content across modes and can help with commit messages, PR descriptions, code reviews, changelogs, and release notes.",
                mode
            );

            // Build conversation history string
            let history_str = if chat_history.is_empty() {
                String::new()
            } else {
                let mut hist = String::from("\n## Conversation History\n");
                for msg in &chat_history {
                    match msg.role {
                        ChatRole::User => hist.push_str(&format!("User: {}\n", msg.content)),
                        ChatRole::Iris => hist.push_str(&format!("Iris: {}\n", msg.content)),
                    }
                }
                hist
            };

            // Build current content section
            let content_section = if let Some(content) = &current_content {
                format!("\n## Current Content\n```\n{}\n```\n", content)
            } else {
                String::new()
            };

            // Tool-based update instructions
            let update_instructions = r"
## Response Guidelines
- Be concise - don't repeat content the user already sees
- When updating content, briefly explain what you changed

## Content Update Tools
You have tools to update content. When the user asks you to modify, change, update, or rewrite content:

1. **update_commit** - Update the commit message (emoji, title, message)
2. **update_pr** - Update the PR description (content)
3. **update_review** - Update the code review (content)

Simply call the appropriate tool with the new content. Do NOT echo back the full content in your response - the tool will update it directly.";

            let prompt = format!(
                "{}{}{}{}\n\n## Current Request\nUser: {}",
                mode_context, content_section, history_str, update_instructions, message
            );

            // Execute with streaming and content update tools
            let streaming_tx = tx.clone();
            let on_chunk = move |chunk: &str, aggregated: &str| {
                let _ = streaming_tx.send(IrisTaskResult::StreamingChunk {
                    task_type: TaskType::Chat,
                    chunk: chunk.to_string(),
                    aggregated: aggregated.to_string(),
                });
            };

            match agent
                .execute_chat_streaming(&prompt, content_tx, on_chunk)
                .await
            {
                Ok(response) => {
                    // Signal streaming complete
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::Chat,
                    });

                    let text = match response {
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };

                    tracing::debug!("Chat response received, length: {}", text.len());
                    let _ = tx.send(IrisTaskResult::ChatResponse(text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::ChatResponse(format!(
                        "I encountered an error: {}",
                        e
                    )));
                }
            }

            // Signal that we're done so the helper tasks stop
            cancel_token.cancel();
        });
    }

    /// Get ALL generated content for chat context (universal across modes)
    pub(super) fn get_current_content_for_chat(&self) -> Option<String> {
        let mut sections = Vec::new();

        // Commit message
        let commit = &self.state.modes.commit;
        if let Some(msg) = commit.messages.get(commit.current_index) {
            let formatted = crate::types::format_commit_message(msg);
            if !formatted.trim().is_empty() {
                sections.push(format!("## Commit Message\n{}", formatted));
            }
        }

        // Code review
        let review = &self.state.modes.review.review_content;
        if !review.is_empty() {
            let preview = if review.len() > 500 {
                format!("{}...", &review[..500])
            } else {
                review.clone()
            };
            sections.push(format!("## Code Review\n{}", preview));
        }

        // PR description
        let pr = &self.state.modes.pr.pr_content;
        if !pr.is_empty() {
            let preview = if pr.len() > 500 {
                format!("{}...", &pr[..500])
            } else {
                pr.clone()
            };
            sections.push(format!("## PR Description\n{}", preview));
        }

        // Changelog
        let cl = &self.state.modes.changelog.changelog_content;
        if !cl.is_empty() {
            let preview = if cl.len() > 500 {
                format!("{}...", &cl[..500])
            } else {
                cl.clone()
            };
            sections.push(format!("## Changelog\n{}", preview));
        }

        // Release notes
        let rn = &self.state.modes.release_notes.release_notes_content;
        if !rn.is_empty() {
            let preview = if rn.len() > 500 {
                format!("{}...", &rn[..500])
            } else {
                rn.clone()
            };
            sections.push(format!("## Release Notes\n{}", preview));
        }

        if sections.is_empty() {
            None
        } else {
            Some(sections.join("\n\n"))
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Review Generation
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Spawn a task for code review generation with streaming
    pub(super) fn spawn_review_generation(&self, from_ref: String, to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::Review,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Use review context with specified refs
            let context = match TaskContext::for_review(None, Some(from_ref), Some(to_ref), false) {
                Ok(ctx) => ctx,
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::Review,
                        error: format!("Context error: {}", e),
                    });
                    return;
                }
            };

            // Execute with streaming - send chunks as they arrive
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::Review,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent
                .execute_task_streaming("review", context, on_chunk)
                .await
            {
                Ok(response) => {
                    // Send streaming complete
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::Review,
                    });

                    // Also send final content
                    let review_text = match response {
                        StructuredResponse::MarkdownReview(review) => review.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ReviewContent(review_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::Review,
                        error: format!("Review error: {}", e),
                    });
                }
            }
        });
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PR Generation
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Spawn a task for PR description generation with streaming
    pub(super) fn spawn_pr_generation(&self, base_branch: String, _to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::PR,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Build context for PR (comparing current branch to base)
            let context = TaskContext::for_pr(Some(base_branch), None);

            // Execute with streaming
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::PR,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent.execute_task_streaming("pr", context, on_chunk).await {
                Ok(response) => {
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::PR,
                    });

                    let pr_text = match response {
                        StructuredResponse::PullRequest(pr) => pr.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::PRContent(pr_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::PR,
                        error: format!("PR error: {}", e),
                    });
                }
            }
        });
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Changelog Generation
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Spawn a task for changelog generation with streaming
    pub(super) fn spawn_changelog_generation(&self, from_ref: String, to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::Changelog,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Build context for changelog (comparing two refs)
            let context = TaskContext::for_changelog(from_ref, Some(to_ref));

            // Execute with streaming
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::Changelog,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent
                .execute_task_streaming("changelog", context, on_chunk)
                .await
            {
                Ok(response) => {
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::Changelog,
                    });

                    let changelog_text = match response {
                        StructuredResponse::Changelog(cl) => cl.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ChangelogContent(changelog_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::Changelog,
                        error: format!("Changelog error: {}", e),
                    });
                }
            }
        });
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Release Notes Generation
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Spawn a task for release notes generation with streaming
    pub(super) fn spawn_release_notes_generation(&self, from_ref: String, to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::ReleaseNotes,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Build context for release notes (comparing two refs)
            let context = TaskContext::for_changelog(from_ref, Some(to_ref));

            // Execute with streaming
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::ReleaseNotes,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent
                .execute_task_streaming("release_notes", context, on_chunk)
                .await
            {
                Ok(response) => {
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::ReleaseNotes,
                    });

                    let release_notes_text = match response {
                        StructuredResponse::ReleaseNotes(rn) => rn.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ReleaseNotesContent(release_notes_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::ReleaseNotes,
                        error: format!("Release notes error: {}", e),
                    });
                }
            }
        });
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Commit Generation
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Spawn a task to generate a commit message
    pub(super) fn spawn_commit_generation(
        &self,
        instructions: Option<String>,
        preset: String,
        use_gitmoji: bool,
        amend: bool,
    ) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::Commit,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        // Get original message for amend mode
        let original_message = if amend {
            self.state
                .repo
                .as_ref()
                .and_then(|r| r.get_head_commit_message().ok())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let tx = self.iris_result_tx.clone();

        tokio::spawn(async move {
            // Use amend context if amending, otherwise standard commit context
            let context = if amend {
                TaskContext::for_amend(original_message)
            } else {
                TaskContext::for_gen()
            };

            // Execute commit capability with style overrides
            let preset_opt = if preset == "default" {
                None
            } else {
                Some(preset.as_str())
            };

            match agent
                .execute_task_with_style(
                    "commit",
                    context,
                    preset_opt,
                    Some(use_gitmoji),
                    instructions.as_deref(),
                )
                .await
            {
                Ok(response) => {
                    // Extract message from response
                    match response {
                        StructuredResponse::CommitMessage(msg) => {
                            let _ = tx.send(IrisTaskResult::CommitMessages(vec![msg]));
                        }
                        _ => {
                            let _ = tx.send(IrisTaskResult::Error {
                                task_type: TaskType::Commit,
                                error: "Unexpected response type from agent".to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::Commit,
                        error: format!("Agent error: {}", e),
                    });
                }
            }
        });
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Semantic Blame
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Gather blame information from git and spawn the semantic blame agent.
    /// All blocking I/O (file read, git blame) runs in a background task to avoid
    /// blocking the UI event loop.
    pub(super) fn gather_blame_and_spawn(
        &self,
        file: &std::path::Path,
        start_line: usize,
        end_line: usize,
    ) {
        use crate::agents::StructuredResponse;

        let Some(repo) = &self.state.repo else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::SemanticBlame,
                error: "Repository not available".to_string(),
            });
            return;
        };

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::SemanticBlame,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        // Clone values needed in the async task
        let tx = self.iris_result_tx.clone();
        let file = file.to_path_buf();
        let repo_path = repo.repo_path().clone();

        tokio::spawn(async move {
            // Run blocking I/O in spawn_blocking to avoid blocking the tokio runtime
            let blame_result = tokio::task::spawn_blocking(move || {
                use std::fs;
                use std::process::Command;

                // Read file content
                let content = fs::read_to_string(&file)?;
                let lines: Vec<&str> = content.lines().collect();

                if start_line == 0 || start_line > lines.len() {
                    return Err(anyhow::anyhow!("Invalid line range"));
                }

                let end = end_line.min(lines.len());
                let code_content = lines[(start_line - 1)..end].join("\n");

                // Run git blame
                let output = Command::new("git")
                    .args([
                        "-C",
                        &repo_path.to_string_lossy(),
                        "blame",
                        "-L",
                        &format!("{},{}", start_line, end_line),
                        "--porcelain",
                        &file.to_string_lossy(),
                    ])
                    .output()?;

                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow::anyhow!("Git blame failed: {}", err));
                }

                let blame_output = String::from_utf8_lossy(&output.stdout);
                let (commit_hash, author, commit_date, commit_message) =
                    parse_blame_porcelain(&blame_output);

                Ok(BlameInfo {
                    file,
                    start_line,
                    end_line,
                    commit_hash,
                    author,
                    commit_date,
                    commit_message,
                    code_content,
                })
            })
            .await;

            // Handle spawn_blocking result
            let blame_info = match blame_result {
                Ok(Ok(info)) => info,
                Ok(Err(e)) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::SemanticBlame,
                        error: e.to_string(),
                    });
                    return;
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::SemanticBlame,
                        error: format!("Task panicked: {}", e),
                    });
                    return;
                }
            };

            // Build context for agent
            let context_text = format!(
                "File: {}\nLines: {}-{}\nCommit: {} by {} on {}\nMessage: {}\n\nCode:\n{}",
                blame_info.file.display(),
                blame_info.start_line,
                blame_info.end_line,
                blame_info.commit_hash,
                blame_info.author,
                blame_info.commit_date,
                blame_info.commit_message,
                blame_info.code_content
            );

            // Execute semantic_blame capability
            match agent
                .execute_task_with_prompt("semantic_blame", &context_text)
                .await
            {
                Ok(response) => match response {
                    StructuredResponse::SemanticBlame(explanation) => {
                        let result = SemanticBlameResult {
                            file: blame_info.file,
                            start_line: blame_info.start_line,
                            end_line: blame_info.end_line,
                            commit_hash: blame_info.commit_hash,
                            author: blame_info.author,
                            commit_date: blame_info.commit_date,
                            commit_message: blame_info.commit_message,
                            explanation,
                        };
                        let _ = tx.send(IrisTaskResult::SemanticBlame(result));
                    }
                    _ => {
                        let _ = tx.send(IrisTaskResult::Error {
                            task_type: TaskType::SemanticBlame,
                            error: "Unexpected response type from agent".to_string(),
                        });
                    }
                },
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::SemanticBlame,
                        error: format!("Semantic blame error: {}", e),
                    });
                }
            }
        });
    }

    /// Spawn the semantic blame agent to explain why the code exists.
    /// Used when blame info is already collected (e.g., from `AgentTask::SemanticBlame`).
    pub(super) fn spawn_semantic_blame(&self, blame_info: BlameInfo) {
        use crate::agents::StructuredResponse;

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error {
                task_type: TaskType::SemanticBlame,
                error: "Agent service not available".to_string(),
            });
            return;
        };

        let tx = self.iris_result_tx.clone();

        tokio::spawn(async move {
            // Build context with blame info
            let context_text = format!(
                "File: {}\nLines: {}-{}\nCommit: {} by {} on {}\nMessage: {}\n\nCode:\n{}",
                blame_info.file.display(),
                blame_info.start_line,
                blame_info.end_line,
                blame_info.commit_hash,
                blame_info.author,
                blame_info.commit_date,
                blame_info.commit_message,
                blame_info.code_content
            );

            // Execute semantic_blame capability
            match agent
                .execute_task_with_prompt("semantic_blame", &context_text)
                .await
            {
                Ok(response) => match response {
                    StructuredResponse::SemanticBlame(explanation) => {
                        let result = SemanticBlameResult {
                            file: blame_info.file,
                            start_line: blame_info.start_line,
                            end_line: blame_info.end_line,
                            commit_hash: blame_info.commit_hash,
                            author: blame_info.author,
                            commit_date: blame_info.commit_date,
                            commit_message: blame_info.commit_message,
                            explanation,
                        };
                        let _ = tx.send(IrisTaskResult::SemanticBlame(result));
                    }
                    _ => {
                        let _ = tx.send(IrisTaskResult::Error {
                            task_type: TaskType::SemanticBlame,
                            error: "Unexpected response type from agent".to_string(),
                        });
                    }
                },
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error {
                        task_type: TaskType::SemanticBlame,
                        error: format!("Semantic blame error: {}", e),
                    });
                }
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse git blame porcelain output to extract commit info
fn parse_blame_porcelain(output: &str) -> (String, String, String, String) {
    let mut commit_hash = String::new();
    let mut author = String::new();
    let mut commit_time = String::new();
    let mut summary = String::new();

    for line in output.lines() {
        if commit_hash.is_empty()
            && line.len() >= 40
            && line.chars().take(40).all(|c| c.is_ascii_hexdigit())
        {
            commit_hash = line.split_whitespace().next().unwrap_or("").to_string();
        } else if let Some(rest) = line.strip_prefix("author ") {
            author = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("author-time ") {
            if let Ok(timestamp) = rest.parse::<i64>() {
                commit_time = chrono::DateTime::from_timestamp(timestamp, 0).map_or_else(
                    || "Unknown date".to_string(),
                    |dt| dt.format("%Y-%m-%d %H:%M").to_string(),
                );
            }
        } else if let Some(rest) = line.strip_prefix("summary ") {
            summary = rest.to_string();
        }
    }

    if commit_hash.is_empty() {
        commit_hash = "Unknown".to_string();
    }
    if author.is_empty() {
        author = "Unknown".to_string();
    }
    if commit_time.is_empty() {
        commit_time = "Unknown date".to_string();
    }

    (commit_hash, author, commit_time, summary)
}
