//! Task context for agent operations
//!
//! This module provides structured, validated context for agent tasks,
//! replacing fragile string-based parameter passing.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Validated, structured context for agent tasks.
///
/// This enum represents the different modes of operation for code analysis,
/// with validation built into the constructors.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum TaskContext {
    /// Analyze staged changes (optionally including unstaged)
    Staged {
        /// Whether to include unstaged changes in analysis
        include_unstaged: bool,
    },

    /// Analyze a single commit
    Commit {
        /// The commit ID (hash, branch name, or commitish like HEAD~1)
        commit_id: String,
    },

    /// Analyze a range of commits or branch comparison
    Range {
        /// Starting reference (exclusive)
        from: String,
        /// Ending reference (inclusive)
        to: String,
    },

    /// Amend the previous commit with staged changes
    /// The agent sees the combined diff from HEAD^1 to staged state
    Amend {
        /// The original commit message being amended
        original_message: String,
    },

    /// Let the agent discover context via tools (default for gen command)
    #[default]
    Discover,
}

impl TaskContext {
    /// Create context for the gen (commit message) command.
    /// Always uses staged changes only.
    pub fn for_gen() -> Self {
        Self::Staged {
            include_unstaged: false,
        }
    }

    /// Create context for amending the previous commit.
    /// The agent will see the combined diff from HEAD^1 to staged state.
    pub fn for_amend(original_message: String) -> Self {
        Self::Amend { original_message }
    }

    /// Create context for the review command with full parameter validation.
    ///
    /// Validates:
    /// - `--from` requires `--to` for range comparison
    /// - `--commit` is mutually exclusive with `--from/--to`
    /// - `--include-unstaged` is incompatible with range comparisons
    pub fn for_review(
        commit: Option<String>,
        from: Option<String>,
        to: Option<String>,
        include_unstaged: bool,
    ) -> Result<Self> {
        // Validate: --from requires --to
        if from.is_some() && to.is_none() {
            bail!("When using --from, you must also specify --to for branch comparison reviews");
        }

        // Validate: --commit is mutually exclusive with --from/--to
        if commit.is_some() && (from.is_some() || to.is_some()) {
            bail!("Cannot use --commit with --from/--to. These are mutually exclusive options");
        }

        // Validate: --include-unstaged incompatible with range comparisons
        if include_unstaged && (from.is_some() || to.is_some()) {
            bail!(
                "Cannot use --include-unstaged with --from/--to. Branch comparison reviews don't include working directory changes"
            );
        }

        // Route to correct variant based on parameters
        Ok(match (commit, from, to) {
            (Some(id), _, _) => Self::Commit { commit_id: id },
            (_, Some(f), Some(t)) => Self::Range { from: f, to: t },
            _ => Self::Staged { include_unstaged },
        })
    }

    /// Create context for the PR command.
    ///
    /// PR command is more flexible - all parameter combinations are valid:
    /// - `from` + `to`: Explicit range/branch comparison
    /// - `from` only: Compare `from..HEAD`
    /// - `to` only: Compare `main..to`
    /// - Neither: Compare `main..HEAD`
    pub fn for_pr(from: Option<String>, to: Option<String>) -> Self {
        match (from, to) {
            (Some(f), Some(t)) => Self::Range { from: f, to: t },
            (Some(f), None) => Self::Range {
                from: f,
                to: "HEAD".to_string(),
            },
            (None, Some(t)) => Self::Range {
                from: "main".to_string(),
                to: t,
            },
            (None, None) => Self::Range {
                from: "main".to_string(),
                to: "HEAD".to_string(),
            },
        }
    }

    /// Create context for changelog/release-notes commands.
    ///
    /// These always require a `from` reference; `to` defaults to HEAD.
    pub fn for_changelog(from: String, to: Option<String>) -> Self {
        Self::Range {
            from,
            to: to.unwrap_or_else(|| "HEAD".to_string()),
        }
    }

    /// Generate a human-readable prompt context string for the agent.
    pub fn to_prompt_context(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| format!("{self:?}"))
    }

    /// Generate a hint for which `git_diff` call the agent should make.
    pub fn diff_hint(&self) -> String {
        match self {
            Self::Staged { include_unstaged } => {
                if *include_unstaged {
                    "git_diff() for staged changes, then check unstaged files".to_string()
                } else {
                    "git_diff() for staged changes".to_string()
                }
            }
            Self::Commit { commit_id } => {
                format!("git_diff(from=\"{commit_id}^1\", to=\"{commit_id}\")")
            }
            Self::Range { from, to } => {
                format!("git_diff(from=\"{from}\", to=\"{to}\")")
            }
            Self::Amend { .. } => {
                "git_diff(from=\"HEAD^1\") for combined amend diff (original commit + new staged changes)".to_string()
            }
            Self::Discover => "git_diff() to discover current changes".to_string(),
        }
    }

    /// Check if this context represents a range comparison (vs staged/single commit)
    pub fn is_range(&self) -> bool {
        matches!(self, Self::Range { .. })
    }

    /// Check if this context involves unstaged changes
    pub fn includes_unstaged(&self) -> bool {
        matches!(
            self,
            Self::Staged {
                include_unstaged: true
            }
        )
    }

    /// Check if this is an amend operation
    pub fn is_amend(&self) -> bool {
        matches!(self, Self::Amend { .. })
    }

    /// Get the original commit message if this is an amend context
    pub fn original_message(&self) -> Option<&str> {
        match self {
            Self::Amend { original_message } => Some(original_message),
            _ => None,
        }
    }
}

