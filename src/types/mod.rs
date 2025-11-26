//! Response types for agent-generated content
//!
//! This module consolidates all structured output types that the Iris agent produces:
//! - Commit messages
//! - Pull request descriptions
//! - Code reviews
//! - Changelogs
//! - Release notes

mod changelog;
mod commit;
mod pr;
mod release_notes;
mod review;

// Commit types
pub use self::commit::{GeneratedMessage, format_commit_message};

// PR types
pub use pr::{GeneratedPullRequest, format_pull_request};

// Review types
pub use review::{CodeIssue, DimensionAnalysis, GeneratedReview, QualityDimension};

// Changelog types
pub use changelog::{BreakingChange, ChangeEntry, ChangeMetrics, ChangelogResponse, ChangelogType};

// Release notes types
pub use release_notes::{Highlight, ReleaseNotesResponse, Section, SectionItem};
