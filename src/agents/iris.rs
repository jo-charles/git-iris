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
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;

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
    Review(Box<crate::commit::review::GeneratedReview>),
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
    use crate::agents::debug;

    debug::debug_section("JSON Extraction");

    let trimmed_response = response.trim();

    // First, try parsing the entire response as JSON (for well-behaved responses)
    if trimmed_response.starts_with('{')
        && serde_json::from_str::<serde_json::Value>(trimmed_response).is_ok()
    {
        debug::debug_context_management(
            "Response is pure JSON",
            &format!("{} characters", trimmed_response.len()),
        );
        return Ok(trimmed_response.to_string());
    }

    // Try to find JSON within markdown code blocks
    if let Some(start) = response.find("```json") {
        let content_start = start + "```json".len();
        // Find the closing ``` on its own line (to avoid matching ``` inside JSON strings)
        // First try with newline prefix to find standalone closing marker
        let json_end = if let Some(end) = response[content_start..].find("\n```") {
            // Found it with newline - the JSON ends before the newline
            end
        } else {
            // Fallback: try to find ``` at start of response section or end of string
            response[content_start..]
                .find("```")
                .unwrap_or(response.len() - content_start)
        };

        let json_content = &response[content_start..content_start + json_end];
        let trimmed = json_content.trim().to_string();

        debug::debug_context_management(
            "Found JSON in markdown code block",
            &format!("{} characters", trimmed.len()),
        );

        // Save extracted JSON for debugging
        if let Err(e) = std::fs::write("/tmp/iris_extracted.json", &trimmed) {
            debug::debug_warning(&format!("Failed to write extracted JSON: {}", e));
        }

        debug::debug_json_parse_attempt(&trimmed);
        return Ok(trimmed);
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
        debug::debug_json_parse_attempt(json_content);

        // Validate it's actually JSON by attempting to parse it
        let _: serde_json::Value = serde_json::from_str(json_content).map_err(|e| {
            debug::debug_json_parse_error(&format!(
                "Found JSON-like content but it's not valid JSON: {}",
                e
            ));
            anyhow::anyhow!("Found JSON-like content but it's not valid JSON")
        })?;

        debug::debug_context_management(
            "Found valid JSON object",
            &format!("{} characters", json_content.len()),
        );
        return Ok(json_content.to_string());
    }

    // If no JSON found, return error
    debug::debug_json_parse_error("No valid JSON found in response");
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
    /// Configuration for features like gitmoji, presets, etc.
    config: Option<crate::config::Config>,
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
            config: None,
        })
    }

    /// Build the actual agent for execution
    fn build_agent(&self) -> Agent<impl CompletionModel + 'static> {
        use crate::agents::debug_tool::DebugTool;

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
            .tool(DebugTool::new(GitStatus))
            .tool(DebugTool::new(GitDiff))
            .tool(DebugTool::new(GitLog))
            .tool(DebugTool::new(GitChangedFiles))
            .tool(DebugTool::new(FileAnalyzer))
            .tool(DebugTool::new(CodeSearch))
            .build();

        agent_builder
            .preamble(preamble)
            .max_tokens(16384) // Increased for complex structured outputs like PRs and release notes
            // Git tools - wrapped with debug observability
            .tool(DebugTool::new(GitStatus))
            .tool(DebugTool::new(GitDiff))
            .tool(DebugTool::new(GitLog))
            .tool(DebugTool::new(GitRepoInfo))
            .tool(DebugTool::new(GitChangedFiles))
            // Analysis and search tools
            .tool(DebugTool::new(FileAnalyzer))
            .tool(DebugTool::new(CodeSearch))
            // Workspace for Iris's notes and task management
            .tool(DebugTool::new(Workspace::new()))
            // Sub-agent delegation (Rig's built-in agent-as-tool!)
            .tool(sub_agent)
            .build()
    }

    /// Execute task using agent with tools and parse structured JSON response
    /// This is the core method that enables Iris to use tools and generate structured outputs
    async fn execute_with_agent<T>(&self, system_prompt: &str, user_prompt: &str) -> Result<T>
    where
        T: JsonSchema + for<'a> serde::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
    {
        use crate::agents::debug;
        use schemars::schema_for;

        debug::debug_phase_change(&format!("AGENT EXECUTION: {}", std::any::type_name::<T>()));

        // Build agent with all tools attached
        let agent = self.build_agent();
        debug::debug_context_management(
            "Agent built with tools",
            &format!("Provider: {}, Model: {}", self.provider, self.model),
        );

        // Create JSON schema for the response type
        let schema = schema_for!(T);
        let schema_json = serde_json::to_string_pretty(&schema)?;
        debug::debug_context_management(
            "JSON schema created",
            &format!("Type: {}", std::any::type_name::<T>()),
        );

        // Enhanced prompt that instructs Iris to use tools and respond with JSON
        let full_prompt = format!(
            "{system_prompt}\n\n{user_prompt}\n\n\
            === CRITICAL: RESPONSE FORMAT ===\n\
            After using the available tools to gather necessary information, you MUST respond with ONLY a valid JSON object.\n\n\
            REQUIRED JSON SCHEMA:\n\
            {schema_json}\n\n\
            CRITICAL INSTRUCTIONS:\n\
            - Return ONLY the raw JSON object - nothing else\n\
            - NO explanations before the JSON\n\
            - NO explanations after the JSON\n\
            - NO markdown code blocks (just raw JSON)\n\
            - NO preamble text like 'Here is the JSON:' or 'Let me generate:'\n\
            - Start your response with {{ and end with }}\n\
            - The JSON must be complete and valid\n\n\
            Your entire response should be ONLY the JSON object."
        );

        debug::debug_llm_request(&full_prompt, Some(16384));

        // Prompt the agent with retry logic for transient failures
        // Set multi_turn to allow the agent to call multiple tools (default is 0 = only 1 tool call)
        // For complex tasks like PRs and release notes, Iris may need many tool calls to analyze all changes
        // The agent knows when to stop, so we give it plenty of room (50 rounds)
        let timer = debug::DebugTimer::start("Agent prompt execution");

        let retry_strategy = ExponentialBackoff::from_millis(10).factor(2).take(2); // 2 attempts total: initial + 1 retry

        let response = Retry::spawn(retry_strategy, || async {
            debug::debug_context_management("LLM attempt", "Sending prompt to agent");
            agent.prompt(&full_prompt).multi_turn(50).await
        })
        .await?;

        timer.finish();

        debug::debug_llm_response(&response, std::time::Duration::from_secs(0), None);

        // Extract and parse JSON from the response
        let json_str = extract_json_from_response(&response)?;

        // Try to parse the JSON
        let result: T = serde_json::from_str(&json_str).map_err(|e| {
            debug::debug_json_parse_error(&format!("Failed to parse JSON response: {}", e));
            anyhow::anyhow!("Failed to parse JSON response: {}", e)
        })?;

        debug::debug_json_parse_success(std::any::type_name::<T>());

        Ok(result)
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
        let (mut system_prompt, output_type) = self.load_capability_config(capability).await?;

        // Inject gitmoji instructions if applicable
        if let Some(config) = &self.config {
            let is_conventional = config.instruction_preset == "conventional";

            // Add gitmoji for commit and PR capabilities if enabled
            if (capability == "commit" || capability == "pr")
                && config.use_gitmoji
                && !is_conventional
            {
                system_prompt.push_str("\n\n=== GITMOJI INSTRUCTIONS ===\n");
                system_prompt.push_str("Set the 'emoji' field to a single relevant gitmoji. ");
                system_prompt.push_str("DO NOT include the emoji in the 'message' or 'title' text - only set the 'emoji' field. ");
                system_prompt.push_str("Choose the most relevant emoji from this list:\n\n");
                system_prompt.push_str(&crate::gitmoji::get_gitmoji_list());
                system_prompt.push_str("\n\nThe emoji should match the primary type of change.");
            } else if is_conventional && (capability == "commit" || capability == "pr") {
                system_prompt.push_str("\n\n=== CONVENTIONAL COMMITS FORMAT ===\n");
                system_prompt.push_str("IMPORTANT: This uses Conventional Commits format. ");
                system_prompt
                    .push_str("DO NOT include any emojis in the commit message or PR title. ");
                system_prompt.push_str("The 'emoji' field should be null.");
            }
        }

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
                Ok(StructuredResponse::Review(Box::new(response)))
            }
            _ => {
                // Fallback to regular agent for unknown types
                let agent = self.build_agent();
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
        let agent = self.build_agent();
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

    /// Set configuration
    pub fn set_config(&mut self, config: crate::config::Config) {
        self.config = Some(config);
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
