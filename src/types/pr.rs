//! Pull request types and formatting
//!
//! This module provides markdown-based PR output that lets the LLM drive
//! the structure while we beautify it for terminal display.

use crate::types::review::render_markdown_for_terminal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Markdown-based pull request that lets the LLM determine structure
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MarkdownPullRequest {
    /// The full markdown content of the PR description
    pub content: String,
}

impl MarkdownPullRequest {
    /// Render the markdown content with terminal styling
    pub fn format(&self) -> String {
        render_markdown_for_terminal(&self.content)
    }

    /// Get the raw markdown content (for GitHub/GitLab, etc.)
    pub fn raw_content(&self) -> &str {
        &self.content
    }
}
