//! Iris Agent - The unified AI agent for Git-Iris operations
//!
//! This agent can handle any Git workflow task through capability-based prompts
//! and multi-turn execution using Rig. One agent to rule them all! ✨

use anyhow::Result;
use rig::agent::Agent;
use rig::client::builder::DynClientBuilder;
use rig::completion::{CompletionModel, Prompt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tokio::fs;

use crate::agents::tools::{
    CodeSearch, FileAnalyzer, GitChangedFiles, GitDiff, GitLog, GitRepoInfo, GitStatus, Workspace,
};
// Added to ensure builder extension methods like `.max_tokens` are in scope

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

/// Unified response type that can hold any structured output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredResponse {
    CommitMessage(crate::commit::types::GeneratedMessage),
    PullRequest(crate::commit::types::GeneratedPullRequest),
    Changelog(crate::changes::models::ChangelogResponse),
    ReleaseNotes(crate::changes::models::ReleaseNotesResponse),
    Review(crate::commit::review::GeneratedReview),
    PlainText(String),
}

impl fmt::Display for StructuredResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StructuredResponse::CommitMessage(msg) => {
                if let Some(emoji) = &msg.emoji {
                    write!(f, "{} {}", emoji, msg.title)?;
                } else {
                    write!(f, "{}", msg.title)?;
                }
                if !msg.message.is_empty() {
                    write!(f, "\n\n{}", msg.message)?;
                }
                Ok(())
            }
            StructuredResponse::PullRequest(pr) => {
                write!(f, "# {}\n\n{}", pr.title, pr.description)
            }
            StructuredResponse::Changelog(cl) => {
                write!(f, "{}", cl.content())
            }
            StructuredResponse::ReleaseNotes(rn) => {
                write!(f, "{}", rn.content())
            }
            StructuredResponse::Review(review) => {
                write!(f, "{}", review.format())
            }
            StructuredResponse::PlainText(text) => {
                write!(f, "{text}")
            }
        }
    }
}

