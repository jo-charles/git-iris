//! Changelog types and formatting
//!
//! This module provides markdown-based changelog output that lets the LLM drive
//! the structure while we beautify it for terminal display.

use crate::types::review::render_markdown_for_terminal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Markdown-based changelog that lets the LLM determine structure
///
/// Follows the Keep a Changelog format (<https://keepachangelog.com>/) but allows
/// the LLM flexibility in how it organizes and presents changes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MarkdownChangelog {
    /// The full markdown content of the changelog entry
    pub content: String,
}

impl MarkdownChangelog {
    /// Render the markdown content with terminal styling
    pub fn format(&self) -> String {
        render_markdown_for_terminal(&self.content)
    }

    /// Get the raw markdown content (for file output, etc.)
    pub fn raw_content(&self) -> &str {
        &self.content
    }
}

// Re-export the types that are still used by tests and update_changelog_file
// These are used for test assertions but not for LLM output

/// Enumeration of possible change types for changelog entries (for reference)
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Hash)]
pub enum ChangelogType {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

/// Metrics summarizing the changes in a release
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, Default)]
pub struct ChangeMetrics {
    /// Total number of commits in this release
    pub total_commits: usize,
    /// Number of files changed in this release
    pub files_changed: usize,
    /// Number of lines inserted in this release
    pub insertions: usize,
    /// Number of lines deleted in this release
    pub deletions: usize,
    /// Total lines changed in this release
    pub total_lines_changed: usize,
}

/// Represents a single change entry in the changelog (for test assertions)
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
pub struct ChangeEntry {
    /// Description of the change
    pub description: String,
    /// List of commit hashes associated with this change
    pub commit_hashes: Vec<String>,
    /// List of issue numbers associated with this change
    pub associated_issues: Vec<String>,
    /// Pull request number associated with this change, if any
    pub pull_request: Option<String>,
}
