use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::git::GitRepo;

/// Unified Agent Backend Configuration  
#[derive(Debug, Clone)]
pub struct AgentBackend {
    pub provider_name: String,
    pub model_name: String,
    pub config: Config,
}

impl AgentBackend {
    pub fn new(provider_name: String, model_name: String, config: Config) -> Self {
        Self { 
            provider_name,
            model_name, 
            config 
        }
    }
}

/// Agent Context containing all necessary state for task execution
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub config: Arc<Config>,
    pub git_repo: Arc<GitRepo>,
    pub backend: AgentBackend,
}

impl AgentContext {
    pub fn new(config: Config, git_repo: GitRepo, backend: AgentBackend) -> Self {
        Self {
            config: Arc::new(config),
            git_repo: Arc::new(git_repo),
            backend,
        }
    }
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub confidence: f64,
    pub execution_time: Option<std::time::Duration>,
}

impl TaskResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
            confidence: 1.0,
            execution_time: None,
        }
    }

    pub fn success_with_data(message: String, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            confidence: 1.0,
            execution_time: None,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            confidence: 0.0,
            execution_time: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_execution_time(mut self, duration: std::time::Duration) -> Self {
        self.execution_time = Some(duration);
        self
    }
}

/// Task capability definition 
#[derive(Debug, Clone)]
pub struct TaskCapability {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub required_tools: Vec<String>,
    pub example_flow: Option<String>,
}

impl TaskCapability {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        system_prompt: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            system_prompt: system_prompt.into(),
            required_tools: Vec::new(),
            example_flow: None,
        }
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.required_tools = tools;
        self
    }

    pub fn with_example_flow(mut self, flow: impl Into<String>) -> Self {
        self.example_flow = Some(flow.into());
        self
    }
}

/// Unified Agent trait - simplified from the old complex system
#[async_trait]
pub trait UnifiedAgent: Send + Sync {
    /// Execute a task with the agent
    async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult>;

    /// Get available task capabilities
    fn get_capabilities(&self) -> Vec<TaskCapability>;

    /// Check if agent can handle specific task
    fn can_handle_task(&self, task: &str) -> bool {
        self.get_capabilities()
            .iter()
            .any(|cap| cap.name == task)
    }
}

/// Legacy trait for backward compatibility - will be removed
#[async_trait]
pub trait IrisAgent: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;

    async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult>;

    fn can_handle_task(&self, task: &str) -> bool;
    fn task_priority(&self, task: &str) -> u8;

    async fn initialize(&mut self, context: &AgentContext) -> Result<()>;
    async fn cleanup(&self) -> Result<()>;

    fn as_any(&self) -> &dyn std::any::Any;
}
