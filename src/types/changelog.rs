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
    /// Brief summary describing the release highlights (1-3 sentences)
    #[serde(default)]
    pub summary: Option<String>,
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

impl ChangelogType {
    /// Returns the emoji associated with this change type
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Added => "✨",
            Self::Changed => "🔄",
            Self::Deprecated => "⚠️",
            Self::Removed => "🗑️",
            Self::Fixed => "🐛",
            Self::Security => "🔒",
        }
    }

    /// Returns the display order for consistent section ordering
    pub const fn order(&self) -> u8 {
        match self {
            Self::Added => 0,
            Self::Changed => 1,
            Self::Fixed => 2,
            Self::Security => 3,
            Self::Deprecated => 4,
            Self::Removed => 5,
        }
    }
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
    /// Generate the content string for display (rich markdown format)
    pub fn content(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Header with version
        if let Some(version) = &self.version {
            let _ = writeln!(
                output,
                "## [{version}] - {}",
                self.release_date.as_deref().unwrap_or("")
            );
        } else {
            output.push_str("## [Unreleased]\n");
        }
        output.push('\n');

        // Summary if present
        if let Some(summary) = &self.summary {
            let _ = writeln!(output, "{summary}\n");
        }

        // Sort sections by display order
        let mut sorted_sections: Vec<_> = self.sections.iter().collect();
        sorted_sections.sort_by_key(|(t, _)| t.order());

        for (change_type, entries) in sorted_sections {
            if !entries.is_empty() {
                let _ = writeln!(output, "### {} {change_type:?}\n", change_type.emoji());
                for entry in entries {
                    // Format: - Description (hash1, hash2) #issue PR #123
                    let mut line = format!("- {}", entry.description);

                    // Add commit hashes if present
                    if !entry.commit_hashes.is_empty() {
                        let hashes: Vec<_> = entry
                            .commit_hashes
                            .iter()
                            .map(|h| if h.len() > 7 { &h[..7] } else { h.as_str() })
                            .collect();
                        let _ = write!(line, " ({})", hashes.join(", "));
                    }

                    // Add issue references
                    for issue in &entry.associated_issues {
                        let _ = write!(line, " {issue}");
                    }

                    // Add PR reference
                    if let Some(pr) = &entry.pull_request {
                        let _ = write!(line, " {pr}");
                    }

                    let _ = writeln!(output, "{line}");
                }
                output.push('\n');
            }
        }

        // Breaking changes section
        if !self.breaking_changes.is_empty() {
            output.push_str("### 💥 Breaking Changes\n\n");
            for change in &self.breaking_changes {
                let hash = if change.commit_hash.len() > 7 {
                    &change.commit_hash[..7]
                } else {
                    &change.commit_hash
                };
                let _ = writeln!(output, "- {} ({hash})", change.description);
            }
            output.push('\n');
        }

        // Metrics section
        let _ = writeln!(output, "### 📊 Metrics\n");
        let _ = writeln!(output, "- Total Commits: {}", self.metrics.total_commits);
        let _ = writeln!(output, "- Files Changed: {}", self.metrics.files_changed);
        let _ = writeln!(output, "- Insertions: {}", self.metrics.insertions);
        let _ = writeln!(output, "- Deletions: {}", self.metrics.deletions);
        let _ = writeln!(
            output,
            "- Total Lines Changed: {}",
            self.metrics.total_lines_changed
        );

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
                summary: None,
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
