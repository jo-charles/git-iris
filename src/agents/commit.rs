use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    tools::AgentTool,
};
use crate::commit::types::GeneratedMessage;
use crate::common::get_combined_instructions;
use crate::context::{ChangeType, ProjectMetadata, RecentCommit, StagedFile};
use crate::gitmoji::get_gitmoji_list;
use crate::log_debug;

/// Comprehensive context for commit message generation
#[derive(Debug, Clone)]
pub struct FullCommitContext {
    pub branch: String,
    pub staged_files: Vec<StagedFile>,
    pub recent_commits: Vec<RecentCommit>,
    pub project_metadata: ProjectMetadata,
    pub file_analyses: Vec<serde_json::Value>,
    pub repository_status: serde_json::Value,
}

/// Parsed commit message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommitResponse {
    pub emoji: Option<String>,
    pub title: String,
    pub message: String,
}

/// Specialized agent for commit message generation and commit operations
pub struct CommitAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    backend: AgentBackend,
    tools: Vec<Arc<dyn AgentTool>>,
    initialized: bool,
}

impl CommitAgent {
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        let mut agent = Self {
            id: "commit_agent".to_string(),
            name: "Commit Agent".to_string(),
            description:
                "Specialized agent for generating commit messages and performing commit operations"
                    .to_string(),
            capabilities: vec![
                "commit_message_generation".to_string(),
                "diff_analysis".to_string(),
                "change_summarization".to_string(),
                "commit_validation".to_string(),
            ],
            backend,
            tools,
            initialized: false,
        };

