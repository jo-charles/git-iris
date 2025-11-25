//! Agent tools module
//!
//! This module contains all the tools available to Iris for performing various operations.
//! Each tool implements Rig's Tool trait for proper integration.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::agents::core::AgentContext;

/// Trait for tools that can be used by the agent
/// This is the legacy trait for tools that haven't been converted to Rig yet
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Get the unique identifier for this tool
    fn id(&self) -> &str;

    /// Get the human-readable name of this tool
    fn name(&self) -> &'static str;

    /// Get a description of what this tool does
    fn description(&self) -> &'static str;

    /// Get the capabilities this tool provides
    fn capabilities(&self) -> Vec<String>;

    /// Get the JSON schema for this tool's parameters
    fn parameter_schema(&self) -> Value;

    /// Execute the tool with the given context and parameters
    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, Value>,
    ) -> Result<Value>;
}

// Tool modules with Rig-based implementations
pub mod git;

// Re-export the tool structs (not functions) for Rig agents
pub use git::{GitChangedFiles, GitDiff, GitLog, GitRepoInfo, GitStatus};

// Migrated Rig tools
pub mod file_analyzer;
pub use file_analyzer::FileAnalyzer;

pub mod code_search;
pub use code_search::CodeSearch;

pub mod project_metadata;
pub use project_metadata::ProjectMetadataTool;

pub mod docs;
pub use docs::ProjectDocs;

pub mod workspace_new;
pub use workspace_new::Workspace;

pub mod parallel_analyze;
pub use parallel_analyze::{ParallelAnalyze, ParallelAnalyzeResult, SubagentResult};

// Legacy workspace (unused, kept for reference)
pub mod workspace;
