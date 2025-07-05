//! Git repository operations tool
//!
//! This tool provides Iris with the ability to interact with Git repositories,
//! including getting diffs, status, commit history, and file lists.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AgentTool;
use crate::agents::core::AgentContext;
use crate::log_debug;

/// Git repository operations tool
pub struct GitTool {
    id: String,
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitTool {
    pub fn new() -> Self {
        Self {
            id: "git_operations".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct GitToolArgs {
    pub operation: String,
    pub path: Option<String>,
    pub commit_range: Option<String>,
}

#[async_trait]
impl AgentTool for GitTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Git Operations"
    }

    fn description(&self) -> &'static str {
        "Perform git repository operations like getting diff, status, log, etc."
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "git".to_string(),
            "commit".to_string(),
            "review".to_string(),
        ]
    }

    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        log_debug!("🔧 GitTool executing with params: {:?}", params);

        let args: GitToolArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        log_debug!("⚙️  GitTool operation: {}", args.operation);
        let repo = &context.git_repo;

        match args.operation.as_str() {
            "diff" => {
                log_debug!("📄 GitTool: Getting staged files with diffs");
                // Get the actual git context which includes diffs
                let git_context = repo.get_git_info(&context.config).await?;
                let combined_diff = git_context
                    .staged_files
                    .iter()
                    .map(|f| format!("{}:\n{}", f.path, f.diff))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                log_debug!(
                    "✅ GitTool: Diff retrieved, {} staged files, {} total chars",
                    git_context.staged_files.len(),
                    combined_diff.len()
                );

                Ok(serde_json::json!({
                    "operation": "diff",
                    "content": combined_diff,
                }))
            }
            "status" => {
                log_debug!("📊 GitTool: Getting repository status");
                let files = repo.get_unstaged_files()?;
                log_debug!(
                    "✅ GitTool: Status retrieved, {} unstaged files",
                    files.len()
                );

                Ok(serde_json::json!({
                    "operation": "status",
                    "content": files,
                }))
            }
            "log" => {
                log_debug!("📜 GitTool: Getting recent commit history (10 commits)");
                let commits = repo.get_recent_commits(10)?;
                log_debug!(
                    "✅ GitTool: Commit history retrieved, {} commits found",
                    commits.len()
                );

                Ok(serde_json::json!({
                    "operation": "log",
                    "content": commits,
                }))
            }
            "files" => {
                log_debug!("📂 GitTool: Getting list of changed files");
                let git_context = repo.get_git_info(&context.config).await?;
                let files: Vec<String> = git_context
                    .staged_files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect();

                log_debug!(
                    "✅ GitTool: File list retrieved, {} files: {:?}",
                    files.len(),
                    files
                );

                Ok(serde_json::json!({
                    "operation": "files",
                    "content": files,
                }))
            }
            _ => {
                log_debug!("❌ GitTool: Unknown operation: {}", args.operation);
                Err(anyhow::anyhow!("Unknown git operation: {}", args.operation))
            }
        }
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["diff", "status", "log", "files"],
                    "description": "The git operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "Optional path to limit operation scope"
                },
                "commit_range": {
                    "type": "string",
                    "description": "Optional commit range for diff operation (e.g., 'HEAD~1..HEAD')"
                }
            },
            "required": ["operation"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("GitTool: {}", self.name())
    }
}
