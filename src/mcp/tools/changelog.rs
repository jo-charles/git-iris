//! MCP changelog tool implementation
//!
//! This module provides the MCP tool for generating changelogs.

use crate::changes::ChangelogGenerator;
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

/// Changelog tool for generating comprehensive changelog documents
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ChangelogTool {
    /// Starting reference (commit hash, tag, or branch name)
    pub from: String,

    /// Ending reference (commit hash, tag, or branch name). Defaults to HEAD if not specified.
    #[serde(default)]
    pub to: String,

    /// Level of detail for the changelog
    #[serde(default)]
    pub detail_level: String,

    /// Custom instructions for the AI
    #[serde(default)]
    pub custom_instructions: String,
}

impl ChangelogTool {
    /// Returns the tool definition for the changelog tool
    pub fn get_tool_definition() -> Tool {
        Tool {
            name: Cow::Borrowed("git_iris_changelog"),
            description: Some(Cow::Borrowed(
                "Generate a detailed changelog between two Git references",
            )),
            input_schema: cached_schema_for_type::<Self>(),
            annotations: None,
        }
    }

    /// Execute the changelog tool with the provided repository and configuration
    pub async fn execute(
        &self,
        git_repo: Arc<GitRepo>,
        config: GitIrisConfig,
    ) -> Result<CallToolResult, anyhow::Error> {
        log_debug!("Generating changelog with: {:?}", self);

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

        // Generate the changelog using the generator
        let content =
            ChangelogGenerator::generate(git_repo.clone(), &self.from, &to, &config, detail_level)
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