/// Extract JSON from a potentially verbose response that might contain explanations
fn extract_json_from_response(response: &str) -> Result<String> {
    // First try to find JSON within markdown code blocks
    if let Some(start) = response.find("```json") {
        if let Some(json_end) = response[start + 7..].find("```") {
            let json_content = &response[start + 7..start + 7 + json_end];
            return Ok(json_content.trim().to_string());
        }
    }
    
    // Look for JSON objects by finding { and matching }
    let mut brace_count = 0;
    let mut json_start = None;
    let mut json_end = None;
    
    for (i, ch) in response.char_indices() {
        match ch {
            '{' => {
                if brace_count == 0 {
                    json_start = Some(i);
                }
                brace_count += 1;
            }
            '}' => {
                brace_count -= 1;
                if brace_count == 0 && json_start.is_some() {
                    json_end = Some(i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    
    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_content = &response[start..end];
        // Validate it's actually JSON by attempting to parse it
        let _: serde_json::Value = serde_json::from_str(json_content)
            .map_err(|_| anyhow::anyhow!("Found JSON-like content but it's not valid JSON"))?;
        return Ok(json_content.to_string());
    }
    
    // If no JSON found, return error
    Err(anyhow::anyhow!("No valid JSON found in response"))
}

/// The unified Iris agent that can handle any Git-Iris task
pub struct IrisAgent {
    /// The underlying Rig client builder - we'll store the builder components instead
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
            "You are Iris, a helpful AI assistant specialized in Git operations and workflows.

You have access to Git tools and code analysis tools. You also have the ability to delegate tasks to specialized sub-agents.

**When to use sub-agents:**
- Large PR analysis: Delegate analysis of different directories/modules to parallel sub-agents
- Complex multi-part tasks: Break down into independent subtasks
- Avoid context overflow: When you have too much data to analyze at once

**How to use sub-agents:**
Use the 'delegate_task' tool to spawn a sub-agent with a specific focused task. The sub-agent will analyze its portion and return a summary for you to synthesize."
        );

        // Build a simple sub-agent that can be delegated to
        // This sub-agent has tools but cannot spawn more sub-agents (prevents recursion)
        let sub_agent = self
            .client_builder
            .agent(&self.provider, &self.model)
            .expect("Failed to create sub-agent")
            .preamble("You are a specialized analysis sub-agent. Complete your assigned task concisely and return a focused summary.")
            .max_tokens(4096)
            .tool(GitStatus)
            .tool(GitDiff)
            .tool(GitLog)
            .tool(GitChangedFiles)
            .tool(FileAnalyzer)
            .tool(CodeSearch)
            .build();

        let agent = agent_builder
            .preamble(preamble)
            .max_tokens(8192) // Required for Anthropic and good default for other providers
            // Git tools
            .tool(GitStatus)
            .tool(GitDiff)
            .tool(GitLog)
            .tool(GitRepoInfo)
            .tool(GitChangedFiles)
            // Analysis and search tools
            .tool(FileAnalyzer)
            .tool(CodeSearch)
            // Workspace for Iris's notes and task management
            .tool(Workspace::new())
            // Sub-agent delegation (Rig's built-in agent-as-tool!)
            .tool(sub_agent)
            .build();

        Ok(agent)
    }

    /// Execute task using agent with tools and parse structured JSON response
    /// This is the core method that enables Iris to use tools and generate structured outputs
    async fn execute_with_agent<T>(&self, system_prompt: &str, user_prompt: &str) -> Result<T>
    where
        T: JsonSchema
            + for<'a> serde::Deserialize<'a>
            + serde::Serialize
            + Send
            + Sync
            + 'static,
    {
        use schemars::schema_for;

        // Build agent with all tools attached
        let agent = self.build_agent()?;

        // Create JSON schema for the response type
        let schema = schema_for!(T);
        let schema_json = serde_json::to_string_pretty(&schema)?;

        // Enhanced prompt that instructs Iris to use tools and respond with JSON
        let full_prompt = format!(
            "{system_prompt}\n\n{user_prompt}\n\n\
            === RESPONSE FORMAT ===\n\
            After using the available tools to gather necessary information, respond with ONLY a valid JSON object that matches this schema:\n\n\
            {schema_json}\n\n\
            Return ONLY the raw JSON object. No explanations, no additional text, no markdown formatting - just the pure JSON response."
        );

        // Prompt the agent - it will use tools as needed
        // Set multi_turn to allow the agent to call multiple tools (default is 0 = only 1 tool call)
        // For complex tasks like PRs and release notes, Iris may need many tool calls to analyze all changes
        // The agent knows when to stop, so we give it plenty of room (50 rounds)
        let response = agent.prompt(&full_prompt).multi_turn(50).await?;

        // Extract and parse JSON from the response
        let json_str = extract_json_from_response(&response)?;
        serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))
    }

    /// Execute a task with the given capability and user prompt
    ///
    /// This now automatically uses structured output based on the capability type
    pub async fn execute_task(
        &mut self,
        capability: &str,
        user_prompt: &str,
    ) -> Result<StructuredResponse> {
        // Load the capability config to get both prompt and output type
        let (system_prompt, output_type) = self.load_capability_config(capability).await?;

        // Set the current capability
        self.current_capability = Some(capability.to_string());

        // Use agent with tools for all structured outputs
        // The agent will use tools as needed and respond with JSON
        match output_type.as_str() {
            "GeneratedMessage" => {
                let response = self
                    .execute_with_agent::<crate::commit::types::GeneratedMessage>(
                        &system_prompt,
                        user_prompt,
                    )
                    .await?;
                Ok(StructuredResponse::CommitMessage(response))
            }
            "GeneratedPullRequest" => {
                let response = self
                    .execute_with_agent::<crate::commit::types::GeneratedPullRequest>(
                        &system_prompt,
                        user_prompt,
                    )
                    .await?;
                Ok(StructuredResponse::PullRequest(response))
            }
            "ChangelogResponse" => {
                let response = self
                    .execute_with_agent::<crate::changes::models::ChangelogResponse>(
                        &system_prompt,
                        user_prompt,
                    )
                    .await?;
                Ok(StructuredResponse::Changelog(response))
            }
            "ReleaseNotesResponse" => {
                let response = self
                    .execute_with_agent::<crate::changes::models::ReleaseNotesResponse>(
                        &system_prompt,
                        user_prompt,
                    )
                    .await?;
                Ok(StructuredResponse::ReleaseNotes(response))
            }
            "GeneratedReview" => {
                let response = self
                    .execute_with_agent::<crate::commit::review::GeneratedReview>(
                        &system_prompt,
                        user_prompt,
                    )
                    .await?;
                Ok(StructuredResponse::Review(response))
            }
            _ => {
                // Fallback to regular agent for unknown types
                let agent = self.build_agent()?;
                let full_prompt = format!("{system_prompt}\n\n{user_prompt}");
                let response = agent.prompt(&full_prompt).await?;
                Ok(StructuredResponse::PlainText(response))
            }
        }
    }

    /// Load capability configuration from TOML file, returning both prompt and output type
    async fn load_capability_config(&self, capability: &str) -> Result<(String, String)> {
        let capability_file = format!("src/agents/capabilities/{capability}.toml");

        match fs::read_to_string(&capability_file).await {
            Ok(content) => {
                // Parse TOML to extract both task_prompt and output_type
                let parsed: toml::Value = toml::from_str(&content)?;

                let task_prompt = parsed
                    .get("task_prompt")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("No task_prompt found in capability file"))?;

                let output_type = parsed
                    .get("output_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("PlainText") // Default to plain text if not specified
                    .to_string();

                Ok((task_prompt.to_string(), output_type))
            }
            Err(_) => {
                // Return generic prompt and plain text output if capability file doesn't exist
                Ok((
                    format!(
                        "You are helping with a {capability} task. Use the available Git tools to assist the user."
                    ),
                    "PlainText".to_string(),
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