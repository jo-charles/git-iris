//! MCP code review tool implementation
//!
//! This module provides the MCP tool for generating code reviews with options for
//! staged changes, unstaged changes, and specific commits.

use crate::agents::{IrisAgentService, StructuredResponse, TaskContext};
use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;
use crate::mcp::tools::utils::{
    GitIrisTool, apply_custom_instructions, create_text_result, resolve_git_repo,
    validate_repository_parameter,
};

use rmcp::handler::server::tool::cached_schema_for_type;
use rmcp::model::{CallToolResult, Tool};
use rmcp::schemars;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

/// Code review tool for generating comprehensive code reviews
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CodeReviewTool {
    /// Include unstaged changes in the review
    #[serde(default)]
    pub include_unstaged: bool,

    /// Specific commit to review (hash, branch name, or reference)
    #[serde(default)]
    pub commit_id: String,

    /// Starting branch for comparison (defaults to 'main'). Used with to for branch comparison reviews
    #[serde(default)]
    pub from: String,

    /// Target branch for comparison (e.g., 'feature-branch', 'pr-branch'). Used with from for branch comparison reviews
    #[serde(default)]
    pub to: String,

    /// Preset instruction set to use for the review
    #[serde(default)]
    pub preset: String,

    /// Custom instructions for the AI
    #[serde(default)]
    pub custom_instructions: String,

    /// Repository path (local) or URL (remote). Required.
    pub repository: String,
}

impl CodeReviewTool {
    /// Returns the tool definition for the code review tool
    pub fn get_tool_definition() -> Tool {
        Tool {
            name: Cow::Borrowed("git_iris_code_review"),
            description: Cow::Borrowed(
                "Generate a comprehensive code review with options for staged changes, unstaged changes, specific commits, or branch comparisons (e.g., PR reviews)",
            ),
            input_schema: cached_schema_for_type::<Self>(),
        }
    }
}

#[async_trait::async_trait]
impl GitIrisTool for CodeReviewTool {
    /// Execute the code review tool with the provided repository and configuration
    async fn execute(
        &self,
        git_repo: Arc<GitRepo>,
        config: GitIrisConfig,
    ) -> Result<CallToolResult, anyhow::Error> {
        log_debug!("Generating code review with: {:?}", self);

        // Validate repository parameter
        validate_repository_parameter(&self.repository)?;
        let git_repo = resolve_git_repo(Some(self.repository.as_str()), git_repo)?;
        log_debug!("Using repository: {}", git_repo.repo_path().display());

        // Validate parameter combinations
        let has_commit = !self.commit_id.trim().is_empty();
        let has_branches = !self.from.trim().is_empty() || !self.to.trim().is_empty();

        if has_commit && has_branches {
            return Err(anyhow::anyhow!(
                "Cannot use both commit_id and branch parameters. These options are mutually exclusive."
            ));
        }

        if !self.from.trim().is_empty() && self.to.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "When using 'from', you must also specify 'to' for branch comparison reviews"
            ));
        }

        if self.include_unstaged && has_branches {
            return Err(anyhow::anyhow!(
                "Cannot use include_unstaged with branch comparison. Branch reviews compare committed changes only."
            ));
        }

        // Check if local operations are required
        if !has_commit
            && !has_branches
            && git_repo.is_remote()
            && (self.include_unstaged || self.commit_id.trim().is_empty())
        {
            return Err(anyhow::anyhow!(
                "Cannot review staged/unstaged changes on a remote repository"
            ));
        }

        // Set up config with custom instructions if provided
        let mut config_clone = config.clone();
        apply_custom_instructions(&mut config_clone, &self.custom_instructions);

        // Create the appropriate TaskContext based on parameters
        let context = if has_branches {
            // Branch comparison review
            let from_branch = if self.from.trim().is_empty() {
                "main".to_string()
            } else {
                self.from.trim().to_string()
            };
            let to_branch = self.to.trim().to_string();

            TaskContext::Range {
                from: from_branch,
                to: to_branch,
            }
        } else if has_commit {
            // Review a specific commit
            TaskContext::Commit {
                commit_id: self.commit_id.clone(),
            }
        } else {
            // Review staged changes (optionally with unstaged)
            TaskContext::Staged {
                include_unstaged: self.include_unstaged,
            }
        };

        // Create IrisAgentService for LLM operations
        let backend = crate::agents::AgentBackend::from_config(&config_clone)?;
        let agent_service =
            IrisAgentService::new(config_clone, backend.provider_name, backend.model);

        // Generate the code review using agent
        let response = agent_service.execute_task("review", context).await?;

        // Extract the review from the response
        let review = match response {
            StructuredResponse::Review(review) => *review,
            _ => return Err(anyhow::anyhow!("Expected code review response")),
        };

        // Format and return the review
        let formatted_review = review.format();
        Ok(create_text_result(formatted_review))
    }
}
