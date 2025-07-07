use anyhow::Result;
use async_trait::async_trait;
use rig::providers::{anthropic, openai};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::llm::get_default_fast_model_for_provider;

/// Agent execution context containing shared state and configuration
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub config: Arc<Config>,
    pub git_repo: Arc<crate::git::GitRepo>,
    pub session_data: Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
}

impl AgentContext {
    pub fn new(config: Config, git_repo: crate::git::GitRepo) -> Self {
        Self {
            config: Arc::new(config),
            git_repo: Arc::new(git_repo),
            session_data: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_session_data(&self, key: &str) -> Option<serde_json::Value> {
        self.session_data.read().await.get(key).cloned()
    }

    pub async fn set_session_data(&self, key: String, value: serde_json::Value) {
        self.session_data.write().await.insert(key, value);
    }
}

/// Task execution result with optional follow-up actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub follow_up_tasks: Vec<String>,
    pub confidence: f32,
}

impl TaskResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
            follow_up_tasks: Vec::new(),
            confidence: 1.0,
        }
    }

    pub fn success_with_data(message: String, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            follow_up_tasks: Vec::new(),
            confidence: 1.0,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            follow_up_tasks: Vec::new(),
            confidence: 0.0,
        }
    }

    #[must_use]
    pub fn with_follow_up(mut self, tasks: Vec<String>) -> Self {
        self.follow_up_tasks = tasks;
        self
    }

    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Core trait for all Git-Iris agents
#[async_trait]
pub trait IrisAgent: Send + Sync {
    /// Get the agent's unique identifier
    fn id(&self) -> &str;

    /// Get the agent's display name
    fn name(&self) -> &str;

    /// Get the agent's description
    fn description(&self) -> &str;

    /// Get the capabilities this agent provides
    fn capabilities(&self) -> Vec<String>;

    /// Execute a task with the given context and parameters
    async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult>;

    /// Check if this agent can handle the given task
    fn can_handle_task(&self, task: &str) -> bool;

    /// Get the agent's priority for handling a task (higher = more likely to be chosen)
    fn task_priority(&self, task: &str) -> u8;

    /// Initialize the agent with configuration
    async fn initialize(&mut self, context: &AgentContext) -> Result<()>;

    /// Clean up resources when agent is shutting down
    async fn cleanup(&self) -> Result<()>;

    /// Cast to Any for dynamic downcast
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Agent configuration for different LLM providers
#[derive(Debug, Clone)]
pub enum AgentBackend {
    OpenAI {
        client: openai::Client,
        model: String,
        fast_model: String,
    },
    Anthropic {
        client: anthropic::Client,
        model: String,
        fast_model: String,
    },
}

impl AgentBackend {
    pub fn from_provider_name(provider_name: &str) -> Result<Self> {
        Err(anyhow::anyhow!(
            "Agent backend creation needs to be implemented for provider: {}",
            provider_name
        ))
    }

    pub fn from_config(config: &Config) -> Result<Self> {
        let provider_name = config
            .provider()
            .ok_or_else(|| anyhow::anyhow!("No provider configured"))?;

        let provider_config = config.providers.get(&provider_name).ok_or_else(|| {
            anyhow::anyhow!("Provider '{}' not found in configuration", provider_name)
        })?;

        if provider_config.api_key.is_empty() {
            return Err(anyhow::anyhow!(
                "API key not configured for provider '{}'",
                provider_name
            ));
        }

        match provider_name.to_lowercase().as_str() {
            "openai" => {
                let client = openai::Client::new(&provider_config.api_key);
                let model = if provider_config.model.is_empty() {
                    "gpt-4".to_string() // Default model
                } else {
                    provider_config.model.clone()
                };

                let fast_model = provider_config
                    .fast_model
                    .clone()
                    .unwrap_or_else(|| get_default_fast_model_for_provider("openai").to_string());

                Ok(AgentBackend::OpenAI {
                    client,
                    model,
                    fast_model,
                })
            }
            "anthropic" => {
                // Anthropic client constructor requires more parameters
                let client = anthropic::Client::new(
                    &provider_config.api_key,
                    "https://api.anthropic.com", // base_url
                    None,                        // betas
                    "2023-06-01",                // version
                );
                let model = if provider_config.model.is_empty() {
                    "claude-3-sonnet-20240229".to_string() // Default model
                } else {
                    provider_config.model.clone()
                };

                let fast_model = provider_config.fast_model.clone().unwrap_or_else(|| {
                    get_default_fast_model_for_provider("anthropic").to_string()
                });

                Ok(AgentBackend::Anthropic {
                    client,
                    model,
                    fast_model,
                })
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported provider for agent framework: {}",
                provider_name
            )),
        }
    }

    /// Get the primary model name
    pub fn primary_model(&self) -> &str {
        match self {
            AgentBackend::OpenAI { model, .. } => model,
            AgentBackend::Anthropic { model, .. } => model,
        }
    }

    /// Get the fast model name  
    pub fn fast_model(&self) -> &str {
        match self {
            AgentBackend::OpenAI { fast_model, .. } => fast_model,
            AgentBackend::Anthropic { fast_model, .. } => fast_model,
        }
    }
}

/// Base agent implementation with common functionality
pub struct BaseAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub backend: AgentBackend,
    pub tools: Vec<Arc<dyn crate::agents::tools::AgentTool>>,
}

impl BaseAgent {
    pub fn new(
        id: String,
        name: String,
        description: String,
        capabilities: Vec<String>,
        backend: AgentBackend,
    ) -> Self {
        Self {
            id,
            name,
            description,
            capabilities,
            backend,
            tools: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_tools(mut self, tools: Vec<Arc<dyn crate::agents::tools::AgentTool>>) -> Self {
        self.tools = tools;
        self
    }

    /// Create a Rig agent with the configured tools
    pub fn create_rig_agent(&self, preamble: &str) -> Result<String> {
        // For now, just return a placeholder until we properly implement Rig integration
        Ok(format!("Rig agent created with preamble: {preamble}"))
    }
}

/// Agent factory for creating different types of agents
pub struct AgentFactory {
    backend: AgentBackend,
    tool_registry: Arc<crate::agents::tools::ToolRegistry>,
}

impl AgentFactory {
    pub fn new(
        backend: AgentBackend,
        tool_registry: Arc<crate::agents::tools::ToolRegistry>,
    ) -> Self {
        Self {
            backend,
            tool_registry,
        }
    }

    pub fn create_iris_agent(&self) -> Result<Box<dyn IrisAgent>> {
        let tools = vec![
            self.tool_registry
                .get_tool("git")
                .expect("git tool should be available"),
            self.tool_registry
                .get_tool("file_analyzer")
                .expect("file_analyzer tool should be available"),
            self.tool_registry
                .get_tool("code_search")
                .expect("code_search tool should be available"),
        ];
        let agent = crate::agents::iris::IrisAgent::new(self.backend.clone(), tools);
        Ok(Box::new(agent))
    }
}
