//! Code search tool
//!
//! This tool provides Iris with the ability to search for code patterns,
//! functions, classes, and related files in the repository.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AgentTool;
use crate::agents::core::AgentContext;

/// Code search tool for finding related files and functions
pub struct CodeSearchTool {
    id: String,
}

impl Default for CodeSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSearchTool {
    pub fn new() -> Self {
        Self {
            id: "code_search".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CodeSearchArgs {
    pub query: String,
    pub search_type: String, // "function", "class", "variable", "text"
    pub file_pattern: Option<String>,
    pub max_results: Option<usize>,
}

#[async_trait]
impl AgentTool for CodeSearchTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Code Search"
    }

    fn description(&self) -> &'static str {
        "Search for code patterns, functions, classes, and related files in the repository"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "search".to_string(),
            "code_analysis".to_string(),
            "review".to_string(),
        ]
    }

    async fn execute(
        &self,
        _context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let args: CodeSearchArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        // This would implement actual code search functionality using _context.git_repo
        // For now, return a placeholder structure
        Ok(serde_json::json!({
            "query": args.query,
            "search_type": args.search_type,
            "results": [],
            "total_found": 0,
        }))
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "search_type": {
                    "type": "string",
                    "enum": ["function", "class", "variable", "text", "pattern"],
                    "description": "Type of search to perform"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "Optional file pattern to limit search scope"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return"
                }
            },
            "required": ["query", "search_type"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("CodeSearchTool: {}", self.name())
    }
}
