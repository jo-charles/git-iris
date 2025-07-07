//! Iris workspace tool
//!
//! This tool provides Iris with her own personal workspace for taking notes,
//! creating task lists, and managing her workflow during complex operations.
//!
//! Iris can use this tool to:
//! - Take notes on discoveries and insights
//! - Create and manage task lists
//! - Track progress through complex workflows
//! - Build knowledge across tool interactions

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::agents::core::AgentContext;

/// Iris's workspace for notes, tasks, and workflow management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IrisWorkspace {
    pub notes: Vec<WorkspaceNote>,
    pub tasks: Vec<WorkspaceTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceNote {
    pub id: String,
    pub content: String,
    pub timestamp: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTask {
    pub id: String,
    pub description: String,
    pub status: String,   // "pending", "in_progress", "completed", "blocked"
    pub priority: String, // "low", "medium", "high", "critical"
    pub created: String,
    pub updated: String,
    pub notes: Vec<String>,
}

impl IrisWorkspace {
    /// Add a note
    pub fn add_note(&mut self, content: String, tags: Vec<String>) -> String {
        let id = format!("note_{}", self.notes.len() + 1);
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        self.notes.push(WorkspaceNote {
            id: id.clone(),
            content,
            timestamp,
            tags,
        });

        id
    }

    /// Add a task
    pub fn add_task(&mut self, description: String, priority: String) -> String {
        let id = format!("task_{}", self.tasks.len() + 1);
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        self.tasks.push(WorkspaceTask {
            id: id.clone(),
            description,
            status: "pending".to_string(),
            priority,
            created: timestamp.clone(),
            updated: timestamp,
            notes: Vec::new(),
        });

        id
    }

    /// Update task status and add a note
    pub fn update_task(
        &mut self,
        task_id: &str,
        status: Option<String>,
        note: Option<String>,
    ) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if let Some(new_status) = status {
                task.status = new_status;
            }
            if let Some(task_note) = note {
                task.notes.push(task_note);
            }
            task.updated = chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string();
            true
        } else {
            false
        }
    }

    /// Get workspace summary
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();

        // Task summary
        let pending_tasks = self.tasks.iter().filter(|t| t.status == "pending").count();
        let in_progress_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == "in_progress")
            .count();
        let completed_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == "completed")
            .count();

        write!(
            summary,
            "**Iris Workspace Summary**\n\
            Tasks: {} pending, {} in progress, {} completed\n\
            Notes: {} total\n\n",
            pending_tasks,
            in_progress_tasks,
            completed_tasks,
            self.notes.len()
        )
        .unwrap();

        // Active tasks
        if in_progress_tasks > 0 || pending_tasks > 0 {
            summary.push_str("**Active Tasks:**\n");
            for task in &self.tasks {
                if task.status == "in_progress" || task.status == "pending" {
                    writeln!(
                        summary,
                        "- [{}] {} ({})",
                        task.status.to_uppercase(),
                        task.description,
                        task.priority
                    )
                    .unwrap();
                }
            }
            summary.push('\n');
        }

        // Recent notes
        if !self.notes.is_empty() {
            summary.push_str("**Recent Notes:**\n");
            for note in self.notes.iter().rev().take(3) {
                let preview = if note.content.len() > 60 {
                    format!("{}...", &note.content[..60])
                } else {
                    note.content.clone()
                };
                writeln!(summary, "- {preview}").unwrap();
            }
        }

        summary
    }

    /// Get all content formatted
    pub fn get_all_content(&self) -> String {
        let mut content = String::new();

        content.push_str("**IRIS WORKSPACE**\n\n");

        // All tasks
        if !self.tasks.is_empty() {
            content.push_str("**TASKS:**\n");
            for task in &self.tasks {
                write!(
                    content,
                    "ID: {} | Status: {} | Priority: {}\n\
                    Description: {}\n\
                    Created: {} | Updated: {}\n",
                    task.id,
                    task.status,
                    task.priority,
                    task.description,
                    task.created,
                    task.updated
                )
                .unwrap();

                if !task.notes.is_empty() {
                    content.push_str("Notes:\n");
                    for note in &task.notes {
                        writeln!(content, "  - {note}").unwrap();
                    }
                }
                content.push('\n');
            }
        }

        // All notes
        if !self.notes.is_empty() {
            content.push_str("**NOTES:**\n");
            for note in &self.notes {
                write!(
                    content,
                    "ID: {} | Time: {}\n\
                    Content: {}\n",
                    note.id, note.timestamp, note.content
                )
                .unwrap();

                if !note.tags.is_empty() {
                    writeln!(content, "Tags: {}", note.tags.join(", ")).unwrap();
                }
                content.push('\n');
            }
        }

        content
    }
}

