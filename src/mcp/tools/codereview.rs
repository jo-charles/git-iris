//! MCP code review tool implementation
//!
//! This module provides the MCP tool for generating code reviews with options for
//! staged changes, unstaged changes, and specific commits.

use crate::commit::service::IrisCommitService;
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
                "Generate a comprehensive code review with options for staged changes, unstaged changes, or specific commits",
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

        // Check if local operations are required without a specific commit
        if !self.commit_id.trim().is_empty() {
            // Specific commit review works with remote repositories
        } else if git_repo.is_remote()
            && (self.include_unstaged || self.commit_id.trim().is_empty())
        {
            return Err(anyhow::anyhow!(
                "Cannot review staged/unstaged changes on a remote repository"
            ));
        }

        // Create a commit service for processing
        let repo_path = git_repo.repo_path().clone();
        let provider_name = &config.default_provider;

        let service = IrisCommitService::new(
            config.clone(),
            &repo_path,
            provider_name,
            false, // gitmoji not needed for review
            false, // verification not needed for review
            GitRepo::new(&repo_path)?,
        )?;

        // Set up config with custom instructions if provided
        let mut config_clone = config.clone();
        apply_custom_instructions(&mut config_clone, &self.custom_instructions);

        // Process the preset
        let preset = if self.preset.trim().is_empty() {
            "default"
        } else {
            &self.preset
        };

        // Generate the code review based on parameters
        let review = if !self.commit_id.trim().is_empty() {
            // Review a specific commit
            service
                .generate_review_for_commit(preset, &self.custom_instructions, &self.commit_id)
                .await?
        } else if self.include_unstaged {
            // Review including unstaged changes
            service
                .generate_review_with_unstaged(preset, &self.custom_instructions, true)
                .await?
        } else {
            // Review only staged changes (default behavior)
            service
                .generate_review(preset, &self.custom_instructions)
                .await?
        };

        // Format and return the review
        let formatted_review = review.format();
        Ok(create_text_result(formatted_review))
    }
}
