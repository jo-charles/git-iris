//! Agent tools module
//!
//! This module contains all the tools available to Iris for performing various operations.
//! Each tool is self-contained and implements the `AgentTool` trait.
//!
//! Available tools:
//! - **`GitTool`**: Repository operations (diff, status, log, files)
//! - **`FileAnalyzerTool`**: File content and structure analysis
//! - **`CodeSearchTool`**: Search for patterns, functions, and classes
//! - **`WorkspaceTool`**: Iris's personal notes and task management

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::core::AgentContext;

/// Core trait for agent tools
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Get the tool's unique identifier
    fn id(&self) -> &str;

    /// Get the tool's display name
    fn name(&self) -> &str;

    /// Get the tool's description
    fn description(&self) -> &str;

    /// Get the capabilities this tool provides
    fn capabilities(&self) -> Vec<String>;

    /// Execute the tool with given parameters
    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value>;

    /// Get the tool's parameter schema
    fn parameter_schema(&self) -> serde_json::Value;
}

/// Simplified tool registry - we always provide all tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Arc<dyn AgentTool>) {
        self.tools.insert(tool.id().to_string(), tool);
    }

    pub fn get_tool(&self, id: &str) -> Option<Arc<dyn AgentTool>> {
        self.tools.get(id).cloned()
    }

    pub fn get_all_tools(&self) -> Vec<Arc<dyn AgentTool>> {
        self.tools.values().cloned().collect()
    }

    pub fn list_tool_ids(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

// Tool modules
pub mod code_search;
pub mod file_analyzer;
pub mod git;
pub mod workspace;

// Re-export all tools
pub use code_search::CodeSearchTool;
pub use file_analyzer::FileAnalyzerTool;
pub use git::GitTool;
pub use workspace::WorkspaceTool;

/// Creates the default tool registry with all built-in tools
pub fn create_default_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry.register_tool(Arc::new(GitTool::new()));
    registry.register_tool(Arc::new(FileAnalyzerTool::new()));
    registry.register_tool(Arc::new(CodeSearchTool::new()));
    registry.register_tool(Arc::new(WorkspaceTool::new()));

    registry
}
