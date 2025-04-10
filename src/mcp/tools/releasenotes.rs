//! MCP release notes tool implementation
//!
//! This module provides the MCP tool for generating release notes.

use crate::changes::ReleaseNotesGenerator;
use crate::common::DetailLevel;
use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;

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

    /// Execute the release notes tool with the provided repository and configuration
    pub async fn execute(
        &self,
        git_repo: Arc<GitRepo>,
        config: GitIrisConfig,
    ) -> Result<CallToolResult, anyhow::Error> {
        log_debug!("Generating release notes with: {:?}", self);

        // Parse detail level with robust empty string handling
        let detail_level = if self.detail_level.trim().is_empty() {
            log_debug!("Empty detail level, using Standard");
            DetailLevel::Standard
        } else {
            match self.detail_level.trim().to_lowercase().as_str() {
                "minimal" => DetailLevel::Minimal,
                "detailed" => DetailLevel::Detailed,
                "standard" => DetailLevel::Standard,
                other => {
                    log_debug!("Unknown detail level '{}', defaulting to Standard", other);
                    DetailLevel::Standard
                }
            }
        };

        // Set up config with custom instructions if provided and not empty
        let mut config = config.clone();
        if !self.custom_instructions.trim().is_empty() {
            config.set_temp_instructions(Some(self.custom_instructions.clone()));
        }

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

        // Create and return the result
        Ok(create_text_result(content))
    }
}

/// Helper function to create a text result
fn create_text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![rmcp::model::Content::from(rmcp::model::Annotated {
            raw: rmcp::model::RawContent::Text(rmcp::model::RawTextContent { text }),
            annotations: None,
        })],
        is_error: None,
    }
}
