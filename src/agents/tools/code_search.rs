//! Code search tool
//!
//! This tool provides Iris with the ability to search for code patterns,
//! functions, classes, and related files in the repository.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::process::Command;

#[derive(Debug, thiserror::Error)]
#[error("Code search error: {0}")]
pub struct CodeSearchError(String);

impl From<anyhow::Error> for CodeSearchError {
    fn from(err: anyhow::Error) -> Self {
        CodeSearchError(err.to_string())
    }
}

impl From<std::io::Error> for CodeSearchError {
    fn from(err: std::io::Error) -> Self {
        CodeSearchError(err.to_string())
    }
}

/// Code search tool for finding related files and functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearch;

impl Default for CodeSearch {
    fn default() -> Self {
        Self
    }
}

impl CodeSearch {
    pub fn new() -> Self {
        Self
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

impl Tool for CodeSearch {
    const NAME: &'static str = "code_search";
    type Error = CodeSearchError;
    type Args = CodeSearchArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "code_search",
            "description": "Search for code patterns, functions, classes, and related files in the repository using ripgrep. Supports multiple search types and file filtering.",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query - can be function name, class name, variable, or text pattern"
                    },
                    "search_type": {
                        "type": "string",
                        "enum": ["function", "class", "variable", "text", "pattern"],
                        "description": "Type of search to perform (default: text)"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Optional file glob pattern to limit search scope (e.g., '*.rs', '*.js')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 20, max: 100)"
                    }
                },
                "required": ["query", "search_type"]
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(CodeSearchError::from)?;
        let max_results = args.max_results.unwrap_or(20);

        let results = Self::execute_ripgrep_search(
            &args.query,
            &current_dir,
            args.file_pattern.as_deref(),
            &args.search_type,
            max_results,
        )
        .map_err(CodeSearchError::from)?;

        let result = serde_json::json!({
            "query": args.query,
            "search_type": args.search_type,
            "results": results,
            "total_found": results.len(),
            "max_results": max_results,
        });

        Ok(serde_json::to_string_pretty(&result).map_err(|e| CodeSearchError(e.to_string()))?)
    }
}
