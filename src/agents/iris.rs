//! Iris Agent - The unified AI agent for Git-Iris operations
//!
//! This agent can handle any Git workflow task through capability-based prompts
//! and multi-turn execution using Rig. One agent to rule them all! ✨

use anyhow::Result;
use async_trait::async_trait;
use rig::prelude::*;
use rig::completion::{Prompt, ToolDefinition};
use rig::providers::openai;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

use crate::config::Config;

/// Streaming callback trait for backward compatibility with LLM service
#[async_trait::async_trait]
pub trait StreamingCallback: Send + Sync {
    async fn on_chunk(&self, chunk: &str, tokens: Option<crate::agents::status::TokenMetrics>) -> Result<()>;
    async fn on_complete(&self, full_response: &str, final_tokens: crate::agents::status::TokenMetrics) -> Result<()>;
    async fn on_error(&self, error: &anyhow::Error) -> Result<()>;
    async fn on_status_update(&self, message: &str) -> Result<()>;
}

/// Task capability definition loaded from TOML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCapability {
    pub name: String,
    pub description: String,
    pub task_prompt: String,
}

/// Simple Git status tool for Rig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusTool;

#[derive(Debug, Error)]
#[error("Git error")]
pub struct GitError;

impl Tool for GitStatusTool {
    const NAME: &'static str = "git_status";
    type Error = GitError;
    type Args = ();
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_status",
            "description": "Get current Git repository status including staged and unstaged files",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        })).expect("Valid tool definition")
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|_| GitError)?;
            
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Simple Git diff tool for Rig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffTool;

#[derive(Deserialize)]
pub struct GitDiffArgs {
    #[serde(default)]
    pub staged: bool,
}

impl Tool for GitDiffTool {
    const NAME: &'static str = "git_diff";
    type Error = GitError;
    type Args = GitDiffArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_diff",
            "description": "Get Git diff output for staged or unstaged changes",
            "parameters": {
                "type": "object",
                "properties": {
                    "staged": {
                        "type": "boolean",
                        "description": "Get staged changes (--cached) instead of unstaged changes",
                        "default": false
                    }
                }
            }
        })).expect("Valid tool definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        use std::process::Command;
        
        let mut cmd = Command::new("git");
        cmd.arg("diff");
        
        if args.staged {
            cmd.arg("--cached");
        }
        
        let output = cmd.output().map_err(|_| GitError)?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// File reading tool for Rig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReaderTool;

#[derive(Deserialize)]
pub struct FileReaderArgs {
    pub path: String,
}

impl Tool for FileReaderTool {
    const NAME: &'static str = "read_file";
    type Error = GitError;
    type Args = FileReaderArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "read_file",
            "description": "Read the contents of a file in the repository",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }
        })).expect("Valid tool definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let content = tokio::fs::read_to_string(&args.path).await.map_err(|_| GitError)?;
        Ok(content)
    }
}

/// The unified Iris agent that handles all Git workflow tasks using Rig's multi-turn execution
pub struct IrisAgent {
    agent: rig::agent::Agent<openai::CompletionModel>,
    capabilities: Vec<TaskCapability>,
}

impl IrisAgent {
    /// Create a new IrisAgent with the specified configuration
    pub async fn new(config: &Config) -> Result<Self> {
        // Load task capabilities from TOML files
        let capabilities = Self::load_capabilities().await?;
        
        // Get provider config
        let provider_config = config.get_provider_config(&config.default_provider)
            .ok_or_else(|| anyhow::anyhow!("No configuration for provider: {}", config.default_provider))?;
        
        // Create the appropriate client based on provider
        let agent = match config.default_provider.as_str() {
            "openai" => {
                let client = openai::Client::from_env();
                client
                    .agent(&provider_config.model)
                    .preamble(&Self::create_system_prompt())
                    .tool(GitStatusTool)
                    .tool(GitDiffTool)
                    .tool(FileReaderTool)
                    .max_tokens(provider_config.get_token_limit().unwrap_or(4000) as u64)
                    .temperature(0.1)
                    .build()
            }
            "anthropic" => {
                // We'll add Anthropic support later - for now, fall back to OpenAI
                let client = openai::Client::from_env();
                client
                    .agent(&provider_config.model)
                    .preamble(&Self::create_system_prompt())
                    .tool(GitStatusTool)
                    .tool(GitDiffTool)
                    .tool(FileReaderTool)
                    .max_tokens(provider_config.get_token_limit().unwrap_or(4000) as u64)
                    .temperature(0.1)
                    .build()
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported provider: {}", config.default_provider));
            }
        };

        Ok(Self {
            agent,
            capabilities,
        })
    }