/// Workspace management tool for file operations and workspace state
pub struct WorkspaceTool {
    id: String,
}

impl Default for WorkspaceTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceTool {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: "workspace".to_string(),
        }
    }

    /// List files and directories in a given path
    fn list_directory(context: &AgentContext, path: Option<String>) -> Result<serde_json::Value> {
        let repo_path = context.git_repo.repo_path();
        let target_path = if let Some(p) = path {
            repo_path.join(p)
        } else {
            repo_path.clone()
        };

        if !target_path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {:?}", target_path));
        }

        if !target_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Path is not a directory: {:?}",
                target_path
            ));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&target_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let path = entry.path();
            let relative_path = path.strip_prefix(repo_path).unwrap_or(&path);

            entries.push(serde_json::json!({
                "name": entry.file_name().to_string_lossy(),
                "path": relative_path.to_string_lossy(),
                "is_dir": metadata.is_dir(),
                "is_file": metadata.is_file(),
                "size": metadata.len(),
                "modified": metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
            }));
        }

        // Sort entries: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a["is_dir"].as_bool().unwrap_or(false);
            let b_is_dir = b["is_dir"].as_bool().unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a["name"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["name"].as_str().unwrap_or("")),
            }
        });

        Ok(serde_json::json!({
            "path": target_path.to_string_lossy(),
            "entries": entries,
            "total_count": entries.len()
        }))
    }

    /// Read file contents
    fn read_file(context: &AgentContext, file_path: &str) -> Result<serde_json::Value> {
        let repo_path = context.git_repo.repo_path();
        let target_path = repo_path.join(file_path);

        if !target_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", target_path));
        }

        if !target_path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {:?}", target_path));
        }

        let file_content = fs::read_to_string(&target_path)?;
        let metadata = fs::metadata(&target_path)?;

        Ok(serde_json::json!({
            "path": file_path,
            "content": file_content,
            "size": file_content.len(),
            "lines": file_content.lines().count(),
            "file_size": metadata.len(),
            "modified": metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
        }))
    }

    /// Find files by pattern
    fn find_files(
        context: &AgentContext,
        pattern: &str,
        directory: Option<String>,
    ) -> Result<serde_json::Value> {
        fn search_recursive(
            path: &Path,
            pattern: &str,
            repo_root: &Path,
            matches: &mut Vec<serde_json::Value>,
        ) -> Result<()> {
            if !path.is_dir() {
                return Ok(());
            }

            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                let filename = entry.file_name().to_string_lossy().to_lowercase();

                // Skip hidden files and directories
                if filename.starts_with('.') {
                    continue;
                }

                if entry_path.is_dir() {
                    // Recurse into subdirectories
                    search_recursive(&entry_path, pattern, repo_root, matches)?;
                } else if filename.contains(pattern) {
                    let relative_path = entry_path.strip_prefix(repo_root).unwrap_or(&entry_path);
                    let metadata = entry.metadata()?;

                    matches.push(serde_json::json!({
                        "name": entry.file_name().to_string_lossy(),
                        "path": relative_path.to_string_lossy(),
                        "size": metadata.len(),
                        "modified": metadata.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                    }));
                }
            }
            Ok(())
        }

        let repo_path = context.git_repo.repo_path();
        let search_path = if let Some(dir) = directory {
            repo_path.join(dir)
        } else {
            repo_path.clone()
        };

        let mut matches = Vec::new();
        let pattern_lower = pattern.to_lowercase();

        search_recursive(&search_path, &pattern_lower, repo_path, &mut matches)?;

        // Sort by relevance (exact matches first, then by name)
        matches.sort_by(|a, b| {
            let a_name = a["name"].as_str().unwrap_or("").to_lowercase();
            let b_name = b["name"].as_str().unwrap_or("").to_lowercase();

            let a_exact = a_name == pattern_lower;
            let b_exact = b_name == pattern_lower;

            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a_name.cmp(&b_name),
            }
        });

        Ok(serde_json::json!({
            "pattern": pattern,
            "search_path": search_path.to_string_lossy(),
            "matches": matches,
            "total_matches": matches.len()
        }))
    }

    /// Get workspace summary
    fn get_workspace_info(context: &AgentContext) -> Result<serde_json::Value> {
        fn count_recursive(
            path: &Path,
            file_count: &mut usize,
            dir_count: &mut usize,
        ) -> Result<()> {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                let filename = entry.file_name().to_string_lossy().to_string();

                // Skip .git and other hidden directories
                if filename.starts_with('.') {
                    continue;
                }

                if entry_path.is_dir() {
                    *dir_count += 1;
                    count_recursive(&entry_path, file_count, dir_count)?;
                } else {
                    *file_count += 1;
                }
            }
            Ok(())
        }

        let repo_path = context.git_repo.repo_path();
        let current_branch = context.git_repo.get_current_branch()?;

        // Count files and directories
        let mut file_count = 0;
        let mut dir_count = 0;
        count_recursive(repo_path, &mut file_count, &mut dir_count)?;

        // Get Git status info
        let recent_commits = context.git_repo.get_recent_commits(5)?;

        Ok(serde_json::json!({
            "workspace_path": repo_path.to_string_lossy(),
            "current_branch": current_branch,
            "file_count": file_count,
            "directory_count": dir_count,
            "recent_commits": recent_commits.len(),
            "latest_commit": recent_commits.first().map(|c| serde_json::json!({
                "hash": c.hash,
                "message": c.message,
                "author": c.author,
                "timestamp": c.timestamp
            }))
        }))
    }

    /// Check if a path exists
    fn path_exists(context: &AgentContext, path: &str) -> Result<serde_json::Value> {
        let repo_path = context.git_repo.repo_path();
        let target_path = repo_path.join(path);

        let exists = target_path.exists();
        let mut info = serde_json::json!({
            "path": path,
            "exists": exists
        });

        if exists {
            let metadata = fs::metadata(&target_path)?;
            info["is_file"] = serde_json::Value::Bool(metadata.is_file());
            info["is_dir"] = serde_json::Value::Bool(metadata.is_dir());
            info["size"] = serde_json::Value::Number(metadata.len().into());
        }

        Ok(info)
    }
}

