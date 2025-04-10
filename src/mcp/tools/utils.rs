//! Common utilities for MCP tools
//!
//! This module provides shared functionality used across different MCP tool implementations.

use crate::common::DetailLevel;
use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use rmcp::model::{Annotated, CallToolResult, Content, RawContent, RawTextContent};
use std::sync::Arc;

/// Common trait for all Git-Iris MCP tools
///
/// This trait defines the common interface that all Git-Iris tools must implement.
#[async_trait::async_trait]
pub trait GitIrisTool {
    /// Execute the tool with the provided repository and configuration
    async fn execute(
        &self,
        git_repo: Arc<GitRepo>,
        config: GitIrisConfig,
    ) -> Result<CallToolResult, anyhow::Error>;
}

/// Creates a text result response for tool calls
///
/// This is a common utility used by all tools to return a text response.
pub fn create_text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::from(Annotated {
            raw: RawContent::Text(RawTextContent { text }),
            annotations: None,
        })],
        is_error: None,
    }
}

/// Parses a detail level string into the corresponding enum value
///
/// This provides consistent handling of detail level across all tools.
pub fn parse_detail_level(detail_level: &str) -> DetailLevel {
    if detail_level.trim().is_empty() {
        return DetailLevel::Standard;
    }

    match detail_level.trim().to_lowercase().as_str() {
        "minimal" => DetailLevel::Minimal,
        "detailed" => DetailLevel::Detailed,
        _ => DetailLevel::Standard,
    }
}

/// Apply custom instructions to config if provided
pub fn apply_custom_instructions(config: &mut crate::config::Config, custom_instructions: &str) {
    if !custom_instructions.trim().is_empty() {
        config.set_temp_instructions(Some(custom_instructions.to_string()));
    }
}
