//! Git operations tool
//!
//! This tool provides Iris with the ability to perform Git operations
//! like examining commits, branches, file status, and repository information.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AgentTool;
use crate::agents::core::AgentContext;

/// Git operations tool for repository management and analysis
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
            id: "git".to_string(),
        }
    }

    /// Get detailed repository information
    fn get_repo_info(context: &AgentContext) -> Result<serde_json::Value> {
        let repo = &context.git_repo;
        let current_branch = repo.get_current_branch()?;
        let recent_commits = repo.get_recent_commits(5)?;

        Ok(serde_json::json!({
            "current_branch": current_branch,
            "repo_path": repo.repo_path(),
            "is_remote": repo.is_remote(),
            "remote_url": repo.get_remote_url(),
            "recent_commits": recent_commits.iter().map(|c| serde_json::json!({
                "hash": c.hash,
                "message": c.message,
                "author": c.author,
                "timestamp": c.timestamp
            })).collect::<Vec<_>>()
        }))
    }

    /// Get commit information for a specific commit
    fn get_commit_info(context: &AgentContext, commit_id: &str) -> Result<serde_json::Value> {
        let repo = &context.git_repo;
        let files = repo.get_commit_files(commit_id)?;
        let date = repo.get_commit_date(commit_id)?;

        Ok(serde_json::json!({
            "commit_id": commit_id,
            "date": date,
            "files": files.iter().map(|f| serde_json::json!({
                "path": f.path,
                "change_type": f.change_type,
                "analysis": f.analysis
            })).collect::<Vec<_>>()
        }))
    }

    /// Get file changes between commits or branches
    fn get_diff_info(context: &AgentContext, from: &str, to: &str) -> Result<serde_json::Value> {
        let repo = &context.git_repo;

        // Special case: if to_ref is "staged", we want to compare against staged changes
        if to == "staged" {
            log::debug!("📋 Git Operations: Handling 'staged' reference - from: {from}, to: {to}");
            let files_info = repo.extract_files_info(false)?;
            let staged_files = files_info.staged_files;

            // Handle any from_ref when to_ref is "staged"
            log::debug!(
                "📋 Git Operations: Returning {} staged files for {from}->staged diff",
                staged_files.len()
            );
            return Ok(serde_json::json!({
                "from": from,
                "to": "staged (current index)",
                "commits": [],
                "changed_files": staged_files.iter().map(|f| serde_json::json!({
                    "path": f.path,
                    "change_type": f.change_type,
                    "diff": f.diff,
                    "analysis": f.analysis
                })).collect::<Vec<_>>(),
                "total_changes": staged_files.len(),
                "note": format!("Shows staged changes ready for commit (compared to {})", from)
            }));
        }

        // Normal case: both from and to are valid Git references
        log::debug!("📋 Git Operations: Using normal diff - from: {from}, to: {to}");
        let files = repo.get_commit_range_files(from, to)?;
        let commits = repo.get_commits_for_pr(from, to)?;

        Ok(serde_json::json!({
            "from": from,
            "to": to,
            "commits": commits,
            "changed_files": files.iter().map(|f| serde_json::json!({
                "path": f.path,
                "change_type": f.change_type,
                "diff": f.diff,
                "analysis": f.analysis
            })).collect::<Vec<_>>(),
            "total_changes": files.len()
        }))
    }

    /// Get current repository status
    fn get_status(context: &AgentContext, include_unstaged: bool) -> Result<serde_json::Value> {
        let repo = &context.git_repo;

        let files_info = repo.extract_files_info(include_unstaged)?;
        let current_branch = repo.get_current_branch()?;

        let mut status = serde_json::json!({
            "current_branch": current_branch,
            "staged_files": files_info.staged_files.iter().map(|f| serde_json::json!({
                "path": f.path,
                "change_type": f.change_type,
                "content_excluded": f.content_excluded
            })).collect::<Vec<_>>()
        });

        if include_unstaged {
            let unstaged_files = repo.get_unstaged_files()?;
            status["unstaged_files"] = serde_json::json!(
                unstaged_files
                    .iter()
                    .map(|f| serde_json::json!({
                        "path": f.path,
                        "change_type": f.change_type
                    }))
                    .collect::<Vec<_>>()
            );
        }

        Ok(status)
    }

    /// Get project metadata based on changed files
    async fn get_project_metadata(
        context: &AgentContext,
        file_paths: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let repo = &context.git_repo;

        let files = if let Some(paths) = file_paths {
            paths
        } else {
            // Use currently staged files
            let files_info = repo.extract_files_info(false)?;
            files_info
                .staged_files
                .iter()
                .map(|f| f.path.clone())
                .collect()
        };

        let metadata = repo.get_project_metadata(&files).await?;

        Ok(serde_json::json!({
            "language": metadata.language,
            "framework": metadata.framework,
            "dependencies": metadata.dependencies,
            "version": metadata.version,
            "build_system": metadata.build_system,
            "test_framework": metadata.test_framework,
            "plugins": metadata.plugins
        }))
    }
}

#[derive(Deserialize, Serialize)]
pub struct GitArgs {
    pub operation: String, // "info", "commit_info", "diff", "status", "metadata"
    pub commit_id: Option<String>,
    pub from_ref: Option<String>,
    pub to_ref: Option<String>,
    pub include_unstaged: Option<bool>,
    pub file_paths: Option<Vec<String>>,
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
        "Perform Git operations like examining commits, branches, file status, and repository information. Use 'staged' as to_ref to see staged changes ready for commit."
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "git".to_string(),
            "version_control".to_string(),
            "commit_analysis".to_string(),
            "diff_analysis".to_string(),
            "repository_info".to_string(),
            "project_metadata".to_string(),
        ]
    }

    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let args: GitArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        match args.operation.as_str() {
            "info" => Self::get_repo_info(context),
            "commit_info" => {
                let commit_id = args.commit_id.ok_or_else(|| {
                    anyhow::anyhow!("commit_id required for commit_info operation")
                })?;
                Self::get_commit_info(context, &commit_id)
            }
            "diff" => {
                let from = args
                    .from_ref
                    .ok_or_else(|| anyhow::anyhow!("from_ref required for diff operation"))?;
                let to = args
                    .to_ref
                    .ok_or_else(|| anyhow::anyhow!("to_ref required for diff operation"))?;
                Self::get_diff_info(context, &from, &to)
            }
            "status" => {
                let include_unstaged = args.include_unstaged.unwrap_or(false);
                Self::get_status(context, include_unstaged)
            }
            "metadata" => Self::get_project_metadata(context, args.file_paths).await,
            _ => Err(anyhow::anyhow!("Unknown git operation: {}", args.operation)),
        }
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["info", "commit_info", "diff", "status", "metadata"],
                    "description": "Git operation to perform: 'info' for repo info, 'commit_info' for specific commit details, 'diff' for changes between refs, 'status' for current repo status, 'metadata' for project metadata"
                },
                "commit_id": {
                    "type": "string",
                    "description": "Commit hash for commit_info operation"
                },
                "from_ref": {
                    "type": "string",
                    "description": "Source reference for diff operation (commit hash, branch name, tag, or 'HEAD')"
                },
                "to_ref": {
                    "type": "string",
                    "description": "Target reference for diff operation (commit hash, branch name, tag, or 'staged' for current staged changes)"
                },
                "include_unstaged": {
                    "type": "boolean",
                    "description": "Include unstaged files in status operation",
                    "default": false
                },
                "file_paths": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Optional list of file paths for metadata operation"
                }
            },
            "required": ["operation"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("GitTool: {}", self.name())
    }
}
