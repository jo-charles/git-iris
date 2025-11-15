//! Iris workspace tool for Rig
//!
//! This tool provides Iris with her own personal workspace for taking notes,
//! creating task lists, and managing her workflow during complex operations.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
#[error("Workspace error: {0}")]
pub struct WorkspaceError(String);

impl From<anyhow::Error> for WorkspaceError {
    fn from(err: anyhow::Error) -> Self {
        WorkspaceError(err.to_string())
    }
}

/// Workspace tool for Iris's note-taking and task management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(skip)]
    data: Arc<Mutex<WorkspaceData>>,
}

#[derive(Debug, Default)]
struct WorkspaceData {
    notes: Vec<String>,
    tasks: Vec<WorkspaceTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceTask {
    description: String,
    status: String,
    priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceArgs {
    pub action: String, // "add_note", "add_task", "update_task", "get_summary"
    pub content: Option<String>,
    pub priority: Option<String>,
    pub task_index: Option<usize>,
    pub status: Option<String>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(WorkspaceData::default())),
        }
    }
}

impl Tool for Workspace {
    const NAME: &'static str = "workspace";
    type Error = WorkspaceError;
    type Args = WorkspaceArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "workspace",
            "description": "Iris's personal workspace for notes and task management. Use this to track progress, take notes on findings, and manage complex workflows.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["add_note", "add_task", "update_task", "get_summary"],
                        "description": "Action to perform: add_note, add_task, update_task, or get_summary"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content for note or task description"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Priority level for tasks"
                    },
                    "task_index": {
                        "type": "integer",
                        "description": "Index of task to update (0-based)"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["pending", "in_progress", "completed", "blocked"],
                        "description": "New status for task updates"
                    }
                },
                "required": ["action"]
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut data = self.data.lock().map_err(|e| WorkspaceError(e.to_string()))?;

        let result = match args.action.as_str() {
            "add_note" => {
                let content = args.content.ok_or_else(|| WorkspaceError("Content required for add_note".to_string()))?;
                data.notes.push(content.clone());
                json!({
                    "success": true,
                    "message": "Note added successfully",
                    "note_count": data.notes.len()
                })
            }
            "add_task" => {
                let content = args.content.ok_or_else(|| WorkspaceError("Content required for add_task".to_string()))?;
                let priority = args.priority.unwrap_or_else(|| "medium".to_string());

                data.tasks.push(WorkspaceTask {
                    description: content,
                    status: "pending".to_string(),
                    priority,
                });

                json!({
                    "success": true,
                    "message": "Task added successfully",
                    "task_count": data.tasks.len()
                })
            }
            "update_task" => {
                let task_index = args.task_index.ok_or_else(|| WorkspaceError("task_index required for update_task".to_string()))?;
                let status = args.status.ok_or_else(|| WorkspaceError("status required for update_task".to_string()))?;

                if task_index >= data.tasks.len() {
                    return Err(WorkspaceError(format!("Task index {} out of range", task_index)));
                }

                data.tasks[task_index].status = status.clone();

                json!({
                    "success": true,
                    "message": format!("Task {} updated to {}", task_index, status),
                })
            }
            "get_summary" => {
                let pending = data.tasks.iter().filter(|t| t.status == "pending").count();
                let in_progress = data.tasks.iter().filter(|t| t.status == "in_progress").count();
                let completed = data.tasks.iter().filter(|t| t.status == "completed").count();

                json!({
                    "notes_count": data.notes.len(),
                    "tasks": {
                        "total": data.tasks.len(),
                        "pending": pending,
                        "in_progress": in_progress,
                        "completed": completed
                    },
                    "recent_notes": data.notes.iter().rev().take(3).collect::<Vec<_>>(),
                    "active_tasks": data.tasks.iter()
                        .filter(|t| t.status != "completed")
                        .map(|t| json!({
                            "description": t.description,
                            "status": t.status,
                            "priority": t.priority
                        }))
                        .collect::<Vec<_>>()
                })
            }
            _ => {
                return Err(WorkspaceError(format!("Unknown action: {}", args.action)));
            }
        };

        Ok(serde_json::to_string_pretty(&result).map_err(|e| WorkspaceError(e.to_string()))?)
    }
}
