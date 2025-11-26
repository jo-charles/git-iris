//! MCP commit tool implementation
//!
//! This module provides the MCP tool for generating and performing commits.

use crate::agents::{IrisAgentService, StructuredResponse, TaskContext};
use crate::commit::types::format_commit_message;
use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;
use crate::mcp::tools::utils::{
    GitIrisTool, create_text_result, resolve_git_repo, validate_repository_parameter,
};
use crate::services::GitCommitService;

use rmcp::handler::server::tool::cached_schema_for_type;
use rmcp::model::{CallToolResult, Tool};
use rmcp::schemars;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

/// Commit tool for generating commit messages and performing commits
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CommitTool {
    /// Whether to generate and perform the commit (true) or just generate a message (false)
    #[serde(default)]
    pub auto_commit: bool,

    /// Whether to use gitmoji in commit messages
    #[serde(default)]
    pub use_gitmoji: bool,

    /// Whether to skip commit verification
    #[serde(default)]
    pub no_verify: bool,

    /// Instruction preset to use
    #[serde(default)]
    pub preset: String,

    /// Custom instructions for the AI
    #[serde(default)]
    pub custom_instructions: String,

    /// Repository path (local) or URL (remote). Required.
    pub repository: String,
}

impl CommitTool {
    /// Returns the tool definition for the commit tool
    pub fn get_tool_definition() -> Tool {
        Tool {
            name: Cow::Borrowed("git_iris_commit"),
            description: Cow::Borrowed("Generate commit messages and perform Git commits"),
            input_schema: cached_schema_for_type::<Self>(),
        }
    }
}

#[async_trait::async_trait]
impl GitIrisTool for CommitTool {
    /// Execute the commit tool with the provided repository and configuration
    async fn execute(
        &self,
        git_repo: Arc<GitRepo>,
        config: GitIrisConfig,
    ) -> Result<CallToolResult, anyhow::Error> {
        log_debug!("Processing commit request with: {:?}", self);

        // Validate repository parameter
        validate_repository_parameter(&self.repository)?;
        let git_repo = resolve_git_repo(Some(self.repository.as_str()), git_repo)?;
        log_debug!("Using repository: {}", git_repo.repo_path().display());

        // Check if we can perform the operation on this repository
        if self.auto_commit && git_repo.is_remote() {
            return Err(anyhow::anyhow!("Cannot auto-commit to a remote repository"));
        }

        let repo_path = git_repo.repo_path().clone();
        let verify = !self.no_verify;

        // Create GitCommitService for commit operations
        let commit_service = GitCommitService::new(
            Arc::new(GitRepo::new(&repo_path)?),
            self.use_gitmoji,
            verify,
        );

        // First check if we have staged changes
        let git_info = git_repo.get_git_info(&config).await?;
        if git_info.staged_files.is_empty() {
            return Err(anyhow::anyhow!(
                "No staged changes. Please stage your changes before generating a commit message."
            ));
        }

        // Run pre-commit hook
        if let Err(e) = commit_service.pre_commit() {
            return Err(anyhow::anyhow!("Pre-commit failed: {}", e));
        }

        // Create IrisAgentService for LLM operations
        let mut agent_config = config.clone();
        if !self.custom_instructions.is_empty() {
            agent_config
                .instructions
                .clone_from(&self.custom_instructions);
        }

        // Create the service from config directly
        let backend = crate::agents::AgentBackend::from_config(&agent_config)?;
        let agent_service =
            IrisAgentService::new(agent_config, backend.provider_name, backend.model);

        // Generate a commit message using agent
        let context = TaskContext::for_gen();
        let response = agent_service.execute_task("commit", context).await?;

        // Extract the commit message from the response
        let StructuredResponse::CommitMessage(message) = response else {
            return Err(anyhow::anyhow!("Expected commit message response"));
        };

        let formatted_message = format_commit_message(&message);

        // If auto_commit is true, perform the commit
        if self.auto_commit {
            match commit_service.perform_commit(&formatted_message) {
                Ok(result) => {
                    // Create result with commit info
                    let result_text = format!(
                        "Commit successful! [{}]\n\n{}\n\n{} file{} changed, {} insertion{}(+), {} deletion{}(-)",
                        result.commit_hash,
                        formatted_message,
                        result.files_changed,
                        if result.files_changed == 1 { "" } else { "s" },
                        result.insertions,
                        if result.insertions == 1 { "" } else { "s" },
                        result.deletions,
                        if result.deletions == 1 { "" } else { "s" }
                    );

                    return Ok(create_text_result(result_text));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to commit: {}", e));
                }
            }
        }

        // If we're just generating a message, return it
        Ok(create_text_result(formatted_message))
    }
}
