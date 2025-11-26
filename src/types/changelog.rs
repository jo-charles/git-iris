//! Changelog types and formatting

use crate::log_debug;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the structured response for a changelog
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
pub struct ChangelogResponse {
    /// The version number of the release
    pub version: Option<String>,
    /// The date of the release
    pub release_date: Option<String>,
    /// Categorized changes, grouped by type
    pub sections: HashMap<ChangelogType, Vec<ChangeEntry>>,
    /// List of breaking changes in this release
    pub breaking_changes: Vec<BreakingChange>,
    /// Metrics summarizing the changes in this release
    pub metrics: ChangeMetrics,
}

/// Enumeration of possible change types for changelog entries
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Hash)]
pub enum ChangelogType {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

/// Represents a single change entry in the changelog
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

/// Represents a breaking change in the release
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
pub struct BreakingChange {
    /// Description of the breaking change
    pub description: String,
    /// Commit hash associated with this breaking change
    pub commit_hash: String,
}

/// Metrics summarizing the changes in a release
#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
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

impl ChangelogResponse {
    /// Generate the content string for display
    pub fn content(&self) -> String {
        let mut output = String::new();

        if let Some(version) = &self.version {
            output.push_str(&format!("# Changelog - {version}\n\n"));
        } else {
            output.push_str("# Changelog\n\n");
        }

        if let Some(date) = &self.release_date {
            output.push_str(&format!("Released: {date}\n\n"));
        }

        for (change_type, entries) in &self.sections {
            if !entries.is_empty() {
                output.push_str(&format!("## {change_type:?}\n\n"));
                for entry in entries {
                    output.push_str(&format!("- {}\n", entry.description));
                }
                output.push('\n');
            }
        }

        if !self.breaking_changes.is_empty() {
            output.push_str("## Breaking Changes\n\n");
            for change in &self.breaking_changes {
                output.push_str(&format!("- {}\n", change.description));
            }
            output.push('\n');
        }

        output
    }
}

impl From<String> for ChangelogResponse {
    /// Converts a JSON string to a `ChangelogResponse`
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap_or_else(|e| {
            log_debug!("Failed to parse ChangelogResponse: {}", e);
            Self {
                version: Some("Error".to_string()),
                release_date: Some("Error".to_string()),
                sections: HashMap::new(),
                breaking_changes: Vec::new(),
                metrics: ChangeMetrics {
                    total_commits: 0,
                    files_changed: 0,
                    insertions: 0,
                    deletions: 0,
                    total_lines_changed: 0,
                },
            }
        })
    }
}