#[derive(Deserialize, Serialize)]
pub struct WorkspaceArgs {
    pub operation: String, // "list", "read", "find", "info", "exists"
    pub path: Option<String>,
    pub pattern: Option<String>,
    pub directory: Option<String>,
}

#[async_trait]
impl crate::agents::tools::AgentTool for WorkspaceTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Workspace Management"
    }

    fn description(&self) -> &'static str {
        "Manage workspace state, file operations, directory navigation, and workspace context"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "workspace".to_string(),
            "file_management".to_string(),
            "directory_navigation".to_string(),
            "file_search".to_string(),
            "workspace_info".to_string(),
        ]
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["list", "read", "find", "info", "exists"],
                    "description": "Workspace operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory path for operations"
                },
                "pattern": {
                    "type": "string",
                    "description": "Search pattern for find operation"
                },
                "directory": {
                    "type": "string",
                    "description": "Directory to search in for find operation"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let args: WorkspaceArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        match args.operation.as_str() {
            "list" => Self::list_directory(context, args.path),
            "read" => {
                let path = args
                    .path
                    .ok_or_else(|| anyhow::anyhow!("path required for read operation"))?;
                Self::read_file(context, &path)
            }
            "find" => {
                let pattern = args
                    .pattern
                    .ok_or_else(|| anyhow::anyhow!("pattern required for find operation"))?;
                Self::find_files(context, &pattern, args.directory)
            }
            "info" => Self::get_workspace_info(context),
            "exists" => {
                let path = args
                    .path
                    .ok_or_else(|| anyhow::anyhow!("path required for exists operation"))?;
                Self::path_exists(context, &path)
            }
            _ => Err(anyhow::anyhow!(
                "Unknown workspace operation: {}",
                args.operation
            )),
        }
    }


}
