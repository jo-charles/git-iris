//! Release notes types and formatting
//!
//! This module provides markdown-based release notes output that lets the LLM drive
//! the structure while we beautify it for terminal display.

use crate::types::review::render_markdown_for_terminal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Markdown-based release notes that lets the LLM determine structure
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MarkdownReleaseNotes {
    /// The full markdown content of the release notes
    pub content: String,
}

impl MarkdownReleaseNotes {
    /// Render the markdown content with terminal styling
    pub fn format(&self) -> String {
        render_markdown_for_terminal(&self.content)
    }

    /// Get the raw markdown content (for file output, etc.)
    pub fn raw_content(&self) -> &str {
        &self.content
    }
}
