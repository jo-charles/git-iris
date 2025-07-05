use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::agents::core::AgentContext;
use crate::log_debug;

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
    pub status: String, // "pending", "in_progress", "completed", "blocked"
    pub priority: String, // "low", "medium", "high", "critical"
    pub created: String,
    pub updated: String,
    pub notes: Vec<String>,
}

impl IrisWorkspace {
    /// Add a note
    pub fn add_note(&mut self, content: String, tags: Vec<String>) -> String {
        let id = format!("note_{}", self.notes.len() + 1);
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
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
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
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
    pub fn update_task(&mut self, task_id: &str, status: Option<String>, note: Option<String>) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if let Some(new_status) = status {
                task.status = new_status;
            }
            if let Some(task_note) = note {
                task.notes.push(task_note);
            }
            task.updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
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
        let in_progress_tasks = self.tasks.iter().filter(|t| t.status == "in_progress").count();
        let completed_tasks = self.tasks.iter().filter(|t| t.status == "completed").count();
        
        summary.push_str(&format!(
            "**Iris Workspace Summary**\n\
            Tasks: {} pending, {} in progress, {} completed\n\
            Notes: {} total\n\n",
            pending_tasks, in_progress_tasks, completed_tasks, self.notes.len()
        ));

        // Active tasks
        if in_progress_tasks > 0 || pending_tasks > 0 {
            summary.push_str("**Active Tasks:**\n");
            for task in &self.tasks {
                if task.status == "in_progress" || task.status == "pending" {
                    summary.push_str(&format!(
                        "- [{}] {} ({})\n", 
                        task.status.to_uppercase(), 
                        task.description, 
                        task.priority
                    ));
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
                summary.push_str(&format!("- {}\n", preview));
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
                content.push_str(&format!(
                    "ID: {} | Status: {} | Priority: {}\n\
                    Description: {}\n\
                    Created: {} | Updated: {}\n",
                    task.id, task.status, task.priority,
                    task.description,
                    task.created, task.updated
                ));
                
                if !task.notes.is_empty() {
                    content.push_str("Notes:\n");
                    for note in &task.notes {
                        content.push_str(&format!("  - {}\n", note));
                    }
                }
                content.push('\n');
            }
        }

        // All notes
        if !self.notes.is_empty() {
            content.push_str("**NOTES:**\n");
            for note in &self.notes {
                content.push_str(&format!(
                    "ID: {} | Time: {}\n\
                    Content: {}\n",
                    note.id, note.timestamp, note.content
                ));
                
                if !note.tags.is_empty() {
                    content.push_str(&format!("Tags: {}\n", note.tags.join(", ")));
                }
                content.push('\n');
            }
        }

        content
    }
}

/// Tool for Iris's workspace management - notes and tasks
pub struct WorkspaceTool {
    id: String,
    workspace: Arc<Mutex<IrisWorkspace>>,
}

impl WorkspaceTool {
    pub fn new() -> Self {
        Self {
            id: "workspace".to_string(),
            workspace: Arc::new(Mutex::new(IrisWorkspace::default())),
        }
    }

    /// Get access to the workspace for external use
    pub fn get_workspace(&self) -> Arc<Mutex<IrisWorkspace>> {
        Arc::clone(&self.workspace)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceInput {
    action: String, // "add_note", "add_task", "update_task", "get_summary", "get_all"
    content: Option<String>,
    task_id: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    tags: Option<Vec<String>>,
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceOutput {
    success: bool,
    message: String,
    workspace_summary: Option<String>,
    item_id: Option<String>,
}

#[async_trait]
impl crate::agents::tools::AgentTool for WorkspaceTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Iris Workspace"
    }

    fn description(&self) -> &str {
        "Iris's personal workspace for taking notes, creating task lists, and managing workflow. Use this to organize your thoughts, track progress, and plan next steps."
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "workspace".to_string(),
            "notes".to_string(),
            "tasks".to_string(),
            "planning".to_string(),
            "organization".to_string(),
        ]
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add_note", "add_task", "update_task", "get_summary", "get_all"],
                    "description": "Action: add_note, add_task, update_task, get_summary, get_all"
                },
                "content": {
                    "type": "string",
                    "description": "Note content or task description"
                },
                "task_id": {
                    "type": "string",
                    "description": "Task ID for updating (from previous task creation)"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "blocked"],
                    "description": "Task status for updates"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"],
                    "description": "Task priority (default: medium)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags for notes (optional)"
                },
                "note": {
                    "type": "string",
                    "description": "Additional note when updating a task"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        _context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let input: WorkspaceInput = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;
        
        let mut workspace = self.workspace
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock workspace: {}", e))?;

        match input.action.as_str() {
            "add_note" => {
                let content = input.content.ok_or_else(|| {
                    anyhow::anyhow!("Content is required for add_note action")
                })?;
                let tags = input.tags.unwrap_or_default();
                
                let note_id = workspace.add_note(content.clone(), tags);
                log_debug!("📝 Workspace: Added note {}: {}", note_id, content);

                Ok(serde_json::to_value(WorkspaceOutput {
                    success: true,
                    message: format!("Added note: {}", content),
                    workspace_summary: None,
                    item_id: Some(note_id),
                })?)
            }
            
            "add_task" => {
                let description = input.content.ok_or_else(|| {
                    anyhow::anyhow!("Content (task description) is required for add_task action")
                })?;
                let priority = input.priority.unwrap_or_else(|| "medium".to_string());
                
                let task_id = workspace.add_task(description.clone(), priority.clone());
                log_debug!("✅ Workspace: Added task {}: {} ({})", task_id, description, priority);

                Ok(serde_json::to_value(WorkspaceOutput {
                    success: true,
                    message: format!("Added task: {} ({})", description, priority),
                    workspace_summary: None,
                    item_id: Some(task_id),
                })?)
            }
            
            "update_task" => {
                let task_id = input.task_id.ok_or_else(|| {
                    anyhow::anyhow!("task_id is required for update_task action")
                })?;
                
                let updated = workspace.update_task(&task_id, input.status, input.note);
                
                if updated {
                    log_debug!("🔄 Workspace: Updated task {}", task_id);
                    Ok(serde_json::to_value(WorkspaceOutput {
                        success: true,
                        message: format!("Updated task {}", task_id),
                        workspace_summary: None,
                        item_id: Some(task_id),
                    })?)
                } else {
                    Err(anyhow::anyhow!("Task {} not found", task_id))
                }
            }
            
            "get_summary" => {
                let summary = workspace.get_summary();
                Ok(serde_json::to_value(WorkspaceOutput {
                    success: true,
                    message: "Workspace summary retrieved".to_string(),
                    workspace_summary: Some(summary),
                    item_id: None,
                })?)
            }
            
            "get_all" => {
                let all_content = workspace.get_all_content();
                Ok(serde_json::to_value(WorkspaceOutput {
                    success: true,
                    message: "All workspace content retrieved".to_string(),
                    workspace_summary: Some(all_content),
                    item_id: None,
                })?)
            }
            
            _ => Err(anyhow::anyhow!("Unknown action: {}", input.action)),
        }
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("WorkspaceTool: {}", self.name())
    }
}

impl Default for WorkspaceTool {
    fn default() -> Self {
        Self::new()
    }
}