//! MCP release notes tool implementation
//!
//! This module provides the MCP tool for generating release notes.

use crate::changes::ReleaseNotesGenerator;
use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;
use crate::mcp::tools::utils::{
    GitIrisTool, apply_custom_instructions, create_text_result, parse_detail_level,
    resolve_git_repo,
};

use rmcp::handler::server::tool::cached_schema_for_type;
use rmcp::model::{CallToolResult, Tool};
use rmcp::schemars;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

/// Release notes tool for generating comprehensive release notes
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReleaseNotesTool {
    /// Starting reference (commit hash, tag, or branch name)
    pub from: String,

    /// Ending reference (commit hash, tag, or branch name). Defaults to HEAD if not specified.
    #[serde(default)]
    pub to: String,

    /// Level of detail for the release notes
    #[serde(default)]
    pub detail_level: String,

    /// Custom instructions for the AI
    #[serde(default)]
    pub custom_instructions: String,

    /// Repository path or URL (optional)
    #[serde(default)]
    pub repository: String,
}

impl ReleaseNotesTool {
    /// Returns the tool definition for the release notes tool
    pub fn get_tool_definition() -> Tool {
        Tool {
            name: Cow::Borrowed("git_iris_release_notes"),
            description: Some(Cow::Borrowed(
                "Generate comprehensive release notes between two Git references",
            )),
            input_schema: cached_schema_for_type::<Self>(),
            annotations: None,
        }
    }
}

#[async_trait::async_trait]
impl GitIrisTool for ReleaseNotesTool {
    /// Execute the release notes tool with the provided repository and configuration
    async fn execute(
        &self,
        git_repo: Arc<GitRepo>,
        config: GitIrisConfig,
    ) -> Result<CallToolResult, anyhow::Error> {
        log_debug!("Generating release notes with: {:?}", self);

        // Resolve repository based on the repository parameter
        let repo_path = if self.repository.trim().is_empty() {
            None
        } else {
            Some(self.repository.as_str())
        };
        let git_repo = resolve_git_repo(repo_path, git_repo)?;
        log_debug!("Using repository: {}", git_repo.repo_path().display());

        // Parse detail level using shared utility
        let detail_level = parse_detail_level(&self.detail_level);

        // Set up config with custom instructions if provided
        let mut config = config.clone();
        apply_custom_instructions(&mut config, &self.custom_instructions);

        // Default to HEAD if to is empty
        let to = if self.to.trim().is_empty() {
            "HEAD".to_string()
        } else {
            self.to.clone()
        };

        // Generate the release notes using the generator
        let content = ReleaseNotesGenerator::generate(
            git_repo.clone(),
            &self.from,
            &to,
            &config,
            detail_level,
        )
        .await?;

        // Create and return the result using shared utility
        Ok(create_text_result(content))
    }
}
