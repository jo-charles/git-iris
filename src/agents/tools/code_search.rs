//! Code search tool
//!
//! This tool provides Iris with the ability to search for code patterns,
//! functions, classes, and related files in the repository.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

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

    /// Execute a ripgrep search for patterns
    fn execute_ripgrep_search(
        query: &str,
        repo_path: &Path,
        file_pattern: Option<&str>,
        search_type: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut cmd = Command::new("rg");

        // Configure ripgrep based on search type
        match search_type {
            "function" => {
                cmd.args(["--type", "rust", "--type", "javascript", "--type", "python"]);
                cmd.args([
                    "-e",
                    &format!(r"fn\s+{query}|function\s+{query}|def\s+{query}"),
                ]);
            }
            "class" => {
                cmd.args(["--type", "rust", "--type", "javascript", "--type", "python"]);
                cmd.args(["-e", &format!(r"struct\s+{query}|class\s+{query}")]);
            }
            "variable" => {
                cmd.args(["-e", &format!(r"let\s+{query}|var\s+{query}|{query}\s*=")]);
            }
            "pattern" => {
                cmd.args(["-e", query]);
            }
            _ => {
                cmd.args(["-i", query]); // case-insensitive text search
            }
        }

        // Add file pattern if specified
        if let Some(pattern) = file_pattern {
            cmd.args(["-g", pattern]);
        }

        // Limit results and add context
        cmd.args(["-n", "--color", "never", "-A", "3", "-B", "1"]);
        cmd.current_dir(repo_path);

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut results = Vec::new();
        let mut current_file = String::new();
        let mut line_number = 0;
        let mut content_lines = Vec::new();

        for line in stdout.lines().take(max_results * 4) {
            // rough estimate with context
            if line.contains(':') && !line.starts_with('-') {
                // Parse file:line:content format
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                if parts.len() >= 3 {
                    let file_path = parts[0].to_string();
                    if let Ok(line_num) = parts[1].parse::<usize>() {
                        let content = parts[2].to_string();

                        if file_path != current_file && !current_file.is_empty() {
                            // Finalize previous result
                            results.push(SearchResult {
                                file_path: current_file.clone(),
                                line_number,
                                content: content_lines.join("\n"),
                                match_type: search_type.to_string(),
                                context_lines: content_lines.len(),
                            });
                            content_lines.clear();
                        }

                        current_file = file_path;
                        line_number = line_num;
                        content_lines.push(content);

                        if results.len() >= max_results {
                            break;
                        }
                    }
                }
            } else if !line.starts_with('-') && !current_file.is_empty() {
                // Context line
                content_lines.push(line.to_string());
            }
        }

        // Add final result
        if !current_file.is_empty() && results.len() < max_results {
            results.push(SearchResult {
                file_path: current_file,
                line_number,
                content: content_lines.join("\n"),
                match_type: search_type.to_string(),
                context_lines: content_lines.len(),
            });
        }

        Ok(results)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub match_type: String,
    pub context_lines: usize,
}

#[derive(Deserialize, Serialize)]
pub struct CodeSearchArgs {
    pub query: String,
    pub search_type: String, // "function", "class", "variable", "text", "pattern"
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
        "Search for code patterns, functions, classes, and related files in the repository using ripgrep"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "search".to_string(),
            "code_analysis".to_string(),
            "review".to_string(),
            "pattern_matching".to_string(),
        ]
    }

    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let args: CodeSearchArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        let max_results = args.max_results.unwrap_or(20);
        let repo_path = context.git_repo.repo_path();

        let results = Self::execute_ripgrep_search(
            &args.query,
            repo_path,
            args.file_pattern.as_deref(),
            &args.search_type,
            max_results,
        )?;

        Ok(serde_json::json!({
            "query": args.query,
            "search_type": args.search_type,
            "results": results,
            "total_found": results.len(),
            "max_results": max_results,
        }))
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query - can be function name, class name, variable, or text pattern"
                },
                "search_type": {
                    "type": "string",
                    "enum": ["function", "class", "variable", "text", "pattern"],
                    "description": "Type of search to perform",
                    "default": "text"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "Optional file glob pattern to limit search scope (e.g., '*.rs', '*.js')"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 20,
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["query", "search_type"]
        })
    }


}
