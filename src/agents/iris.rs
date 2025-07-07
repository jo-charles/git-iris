//! Iris Agent - The unified AI agent for Git-Iris operations
//!
//! This agent can handle any Git workflow task through capability-based prompts
//! and multi-turn execution using Rig. One agent to rule them all! ✨

use anyhow::Result;
use rig::agent::Agent;
use rig::client::builder::DynClientBuilder;
use rig::completion::{CompletionModel, Prompt};
use rig::prelude::*;
use std::collections::HashMap;
use tokio::fs;

use crate::agents::tools::{GitChangedFiles, GitDiff, GitLog, GitRepoInfo, GitStatus};

/// Type alias for a dynamic agent that can work with any completion model
pub type DynAgent = Agent<Box<dyn CompletionModel + Send + Sync>>;

/// Trait for streaming callback to handle real-time response processing
#[async_trait::async_trait]
pub trait StreamingCallback: Send + Sync {
    /// Called when a new chunk of text is received
    async fn on_chunk(
        &self,
        chunk: &str,
        tokens: Option<crate::agents::status::TokenMetrics>,
    ) -> Result<()>;

    /// Called when the response is complete
    async fn on_complete(
        &self,
        full_response: &str,
        final_tokens: crate::agents::status::TokenMetrics,
    ) -> Result<()>;

    /// Called when an error occurs
    async fn on_error(&self, error: &anyhow::Error) -> Result<()>;

    /// Called for status updates
    async fn on_status_update(&self, message: &str) -> Result<()>;
}

/// The unified Iris agent that can handle any Git-Iris task
pub struct IrisAgent {
    /// The underlying Rig agent - we'll store the builder components instead
    client_builder: DynClientBuilder,
    provider: String,
    model: String,
    /// Current capability/task being executed
    current_capability: Option<String>,
    /// Provider configuration
    provider_config: HashMap<String, String>,
    /// Custom preamble
    preamble: Option<String>,
}

impl IrisAgent {
    /// Create a new Iris agent with the given `DynClientBuilder` and provider configuration
    pub fn new(client_builder: DynClientBuilder, provider: &str, model: &str) -> Result<Self> {
        Ok(Self {
            client_builder,
            provider: provider.to_string(),
            model: model.to_string(),
            current_capability: None,
            provider_config: HashMap::new(),
            preamble: None,
        })
    }

    /// Build the actual agent for execution
    fn build_agent(&self) -> Result<Agent<impl CompletionModel + 'static>> {
        let agent_builder = self
            .client_builder
            .agent(&self.provider, &self.model)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to create agent builder for provider: {}",
                    self.provider
                )
            });

        let preamble = self.preamble.as_deref().unwrap_or(
            "You are Iris, a helpful AI assistant specialized in Git operations and workflows. You have access to Git tools to help users with their Git-related tasks."
        );

        let agent = agent_builder
            .preamble(preamble)
            .max_tokens(8192) // Required for Anthropic and good default for other providers
            .tool(GitStatus)
            .tool(GitDiff)
            .tool(GitLog)
            .tool(GitRepoInfo)
            .tool(GitChangedFiles)
            .build();

        Ok(agent)
    }

    /// Execute a task with the given capability and user prompt
    ///
    /// This uses Rig's multi-turn execution to allow the agent to use tools
    /// iteratively to complete complex tasks.
    pub async fn execute_task(&mut self, capability: &str, user_prompt: &str) -> Result<String> {
        // Load the capability prompt if available
        let system_prompt = self
            .load_capability_prompt(capability)
            .await
            .unwrap_or_else(|_| format!("Handle this Git-related task: {capability}"));

        // Set the current capability
        self.current_capability = Some(capability.to_string());

        // Create the full prompt with capability context
        let full_prompt = format!("System: {system_prompt}\n\nUser: {user_prompt}");

        // Build the agent
        let agent = self.build_agent()?;

        // Execute with multi-turn to allow tool usage
        let response = agent
            .prompt(&full_prompt)
            .multi_turn(20) // Allow up to 20 turns for complex tasks
            .await?;

        Ok(response)
    }

    /// Load capability-specific prompt from TOML file
    async fn load_capability_prompt(&self, capability: &str) -> Result<String> {
        let capability_file = format!("src/agents/capabilities/{capability}.toml");

        match fs::read_to_string(&capability_file).await {
            Ok(content) => {
                // Parse TOML to extract the system prompt
                let parsed: toml::Value = toml::from_str(&content)?;
                if let Some(prompt) = parsed.get("system_prompt").and_then(|v| v.as_str()) {
                    Ok(prompt.to_string())
                } else {
                    Err(anyhow::anyhow!("No system_prompt found in capability file"))
                }
            }
            Err(_) => {
                // Return a generic prompt if capability file doesn't exist
                Ok(format!(
                    "You are helping with a {capability} task. Use the available Git tools to assist the user."
                ))
            }
        }
    }

    /// Get the current capability being executed
    pub fn current_capability(&self) -> Option<&str> {
        self.current_capability.as_deref()
    }

    /// Simple single-turn execution for basic queries
    pub async fn chat(&self, message: &str) -> Result<String> {
        let agent = self.build_agent()?;
        let response = agent.prompt(message).await?;
        Ok(response)
    }

    /// Set the current capability
    pub fn set_capability(&mut self, capability: &str) {
        self.current_capability = Some(capability.to_string());
    }

    /// Get provider configuration
    pub fn provider_config(&self) -> &HashMap<String, String> {
        &self.provider_config
    }

    /// Set provider configuration
    pub fn set_provider_config(&mut self, config: HashMap<String, String>) {
        self.provider_config = config;
    }

    /// Set custom preamble
    pub fn set_preamble(&mut self, preamble: String) {
        self.preamble = Some(preamble);
    }
}

/// Builder for creating `IrisAgent` instances with different configurations
pub struct IrisAgentBuilder {
    client_builder: Option<DynClientBuilder>,
    provider: String,
    model: String,
    preamble: Option<String>,
}

impl IrisAgentBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            client_builder: None,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            preamble: None,
        }
    }

    /// Set the client builder (for provider configuration)
    pub fn with_client(mut self, client: DynClientBuilder) -> Self {
        self.client_builder = Some(client);
        self
    }

    /// Set the provider to use
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set a custom preamble
    pub fn with_preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = Some(preamble.into());
        self
    }

    /// Build the `IrisAgent`
    pub fn build(self) -> Result<IrisAgent> {
        let client_builder = self
            .client_builder
            .ok_or_else(|| anyhow::anyhow!("Client builder is required"))?;

        let mut agent = IrisAgent::new(client_builder, &self.provider, &self.model)?;

        // Apply custom preamble if provided
        if let Some(preamble) = self.preamble {
            agent.set_preamble(preamble);
        }

        Ok(agent)
    }
}

impl Default for IrisAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
