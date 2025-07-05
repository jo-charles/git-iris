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

    /// Convert to Rig Tool trait object (placeholder for future integration)
    fn as_rig_tool_placeholder(&self) -> String;
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
    capability_index: HashMap<String, Vec<String>>,
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
            capability_index: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Arc<dyn AgentTool>) {
        let id = tool.id().to_string();

        // Add to capability index
        for capability in tool.capabilities() {
            self.capability_index
                .entry(capability)
                .or_default()
                .push(id.clone());
        }

        // Add to tools registry
        self.tools.insert(id, tool);
    }

    pub fn get_tool(&self, id: &str) -> Option<Arc<dyn AgentTool>> {
        self.tools.get(id).cloned()
    }

    pub fn get_tools_for_capability(&self, capability: &str) -> Result<Vec<Arc<dyn AgentTool>>> {
        let empty_vec = Vec::new();
        let tool_ids = self.capability_index.get(capability).unwrap_or(&empty_vec);

        let mut tools = Vec::new();
        for id in tool_ids {
            if let Some(tool) = self.tools.get(id) {
                tools.push(tool.clone());
            }
        }

        Ok(tools)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn list_capabilities(&self) -> Vec<String> {
        self.capability_index.keys().cloned().collect()
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
