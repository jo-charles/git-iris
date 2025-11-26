//! Agent tools module
//!
//! This module contains all the tools available to Iris for performing various operations.
//! Each tool implements Rig's Tool trait for proper integration.

// Common utilities shared across tools
pub mod common;
pub use common::{get_current_repo, parameters_schema};

// Tool modules with Rig-based implementations
pub mod git;

// Re-export the tool structs (not functions) for Rig agents
pub use git::{GitChangedFiles, GitDiff, GitLog, GitRepoInfo, GitStatus};

// Migrated Rig tools
pub mod file_analyzer;
pub use file_analyzer::FileAnalyzer;

pub mod file_read;
pub use file_read::FileRead;

pub mod code_search;
pub use code_search::CodeSearch;

pub mod project_metadata;
pub use project_metadata::ProjectMetadataTool;

pub mod docs;
pub use docs::ProjectDocs;

pub mod workspace;
pub use workspace::Workspace;

pub mod parallel_analyze;
pub use parallel_analyze::{ParallelAnalyze, ParallelAnalyzeResult, SubagentResult};