    /// Execute a task using multi-turn reasoning
    pub async fn execute_task(&self, task_type: &str, user_request: &str) -> Result<String> {
        // Find the capability for this task type
        let capability = self.capabilities
            .iter()
            .find(|c| c.name == task_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown task type: {}", task_type))?;

        // Combine the capability prompt with the user request
        let full_prompt = format!(
            "{}\n\nUser Request: {}\n\nPlease execute this task step by step, using the available tools as needed.",
            capability.task_prompt,
            user_request
        );

        // Use multi-turn execution - the agent will iteratively call tools until completion
        let result = self.agent
            .prompt(&full_prompt)
            .multi_turn(20)  // Allow up to 20 turns of tool calling
            .await
            .map_err(|e| anyhow::anyhow!("Agent execution failed: {}", e))?;

        Ok(result)
    }

    /// Execute a task with streaming output (for real-time feedback)
    pub async fn execute_task_streaming(&self, task_type: &str, user_request: &str) -> Result<String> {
        // Find the capability for this task type
        let capability = self.capabilities
            .iter()
            .find(|c| c.name == task_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown task type: {}", task_type))?;

        let full_prompt = format!(
            "{}\n\nUser Request: {}\n\nPlease execute this task step by step, using the available tools as needed.",
            capability.task_prompt,
            user_request
        );

        // For now, use regular execution. In the future, we can implement streaming
        // using Rig's streaming capabilities from the examples
        self.agent
            .prompt(&full_prompt)
            .multi_turn(20)
            .await
            .map_err(|e| anyhow::anyhow!("Agent execution failed: {}", e))
    }

    /// Create the main system prompt for the Iris agent
    fn create_system_prompt() -> String {
        r#"You are Iris, an intelligent Git workflow assistant designed to help developers with repository tasks.

## Core Capabilities
You can perform various Git-related tasks including:
- Generating commit messages from staged changes
- Performing comprehensive code reviews
- Creating changelogs from Git history
- Generating pull request descriptions
- Analyzing code quality and structure

## Tool Usage Guidelines
1. **Always use tools** - Don't try to guess or assume file contents, Git state, or repository structure
2. **Read before writing** - Use file analysis and Git tools to understand the current state
3. **Be thorough** - Gather sufficient context before making recommendations
4. **Iterate as needed** - Use multiple tool calls to build a complete understanding

## Multi-turn Execution
You can call tools multiple times in sequence to:
- First understand the repository state
- Analyze relevant files and changes
- Gather additional context as needed
- Provide comprehensive, accurate responses

## Response Style
- Be concise but thorough
- Focus on actionable insights
- Use clear, developer-friendly language
- Structure responses with headings when appropriate

Execute tasks step by step, using tools to gather accurate information about the repository state and changes."#.to_string()
    }

    /// Load task capabilities from TOML files
    async fn load_capabilities() -> Result<Vec<TaskCapability>> {
        let capabilities_dir = PathBuf::from("src/agents/capabilities");
        let mut capabilities = Vec::new();

        let mut entries = fs::read_dir(&capabilities_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path).await?;
                let capability = Self::parse_capability(&content, &path)?;
                capabilities.push(capability);
            }
        }

        Ok(capabilities)
    }

    /// Parse a capability from TOML content
    fn parse_capability(content: &str, path: &PathBuf) -> Result<TaskCapability> {
        let parsed: toml::Value = toml::from_str(content)?;
        
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let description = parsed.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let task_prompt = parsed.get("task_prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(TaskCapability {
            name,
            description,
            task_prompt,
        })
    }

    /// Get available task capabilities
    pub fn capabilities(&self) -> &[TaskCapability] {
        &self.capabilities
    }

    /// Get a specific capability by name
    pub fn get_capability(&self, name: &str) -> Option<&TaskCapability> {
        self.capabilities.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_iris_agent_creation() {
        let (config, _temp_dir) = create_test_config().await;
        
        // This test requires API keys to be set, so we'll skip if not available
        if std::env::var("OPENAI_API_KEY").is_err() {
            return;
        }

        let agent = IrisAgent::new(&config).await;
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_capability_loading() {
        let capabilities = IrisAgent::load_capabilities().await.unwrap();
        assert!(!capabilities.is_empty());
        
        // Should have at least our basic capabilities
        let capability_names: Vec<&str> = capabilities.iter().map(|c| c.name.as_str()).collect();
        assert!(capability_names.contains(&"commit"));
        assert!(capability_names.contains(&"review"));
    }

    #[tokio::test]
    async fn test_capability_retrieval() {
        let capabilities = IrisAgent::load_capabilities().await.unwrap();
        
        // Find a commit capability
        let commit_capability = capabilities.iter().find(|c| c.name == "commit");
        assert!(commit_capability.is_some());
        assert_eq!(commit_capability.unwrap().name, "commit");
    }
}