        // Initialize the agent
        agent.initialized = true;
        agent
    }

    /// Generate a commit message based on repository changes
    async fn generate_commit_message(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        // Extract parameters
        let preset = params
            .get("preset")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        let custom_instructions = params
            .get("instructions")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let use_gitmoji = params
            .get("gitmoji")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(context.config.use_gitmoji);

        log_debug!(
            "🚀 CommitAgent starting commit message generation with preset: '{}', gitmoji: {}",
            preset,
            use_gitmoji
        );

        // Step 1: Gather complete context through multiple tool calls
        log_debug!("📊 Step 1: Gathering comprehensive context through multiple tool calls");
        let commit_context = self.gather_full_commit_context(context).await?;

        log_debug!(
            "✅ Context gathered: {} staged files, {} recent commits, branch: {}",
            commit_context.staged_files.len(),
            commit_context.recent_commits.len(),
            commit_context.branch
        );

        // Step 2: Build system prompt using the same logic as existing system
        log_debug!(
            "📝 Step 2: Building system prompt with instruction preset and gitmoji settings"
        );
        let system_prompt = self
            .create_system_prompt(&context.config, preset, custom_instructions, use_gitmoji)
            .await?;

        log_debug!("✅ System prompt built: {} characters", system_prompt.len());

        // Step 3: Build user prompt with all gathered context
        log_debug!("🔍 Step 3: Building user prompt with complete context");
        let user_prompt = self.create_user_prompt(&commit_context).await?;

        log_debug!("✅ User prompt built: {} characters", user_prompt.len());

        // Step 4: Generate commit message using real LLM
        log_debug!("🤖 Step 4: Generating commit message using LLM backend");
        let generated_message = self
            .generate_with_backend_full(&system_prompt, &user_prompt)
            .await?;

        log_debug!(
            "✅ LLM response received: {} characters",
            generated_message.len()
        );

        // Step 5: Parse and validate the response
        log_debug!("🔧 Step 5: Parsing and validating JSON response");
        let parsed_response = self.parse_commit_response(&generated_message).await?;

        log_debug!(
            "✅ Commit message parsed - Title: '{}', Emoji: {:?}",
            parsed_response.title,
            parsed_response.emoji
        );

        Ok(TaskResult::success_with_data(
            "Commit message generated successfully".to_string(),
            serde_json::json!({
                "commit_message": parsed_response.message,
                "title": parsed_response.title,
                "emoji": parsed_response.emoji,
                "files_changed": commit_context.staged_files.len(),
                "preset_used": preset,
            }),
        )
        .with_confidence(0.9))
    }

    /// Gather comprehensive context through multiple tool calls
    async fn gather_full_commit_context(
        &self,
        context: &AgentContext,
    ) -> Result<FullCommitContext> {
        log_debug!("🔍 Finding Git tool for repository operations");
        let git_tool = self
            .tools
            .iter()
            .find(|t| t.capabilities().contains(&"git".to_string()))
            .ok_or_else(|| anyhow::anyhow!("Git tool not found"))?;

        log_debug!("✅ Git tool found: {}", git_tool.name());

        // 1. Get staged files with diffs
        log_debug!("📄 Tool call 1/4: Getting staged files with diffs");
        let mut diff_params = HashMap::new();
        diff_params.insert(
            "operation".to_string(),
            serde_json::Value::String("diff".to_string()),
        );
        let diff_result = git_tool.execute(context, &diff_params).await?;
        log_debug!(
            "✅ Diff retrieved: {} bytes",
            diff_result
                .get("content")
                .and_then(|v| v.as_str())
                .map_or(0, str::len)
        );

        // 2. Get list of changed files
        log_debug!("📂 Tool call 2/4: Getting list of changed files");
        let mut files_params = HashMap::new();
        files_params.insert(
            "operation".to_string(),
            serde_json::Value::String("files".to_string()),
        );
        let files_result = git_tool.execute(context, &files_params).await?;
        let file_count = files_result
            .get("content")
            .and_then(|v| v.as_array())
            .map_or(0, std::vec::Vec::len);
        log_debug!("✅ File list retrieved: {} files", file_count);

        // 3. Get recent commit history
        log_debug!("📜 Tool call 3/4: Getting recent commit history");
        let mut log_params = HashMap::new();
        log_params.insert(
            "operation".to_string(),
            serde_json::Value::String("log".to_string()),
        );
        let log_result = git_tool.execute(context, &log_params).await?;
        log_debug!(
            "✅ Commit history retrieved: {} bytes",
            log_result
                .get("content")
                .and_then(|v| v.as_str())
                .map_or(0, str::len)
        );

        // 4. Get repository status
        log_debug!("📊 Tool call 4/4: Getting repository status");
        let mut status_params = HashMap::new();
        status_params.insert(
            "operation".to_string(),
            serde_json::Value::String("status".to_string()),
        );
        let status_result = git_tool.execute(context, &status_params).await?;
        log_debug!("✅ Repository status retrieved");

        // 5. Analyze each changed file for deeper context
        let changed_files: Vec<String> = files_result
            .get("content")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        log_debug!(
            "🔬 Starting detailed file analysis for {} files",
            changed_files.len()
        );
        let mut file_analyses = Vec::new();
        if let Some(file_analyzer) = self
            .tools
            .iter()
            .find(|t| t.capabilities().contains(&"file_analysis".to_string()))
        {
            log_debug!("✅ File analyzer tool found: {}", file_analyzer.name());
            let files_to_analyze = changed_files.iter().take(20).collect::<Vec<_>>();
            log_debug!(
                "📊 Analyzing {} files (limited to 20 for performance)",
                files_to_analyze.len()
            );

            for (index, file_path) in files_to_analyze.iter().enumerate() {
                log_debug!(
                    "🔍 Analyzing file {}/{}: {}",
                    index + 1,
                    files_to_analyze.len(),
                    file_path
                );
                let mut analysis_params = HashMap::new();
                analysis_params.insert(
                    "path".to_string(),
                    serde_json::Value::String((*file_path).to_string()),
                );
                analysis_params.insert(
                    "analysis_type".to_string(),
                    serde_json::Value::String("detailed".to_string()),
                );
                analysis_params
                    .insert("include_content".to_string(), serde_json::Value::Bool(true));

                if let Ok(analysis) = file_analyzer.execute(context, &analysis_params).await {
                    let file_type = analysis
                        .get("file_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let lines = analysis
                        .get("lines")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(0);
                    log_debug!(
                        "✅ File analyzed: {} (type: {}, lines: {})",
                        file_path,
                        file_type,
                        lines
                    );
                    file_analyses.push(analysis);
                } else {
                    log_debug!("⚠️  Failed to analyze file: {}", file_path);
                }
            }
        } else {
            log_debug!("⚠️  No file analyzer tool found, skipping detailed file analysis");
        }

        // 6. Get project metadata through file analysis
        log_debug!("📋 Gathering project metadata");
        let project_metadata = self.gather_project_metadata(context).await?;
        log_debug!("✅ Project metadata gathered");

        let branch = context
            .git_repo
            .get_current_branch()
            .unwrap_or_else(|_| "main".to_string());

        log_debug!("🔄 Parsing tool results into structured data");
        let staged_files = self.parse_staged_files(&diff_result, &files_result).await?;
        let recent_commits = self.parse_recent_commits(&log_result).await?;

        log_debug!(
            "📊 Context parsing complete - {} staged files, {} recent commits",
            staged_files.len(),
            recent_commits.len()
        );

        let context = FullCommitContext {
            branch,
            staged_files,
            recent_commits,
            project_metadata,
            file_analyses,
            repository_status: status_result,
        };

        log_debug!("🎯 Full commit context assembled successfully");
        Ok(context)
    }

    /// Build context for commit message generation
    async fn build_commit_context(
        &self,
        diff: &str,
        file_analyses: &[serde_json::Value],
        preset: &str,
        instructions: &str,
    ) -> Result<String> {
        let mut context = String::new();

        // Add preset-specific instructions
        context.push_str(&format!(
            "Generate a commit message using the '{preset}' preset.\n\n"
        ));

        if !instructions.is_empty() {
            context.push_str(&format!("Additional instructions: {instructions}\n\n"));
        }

        // Add file analysis summary
        if !file_analyses.is_empty() {
            context.push_str("Changed files analysis:\n");
            for analysis in file_analyses {
                if let (Some(path), Some(language)) = (
                    analysis.get("path").and_then(|v| v.as_str()),
                    analysis.get("language").and_then(|v| v.as_str()),
                ) {
                    context.push_str(&format!("- {path} ({language})\n"));
                }
            }
            context.push('\n');
        }

        // Add diff (truncated if too long)
        context.push_str("Changes:\n");
        if diff.len() > 4000 {
            context.push_str(&diff[..4000]);
            context.push_str("\n... (diff truncated for brevity)");
        } else {
            context.push_str(diff);
        }

        Ok(context)
    }

    /// Create a Rig agent configured for commit message generation
    async fn create_rig_agent(&self) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        use rig::client::CompletionClient;

        let preamble = r#"You are an expert Git commit message writer. Your role is to analyze code changes and generate clear, informative commit messages that follow best practices.

Guidelines:
1. Use conventional commit format when specified (type(scope): description)
2. Be concise but descriptive (summary under 72 characters)
3. Focus on the "what" and "why" of changes
4. Use imperative mood ("Add feature" not "Added feature")
5. Include scope when relevant (e.g., "feat(auth): add login validation")
6. For breaking changes, include "BREAKING CHANGE:" in body

Common types: feat, fix, docs, style, refactor, test, chore

Response format: Only return the commit message, nothing else."#;

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .temperature(0.3)
                    .build();
                Ok(Box::new(agent))
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .temperature(0.3)
                    .build();
                Ok(Box::new(agent))
            }
        }
    }

    /// Generate commit message using Rig agent
    async fn generate_with_rig_agent(
        &self,
        _rig_agent: &Box<dyn std::any::Any + Send + Sync>,
        context: &str,
    ) -> Result<String> {
        // For now, let's use the backend directly to make LLM calls
        // This will be more reliable than trying to downcast the agent
        self.generate_with_backend(context).await
    }

    /// Generate commit message using the backend directly
    async fn generate_with_backend(&self, context: &str) -> Result<String> {
        use rig::client::CompletionClient;
        use rig::completion::Prompt;

        let user_prompt = format!(
            "Analyze the following Git changes and generate an appropriate commit message:\n\n{context}"
        );

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client.agent(model)
                    .preamble("You are an expert Git commit message writer. Generate clear, conventional commit messages.")
                    .temperature(0.3)
                    .max_tokens(300)  // Enough for detailed commit messages with body
                    .build();

                let response = agent
                    .prompt(&user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI API error: {}", e))?;

                Ok(response.trim().to_string())
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client.agent(model)
                    .preamble("You are an expert Git commit message writer. Generate clear, conventional commit messages.")
                    .temperature(0.3)
                    .max_tokens(300)  // Enough for detailed commit messages with body
                    .build();

                let response = agent
                    .prompt(&user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic API error: {}", e))?;

                Ok(response.trim().to_string())
            }
        }
    }

    /// Validate and enhance the generated commit message
    async fn validate_commit_message(
        &self,
        message: &str,
        _context: &AgentContext,
    ) -> Result<String> {
        // Basic validation and enhancement
        let mut validated = message.trim().to_string();

        // Ensure first line is not too long
        let lines: Vec<&str> = validated.lines().collect();
        if let Some(first_line) = lines.first() {
            if first_line.len() > 72 {
                // Truncate and add appropriate ending
                validated = format!(
                    "{}\n\n{}",
                    &first_line[..69].trim_end(),
                    lines[1..].join("\n")
                );
            }
        }

        Ok(validated)
    }

    /// Analyze repository changes for commit preparation
    async fn analyze_changes(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        // Get current status and diff
        let git_tool = self
            .tools
            .iter()
            .find(|t| t.capabilities().contains(&"git".to_string()))
            .ok_or_else(|| anyhow::anyhow!("Git tool not found"))?;

        let mut status_params = HashMap::new();
        status_params.insert(
            "operation".to_string(),
            serde_json::Value::String("status".to_string()),
        );

        let status_result = git_tool.execute(context, &status_params).await?;

        let mut diff_params = HashMap::new();
        diff_params.insert(
            "operation".to_string(),
            serde_json::Value::String("diff".to_string()),
        );

        let diff_result = git_tool.execute(context, &diff_params).await?;

        Ok(TaskResult::success_with_data(
            "Repository analysis complete".to_string(),
            serde_json::json!({
                "status": status_result,
                "diff": diff_result,
                "analysis_timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        ))
    }

    /// Create system prompt using the same logic as the existing system
    async fn create_system_prompt(
        &self,
        config: &crate::config::Config,
        preset: &str,
        custom_instructions: &str,
        use_gitmoji: bool,
    ) -> Result<String> {
        let commit_schema = schemars::schema_for!(GeneratedMessage);
        let commit_schema_str = serde_json::to_string_pretty(&commit_schema)?;

        let mut prompt = String::from(
            "You are an AI assistant specializing in creating high-quality, professional Git commit messages. \
            Your task is to generate clear, concise, and informative commit messages based solely on the provided context.
            
            Work step-by-step and follow these guidelines exactly:

            1. Use the imperative mood in the subject line (e.g., 'Add feature' not 'Added feature').
            2. Limit the subject line to 50 characters if possible, but never exceed 72 characters.
            3. Capitalize the subject line.
            4. Do not end the subject line with a period.
            5. Separate subject from body with a blank line.
            6. Wrap the body at 72 characters.
            7. Use the body to explain what changes were made and their impact, and how they were implemented.
            8. Be specific and avoid vague language.
            9. Focus on the concrete changes and their effects, not assumptions about intent.
            10. If the changes are part of a larger feature or fix, state this fact if evident from the context.
            11. For non-trivial changes, include a brief explanation of the change's purpose if clearly indicated in the context.
            12. Do not include a conclusion or end summary section.
            13. Avoid common cliché words (like 'enhance', 'streamline', 'leverage', etc) and phrases.
            14. Don't mention filenames in the subject line unless absolutely necessary.
            15. Only describe changes that are explicitly shown in the provided context.
            16. If the purpose or impact of a change is not clear from the context, focus on describing the change itself without inferring intent.
            17. Do not use phrases like 'seems to', 'appears to', or 'might be' - only state what is certain based on the context.
            18. If there's not enough information to create a complete, authoritative message, state only what can be confidently determined from the context.
            19. NO YAPPING!

            Be sure to quote newlines and any other control characters in your response.

            The message should be based entirely on the information provided in the context,
            without any speculation or assumptions.
          ");

        // Add instruction preset and custom instructions
        let mut updated_config = config.clone();
        updated_config.instruction_preset = preset.to_string();
        if !custom_instructions.is_empty() {
            updated_config.instructions = custom_instructions.to_string();
        }
        prompt.push_str(&get_combined_instructions(&updated_config));

        // Check if using conventional commits preset - if so, explicitly disable gitmoji
        let is_conventional = preset == "conventional";

        if use_gitmoji && !is_conventional {
            prompt.push_str(
                "\n\nUse a single gitmoji at the start of the commit message. \
              Choose the most relevant emoji from the following list:\n\n",
            );
            prompt.push_str(&get_gitmoji_list());
        } else if is_conventional {
            prompt.push_str(
                "\n\nIMPORTANT: This is using Conventional Commits format. \
              DO NOT include any emojis in the commit message. \
              Set the emoji field to null in your response.",
            );
        }

        prompt.push_str("
            Your response must be a valid JSON object with the following structure:

            {
              \"emoji\": \"string or null\",
              \"title\": \"string\",
              \"message\": \"string\"
            }

            Follow these steps to generate the commit message:

            1. Analyze the provided context, including staged changes, recent commits, and project metadata.
            2. Identify the main purpose of the commit based on the changes.
            3. Create a concise and descriptive title (subject line) for the commit.
            4. If using emojis (false unless stated below), select the most appropriate one for the commit type.
            5. Write a detailed message body explaining the changes, their impact, and any other relevant information.
            6. Ensure the message adheres to the guidelines above, and follows all of the additional instructions provided.
            7. Construct the final JSON object with the emoji (if applicable), title, and message.

             Here's a minimal example of the expected output format:

            {
              \"emoji\": \"✨\",
              \"title\": \"Add user authentication feature\",
              \"message\": \"Implement user authentication using JWT tokens\\n\\n- Add login and registration endpoints\\n- Create middleware for token verification\\n- Update user model to include password hashing\\n- Add unit tests for authentication functions\"
            }

            Ensure that your response is a valid JSON object matching this structure. Include an empty string for the emoji if not using one.
            "
        );

        prompt.push_str(&commit_schema_str);

        Ok(prompt)
    }

    /// Create user prompt with full context
    async fn create_user_prompt(&self, context: &FullCommitContext) -> Result<String> {
        let detailed_changes = self
            .format_detailed_changes(&context.staged_files, &context.file_analyses)
            .await?;

        let prompt = format!(
            "Based on the following context, generate a Git commit message:\n\n\
            Branch: {}\n\n\
            Recent commits:\n{}\n\n\
            Staged changes:\n{}\n\n\
            Project metadata:\n{}\n\n\
            Detailed changes:\n{}",
            context.branch,
            self.format_recent_commits(&context.recent_commits).await?,
            self.format_staged_files(&context.staged_files).await?,
            self.format_project_metadata(&context.project_metadata)
                .await?,
            detailed_changes
        );

        Ok(prompt)
    }

    /// Generate with backend using full system and user prompts
    async fn generate_with_backend_full(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        use rig::client::CompletionClient;
        use rig::completion::Prompt;

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                log_debug!("🤖 Using OpenAI backend with model: {}", model);
                log_debug!("⚙️  LLM Configuration: temperature=0.3, max_tokens=800");

                let agent = client
                    .agent(model)
                    .preamble(system_prompt)
                    .temperature(0.3)
                    .max_tokens(800) // Increased for JSON response format
                    .build();

                log_debug!(
                    "📤 Sending prompt to OpenAI (user prompt: {} chars)",
                    user_prompt.len()
                );
                let response = agent
                    .prompt(user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI API error: {}", e))?;

                log_debug!("📥 Received response from OpenAI: {} chars", response.len());
                Ok(response.trim().to_string())
            }
            AgentBackend::Anthropic { client, model } => {
                log_debug!("🤖 Using Anthropic backend with model: {}", model);
                log_debug!("⚙️  LLM Configuration: temperature=0.3, max_tokens=800");

                let agent = client
                    .agent(model)
                    .preamble(system_prompt)
                    .temperature(0.3)
                    .max_tokens(800) // Increased for JSON response format
                    .build();

                log_debug!(
                    "📤 Sending prompt to Anthropic (user prompt: {} chars)",
                    user_prompt.len()
                );
                let response = agent
                    .prompt(user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic API error: {}", e))?;

                log_debug!(
                    "📥 Received response from Anthropic: {} chars",
                    response.len()
                );
                Ok(response.trim().to_string())
            }
        }
    }

    /// Parse commit response from JSON
    async fn parse_commit_response(&self, response: &str) -> Result<ParsedCommitResponse> {
        log_debug!("🔍 Attempting to parse LLM response as JSON");

        // Try to parse as JSON first
        if let Ok(generated) = serde_json::from_str::<GeneratedMessage>(response) {
            log_debug!("✅ Successfully parsed JSON response directly");
            log_debug!(
                "📝 Parsed result - Title: '{}', Emoji: {:?}",
                generated.title,
                generated.emoji
            );
            return Ok(ParsedCommitResponse {
                emoji: generated.emoji,
                title: generated.title,
                message: generated.message,
            });
        }

        log_debug!("⚠️  Direct JSON parsing failed, trying to extract JSON from wrapped text");

        // Fallback: try to extract JSON from response if it's wrapped in text
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                log_debug!(
                    "🔍 Found JSON-like content from position {} to {}",
                    start,
                    end
                );

                if let Ok(generated) = serde_json::from_str::<GeneratedMessage>(json_part) {
                    log_debug!("✅ Successfully parsed extracted JSON");
                    log_debug!(
                        "📝 Parsed result - Title: '{}', Emoji: {:?}",
                        generated.title,
                        generated.emoji
                    );
                    return Ok(ParsedCommitResponse {
                        emoji: generated.emoji,
                        title: generated.title,
                        message: generated.message,
                    });
                }
                log_debug!("⚠️  Extracted JSON parsing failed");
            }
        }

        log_debug!("⚠️  All JSON parsing attempts failed, falling back to plain text parsing");

        // Final fallback: treat as plain text
        let lines: Vec<&str> = response.lines().collect();
        let title = lines.first().unwrap_or(&"Update files").trim().to_string();
        let message = response.to_string();

        log_debug!("📝 Plain text fallback - Title: '{}'", title);

        Ok(ParsedCommitResponse {
            emoji: None,
            title,
            message,
        })
    }

    /// Parse staged files from tool results
    async fn parse_staged_files(
        &self,
        diff_result: &serde_json::Value,
        files_result: &serde_json::Value,
    ) -> Result<Vec<StagedFile>> {
        let diff_content = diff_result
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let files: Vec<String> = files_result
            .get("content")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut staged_files = Vec::new();
        for file_path in files {
            // For simplicity, we'll create basic StagedFile structs
            // In a full implementation, we'd parse the actual diff to get proper change types and diffs
            staged_files.push(StagedFile {
                path: file_path,
                change_type: ChangeType::Modified, // Default to modified
                diff: diff_content.to_string(), // This should be per-file, but simplified for now
                analysis: vec!["File changed".to_string()],
                content: None,
                content_excluded: false,
            });
        }

        Ok(staged_files)
    }

    /// Parse recent commits from tool results
    async fn parse_recent_commits(
        &self,
        log_result: &serde_json::Value,
    ) -> Result<Vec<RecentCommit>> {
        let commits_content = log_result
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // This is a simplified parser - in practice, we'd use structured git log output
        let mut commits = Vec::new();
        let lines: Vec<&str> = commits_content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if line.starts_with("commit ") {
                let hash = line.strip_prefix("commit ").unwrap_or("").trim();
                let mut author = String::new();
                let mut timestamp = String::new();
                let mut message = String::new();

                i += 1;
                while i < lines.len() && !lines[i].starts_with("commit ") {
                    let current_line = lines[i];
                    if current_line.starts_with("Author: ") {
                        author = current_line
                            .strip_prefix("Author: ")
                            .unwrap_or("")
                            .trim()
                            .to_string();
                    } else if current_line.starts_with("Date: ") {
                        timestamp = current_line
                            .strip_prefix("Date: ")
                            .unwrap_or("")
                            .trim()
                            .to_string();
                    } else if !current_line.trim().is_empty() && message.is_empty() {
                        message = current_line.trim().to_string();
                    }
                    i += 1;
                }

                if !hash.is_empty() && !message.is_empty() {
                    commits.push(RecentCommit {
                        hash: hash.to_string(),
                        message,
                        author,
                        timestamp,
                    });
                }
            } else {
                i += 1;
            }
        }

        Ok(commits)
    }

    /// Gather project metadata
    async fn gather_project_metadata(&self, context: &AgentContext) -> Result<ProjectMetadata> {
        // Use the existing project metadata detection from Git-Iris
        // This would integrate with the file analyzers to detect languages, frameworks, etc.
        Ok(ProjectMetadata::default())
    }

    /// Format detailed changes for context
    async fn format_detailed_changes(
        &self,
        staged_files: &[StagedFile],
        file_analyses: &[serde_json::Value],
    ) -> Result<String> {
        let mut output = String::new();

        for (i, file) in staged_files.iter().enumerate() {
            output.push_str(&format!("File: {}\n", file.path));
            output.push_str(&format!("Change: {:?}\n", file.change_type));

            if let Some(analysis) = file_analyses.get(i) {
                if let Some(file_type) = analysis.get("file_type").and_then(|v| v.as_str()) {
                    output.push_str(&format!("Type: {file_type}\n"));
                }
                if let Some(lines) = analysis.get("lines").and_then(serde_json::Value::as_i64) {
                    output.push_str(&format!("Lines: {lines}\n"));
                }
            }

            output.push_str("Diff:\n");
            output.push_str(&file.diff);
            output.push_str("\n\n");
        }

        Ok(output)
    }

    /// Format recent commits
    async fn format_recent_commits(&self, commits: &[RecentCommit]) -> Result<String> {
        Ok(commits
            .iter()
            .map(|commit| {
                format!(
                    "{} - {}",
                    &commit.hash[..7.min(commit.hash.len())],
                    commit.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }

    /// Format staged files
    async fn format_staged_files(&self, staged_files: &[StagedFile]) -> Result<String> {
        Ok(staged_files
            .iter()
            .map(|file| format!("{:?}: {}", file.change_type, file.path))
            .collect::<Vec<_>>()
            .join("\n"))
    }

    /// Format project metadata
    async fn format_project_metadata(&self, metadata: &ProjectMetadata) -> Result<String> {
        let mut output = String::new();

        if let Some(language) = &metadata.language {
            output.push_str(&format!("Language: {language}\n"));
        }

        if let Some(framework) = &metadata.framework {
            output.push_str(&format!("Framework: {framework}\n"));
        }

        if !metadata.dependencies.is_empty() {
            output.push_str(&format!(
                "Dependencies: {}\n",
                metadata.dependencies.join(", ")
            ));
        }

        Ok(output)
    }
}

#[async_trait]
impl IrisAgent for CommitAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Agent not initialized"));
        }

        match task {
            "generate_commit_message" => self.generate_commit_message(context, params).await,
            "analyze_changes" => self.analyze_changes(context, params).await,
            "validate_commit" => {
                // Implement commit validation logic
                Ok(TaskResult::success(
                    "Commit validation not yet implemented".to_string(),
                ))
            }
            _ => Err(anyhow::anyhow!("Unknown task: {}", task)),
        }
    }

    fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_commit_message"
                | "analyze_changes"
                | "validate_commit"
                | "commit_message_generation"
                | "diff_analysis"
        )
    }

    fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_commit_message" | "commit_message_generation" => 10,
            "analyze_changes" | "diff_analysis" => 8,
            "validate_commit" => 7,
            _ => 0,
        }
    }

    async fn initialize(&mut self, _context: &AgentContext) -> Result<()> {
        // Perform any initialization needed
        self.initialized = true;
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        // Clean up resources
        Ok(())
    }
}