impl std::fmt::Display for TaskContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Staged { include_unstaged } => {
                if *include_unstaged {
                    write!(f, "staged and unstaged changes")
                } else {
                    write!(f, "staged changes")
                }
            }
            Self::Commit { commit_id } => write!(f, "commit {commit_id}"),
            Self::Range { from, to } => write!(f, "changes from {from} to {to}"),
            Self::Amend { .. } => write!(f, "amending previous commit"),
            Self::Discover => write!(f, "auto-discovered changes"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_gen() {
        let ctx = TaskContext::for_gen();
        assert!(matches!(
            ctx,
            TaskContext::Staged {
                include_unstaged: false
            }
        ));
    }

    #[test]
    fn test_review_staged_only() {
        let ctx = TaskContext::for_review(None, None, None, false).expect("should succeed");
        assert!(matches!(
            ctx,
            TaskContext::Staged {
                include_unstaged: false
            }
        ));
    }

    #[test]
    fn test_review_with_unstaged() {
        let ctx = TaskContext::for_review(None, None, None, true).expect("should succeed");
        assert!(matches!(
            ctx,
            TaskContext::Staged {
                include_unstaged: true
            }
        ));
    }

    #[test]
    fn test_review_single_commit() {
        let ctx = TaskContext::for_review(Some("abc123".to_string()), None, None, false)
            .expect("should succeed");
        assert!(matches!(ctx, TaskContext::Commit { commit_id } if commit_id == "abc123"));
    }

    #[test]
    fn test_review_range() {
        let ctx = TaskContext::for_review(
            None,
            Some("main".to_string()),
            Some("feature".to_string()),
            false,
        )
        .expect("should succeed");
        assert!(
            matches!(ctx, TaskContext::Range { from, to } if from == "main" && to == "feature")
        );
    }

    #[test]
    fn test_review_from_without_to_fails() {
        let result = TaskContext::for_review(None, Some("main".to_string()), None, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("--to"));
    }

    #[test]
    fn test_review_commit_with_range_fails() {
        // commit + from + to should fail as mutually exclusive
        let result = TaskContext::for_review(
            Some("abc123".to_string()),
            Some("main".to_string()),
            Some("feature".to_string()),
            false,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("mutually exclusive")
        );
    }

    #[test]
    fn test_review_unstaged_with_range_fails() {
        let result = TaskContext::for_review(
            None,
            Some("main".to_string()),
            Some("feature".to_string()),
            true,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("include-unstaged"));
    }

    #[test]
    fn test_pr_defaults() {
        let ctx = TaskContext::for_pr(None, None);
        assert!(matches!(ctx, TaskContext::Range { from, to } if from == "main" && to == "HEAD"));
    }

    #[test]
    fn test_pr_from_only() {
        let ctx = TaskContext::for_pr(Some("develop".to_string()), None);
        assert!(
            matches!(ctx, TaskContext::Range { from, to } if from == "develop" && to == "HEAD")
        );
    }

    #[test]
    fn test_changelog() {
        let ctx = TaskContext::for_changelog("v1.0.0".to_string(), None);
        assert!(matches!(ctx, TaskContext::Range { from, to } if from == "v1.0.0" && to == "HEAD"));
    }

    #[test]
    fn test_diff_hint() {
        let staged = TaskContext::for_gen();
        assert!(staged.diff_hint().contains("staged"));

        let commit = TaskContext::Commit {
            commit_id: "abc".to_string(),
        };
        assert!(commit.diff_hint().contains("abc^1"));

        let range = TaskContext::Range {
            from: "main".to_string(),
            to: "dev".to_string(),
        };
        assert!(range.diff_hint().contains("main"));
        assert!(range.diff_hint().contains("dev"));

        let amend = TaskContext::for_amend("Fix bug".to_string());
        assert!(amend.diff_hint().contains("HEAD^1"));
    }

    #[test]
    fn test_amend_context() {
        let ctx = TaskContext::for_amend("Initial commit message".to_string());
        assert!(ctx.is_amend());
        assert_eq!(ctx.original_message(), Some("Initial commit message"));
        assert!(!ctx.is_range());
        assert!(!ctx.includes_unstaged());
    }

    #[test]
    fn test_amend_display() {
        let ctx = TaskContext::for_amend("Fix bug".to_string());
        assert_eq!(format!("{ctx}"), "amending previous commit");
    }
}
