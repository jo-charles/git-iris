use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    agents::{
        core::{AgentBackend, AgentContext},
        executor::{AgentExecutor, TaskPriority, TaskRequest},
        iris::{IrisStreamingCallback, StreamingCallback},
        registry::AgentRegistry,
        tools::create_default_tool_registry,
    },
    common::CommonParams,
    config::Config,
    git::GitRepo,
};

/// Integration layer between Git-Iris services and the agent framework
pub struct AgentIntegration {
    registry: Arc<AgentRegistry>,
    executor: Arc<AgentExecutor>,
    context: AgentContext,
}

impl AgentIntegration {
    /// Create a new agent integration with the given configuration
    pub async fn new(config: Config, git_repo: GitRepo) -> Result<Self> {
        // Create agent context
        let context = AgentContext::new(config.clone(), git_repo);

        // Create agent backend from provider configuration
        let backend = Self::create_backend_from_config(&config)?;

        // Create tool registry and agent registry
        let _tool_registry = Arc::new(create_default_tool_registry());
        let registry = Arc::new(AgentRegistry::create_default(backend).await?);

        // Create executor
        let executor = Arc::new(
            AgentExecutor::new(registry.clone())
                .with_max_concurrent_tasks(config.performance.max_concurrent_tasks.unwrap_or(5))
                .with_default_timeout(std::time::Duration::from_secs(
                    config.performance.default_timeout_seconds.unwrap_or(300),
                )),
        );

        Ok(Self {
            registry,
            executor,
            context,
        })
    }

    /// Create agent backend from Git-Iris configuration
    fn create_backend_from_config(config: &Config) -> Result<AgentBackend> {
        AgentBackend::from_config(config)
    }

    /// Generate commit message using agent framework
    pub async fn generate_commit_message(
        &self,
        preset: Option<&str>,
        instructions: Option<&str>,
    ) -> Result<String> {
        let mut params = HashMap::new();

        if let Some(preset) = preset {
            params.insert(
                "preset".to_string(),
                serde_json::Value::String(preset.to_string()),
            );
        }

        if let Some(instructions) = instructions {
            params.insert(
                "instructions".to_string(),
                serde_json::Value::String(instructions.to_string()),
            );
        }

        self.generate_commit_message_with_params(params).await
    }

    /// Generate commit message with streaming support
    pub async fn generate_commit_message_streaming(
        &self,
        preset: Option<&str>,
        instructions: Option<&str>,
        callback: Option<&dyn StreamingCallback>,
    ) -> Result<String> {
        let mut params = HashMap::new();

        if let Some(preset) = preset {
            params.insert(
                "preset".to_string(),
                serde_json::Value::String(preset.to_string()),
            );
        }

        if let Some(instructions) = instructions {
            params.insert(
                "instructions".to_string(),
                serde_json::Value::String(instructions.to_string()),
            );
        }

        self.generate_commit_message_with_params_streaming(params, callback)
            .await
    }

    /// Generate commit message with custom parameters and streaming
    pub async fn generate_commit_message_with_params_streaming(
        &self,
        params: HashMap<String, serde_json::Value>,
        callback: Option<&dyn StreamingCallback>,
    ) -> Result<String> {
        // Use default callback if none provided
        let default_callback = IrisStreamingCallback::new(true);
        let callback = callback.unwrap_or(&default_callback);

        // Find the Iris agent
        let iris_agent = self
            .registry
            .find_agent_for_task("generate_commit_message")
            .await
            .ok_or_else(|| anyhow::anyhow!("No Iris agent found for commit message generation"))?;

        // Downcast to IrisAgent to access streaming method
        if let Some(iris) = iris_agent
            .as_any()
            .downcast_ref::<crate::agents::iris::IrisAgent>()
        {
            let result = iris
                .generate_commit_message_streaming(&self.context, &params, callback)
                .await?;

            if result.success {
                if let Some(data) = result.data {
                    // Try to parse as GeneratedMessage first (Iris agent format)
                    if let Ok(generated_message) = serde_json::from_value::<
                        crate::commit::types::GeneratedMessage,
                    >(data.clone())
                    {
                        // Format the message using the existing formatter
                        let formatted =
                            crate::commit::types::format_commit_message(&generated_message);
                        return Ok(formatted);
                    }
                    // Fallback to old format for backward compatibility
                    if let Some(message) = data.get("commit_message").and_then(|v| v.as_str()) {
                        return Ok(message.to_string());
                    }
                }
                Ok(result.message)
            } else {
                Err(anyhow::anyhow!(
                    "Commit message generation failed: {}",
                    result.message
                ))
            }
        } else {
            Err(anyhow::anyhow!("Failed to access Iris agent for streaming"))
        }
    }

    /// Generate commit message with custom parameters
    pub async fn generate_commit_message_with_params(
        &self,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let request = TaskRequest::new("generate_commit_message".to_string())
            .with_params(params)
            .with_priority(TaskPriority::High)
            .with_timeout(std::time::Duration::from_secs(120));

        let result = self
            .executor
            .execute_task_immediately(request, self.context.clone())
            .await?;

        if result.result.success {
            if let Some(data) = result.result.data {
                // Try to parse as GeneratedMessage first (Iris agent format)
                if let Ok(generated_message) =
                    serde_json::from_value::<crate::commit::types::GeneratedMessage>(data.clone())
                {
                    // Format the message using the existing formatter
                    let formatted = crate::commit::types::format_commit_message(&generated_message);
                    return Ok(formatted);
                }
                // Fallback to old format for backward compatibility
                if let Some(message) = data.get("commit_message").and_then(|v| v.as_str()) {
                    return Ok(message.to_string());
                }
            }
            Ok(result.result.message)
        } else {
            Err(anyhow::anyhow!(
                "Commit message generation failed: {}",
                result.result.message
            ))
        }
    }

    /// Review code changes using agent framework
    pub async fn review_changes(
        &self,
        commit_range: Option<&str>,
        focus_areas: Option<Vec<&str>>,
    ) -> Result<serde_json::Value> {
        let mut params = HashMap::new();

        if let Some(range) = commit_range {
            params.insert(
                "commit_range".to_string(),
                serde_json::Value::String(range.to_string()),
            );
        }

        if let Some(areas) = focus_areas {
            let areas_json: Vec<serde_json::Value> = areas
                .iter()
                .map(|s| serde_json::Value::String((*s).to_string()))
                .collect();
            params.insert(
                "focus_areas".to_string(),
                serde_json::Value::Array(areas_json),
            );
        }

        let request = TaskRequest::new("review_code".to_string())
            .with_params(params)
            .with_priority(TaskPriority::Normal)
            .with_timeout(std::time::Duration::from_secs(300));

        let result = self
            .executor
            .execute_task_immediately(request, self.context.clone())
            .await?;

        if result.result.success {
            result
                .result
                .data
                .ok_or_else(|| anyhow::anyhow!("No review data returned"))
        } else {
            Err(anyhow::anyhow!(
                "Code review failed: {}",
                result.result.message
            ))
        }
    }

    /// Generate changelog using agent framework
    pub async fn generate_changelog(
        &self,
        version: &str,
        from_tag: Option<&str>,
        to_tag: Option<&str>,
        format: Option<&str>,
    ) -> Result<String> {
        let mut params = HashMap::new();
        params.insert(
            "version".to_string(),
            serde_json::Value::String(version.to_string()),
        );

        if let Some(from) = from_tag {
            params.insert(
                "from_tag".to_string(),
                serde_json::Value::String(from.to_string()),
            );
        }

        if let Some(to) = to_tag {
            params.insert(
                "to_tag".to_string(),
                serde_json::Value::String(to.to_string()),
            );
        }

        if let Some(fmt) = format {
            params.insert(
                "format".to_string(),
                serde_json::Value::String(fmt.to_string()),
            );
        }

        let request = TaskRequest::new("generate_changelog".to_string())
            .with_params(params)
            .with_priority(TaskPriority::Normal)
            .with_timeout(std::time::Duration::from_secs(180));

        let result = self
            .executor
            .execute_task_immediately(request, self.context.clone())
            .await?;

        if result.result.success {
            if let Some(data) = result.result.data {
                if let Some(formatted) = data.get("formatted").and_then(|v| v.as_str()) {
                    return Ok(formatted.to_string());
                }
            }
            Ok(result.result.message)
        } else {
            Err(anyhow::anyhow!(
                "Changelog generation failed: {}",
                result.result.message
            ))
        }
    }

    /// Generate release notes using agent framework
    pub async fn generate_release_notes(
        &self,
        version: &str,
        from_tag: Option<&str>,
        to_tag: Option<&str>,
    ) -> Result<String> {
        let mut params = HashMap::new();
        params.insert(
            "version".to_string(),
            serde_json::Value::String(version.to_string()),
        );

        if let Some(from) = from_tag {
            params.insert(
                "from_tag".to_string(),
                serde_json::Value::String(from.to_string()),
            );
        }

        if let Some(to) = to_tag {
            params.insert(
                "to_tag".to_string(),
                serde_json::Value::String(to.to_string()),
            );
        }

        let request = TaskRequest::new("generate_release_notes".to_string())
            .with_params(params)
            .with_priority(TaskPriority::Normal)
            .with_timeout(std::time::Duration::from_secs(240));

        let result = self
            .executor
            .execute_task_immediately(request, self.context.clone())
            .await?;

        if result.result.success {
            if let Some(data) = result.result.data {
                if let Some(formatted) = data.get("formatted").and_then(|v| v.as_str()) {
                    return Ok(formatted.to_string());
                }
            }
            Ok(result.result.message)
        } else {
            Err(anyhow::anyhow!(
                "Release notes generation failed: {}",
                result.result.message
            ))
        }
    }

    /// Submit a task for asynchronous execution
    pub async fn submit_task_async(
        &self,
        task_type: &str,
        params: HashMap<String, serde_json::Value>,
        priority: TaskPriority,
    ) -> Result<String> {
        let request = TaskRequest::new(task_type.to_string())
            .with_params(params)
            .with_priority(priority)
            .with_retries(2);

        self.executor
            .submit_task(request, self.context.clone())
            .await
    }

    /// Get executor statistics
    pub async fn get_executor_statistics(&self) -> crate::agents::executor::ExecutionStatistics {
        self.executor.get_statistics().await
    }

    /// Get agent registry information
    pub async fn get_agent_info(&self) -> Vec<crate::agents::registry::AgentInfo> {
        self.registry.list_agents().await
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: &str) -> Result<bool> {
        self.executor.cancel_task(task_id).await
    }

    /// Wait for all tasks to complete
    pub async fn wait_for_completion(&self, timeout: Option<std::time::Duration>) -> Result<()> {
        self.executor.wait_for_completion(timeout).await
    }

    /// Shutdown the agent framework
    pub async fn shutdown(&self) -> Result<()> {
        self.executor.shutdown().await?;
        self.registry.shutdown().await?;
        Ok(())
    }

    /// Check if agent framework is available for the given configuration
    pub fn is_agent_framework_available(config: &Config) -> bool {
        // Check if the provider is supported by the agent framework
        config.provider().is_some()
    }

    /// Get the agent context for advanced usage
    pub fn context(&self) -> &AgentContext {
        &self.context
    }

    /// Get the agent registry for advanced usage
    pub fn registry(&self) -> &Arc<AgentRegistry> {
        &self.registry
    }

    /// Get the executor for advanced usage
    pub fn executor(&self) -> &Arc<AgentExecutor> {
        &self.executor
    }
}

/// Configuration extension for agent framework settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    /// Whether to use the agent framework
    pub enabled: bool,

    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: Option<usize>,

    /// Default task timeout in seconds
    pub default_timeout_seconds: Option<u64>,

    /// Whether to enable async task execution
    pub async_execution: bool,

    /// Agent-specific settings
    pub agents: HashMap<String, AgentSettings>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSettings {
    /// Whether this agent is enabled
    pub enabled: bool,

    /// Agent-specific timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Agent-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_tasks: Some(5),
            default_timeout_seconds: Some(300),
            async_execution: false,
            agents: HashMap::new(),
        }
    }
}

/// Extension trait to add agent configuration to the main Config
pub trait ConfigAgentExt {
    fn agent_config(&self) -> AgentConfig;
    #[must_use]
    fn with_agent_config(self, config: AgentConfig) -> Self;
}

// Note: This would need to be implemented when integrating with the actual Config struct
// For now, it serves as documentation of the intended interface

/// Helper function to create agent integration from Git-Iris context
pub async fn create_agent_integration_from_context(
    config: Config,
    git_repo: GitRepo,
) -> Result<Option<AgentIntegration>> {
    if AgentIntegration::is_agent_framework_available(&config) {
        let integration = AgentIntegration::new(config, git_repo).await?;
        Ok(Some(integration))
    } else {
        Ok(None)
    }
}

/// CLI command handlers using agent framework
///
/// Handle gen command with agent framework
#[allow(clippy::fn_params_excessive_bools)]
pub async fn handle_gen_with_agent(
    common: CommonParams,
    auto_commit: bool,
    use_gitmoji: bool,
    print_only: bool,
    verify: bool,
    repository_url: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    let git_repo = if let Some(_url) = repository_url {
        // For remote repos, we'd need different handling
        // For now, use current directory
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    } else {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    };

    let integration = create_agent_integration_from_context(config, git_repo).await?;

    if let Some(integration) = integration {
        // Initialize Iris status
        crate::agents::status::IRIS_STATUS.planning();

        // Create and start the Iris agent spinner
        let spinner = crate::ui::create_spinner("");

        // Use agent framework with gitmoji and verification settings
        let preset = common.preset.as_deref();
        let instructions = common.instructions.as_deref();

        // Pass additional parameters to the agent
        let mut agent_params = HashMap::new();
        if let Some(preset) = preset {
            agent_params.insert(
                "preset".to_string(),
                serde_json::Value::String(preset.to_string()),
            );
        }
        if let Some(instructions) = instructions {
            agent_params.insert(
                "instructions".to_string(),
                serde_json::Value::String(instructions.to_string()),
            );
        }
        agent_params.insert(
            "use_gitmoji".to_string(),
            serde_json::Value::Bool(use_gitmoji),
        );
        agent_params.insert("verify".to_string(), serde_json::Value::Bool(verify));

        // Use streaming for better user experience
        let commit_message = integration
            .generate_commit_message_with_params_streaming(agent_params, None)
            .await?;

        // Mark completion
        crate::agents::status::IRIS_STATUS.completed();

        // Give a moment for the completion message to be seen
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Stop the spinner
        spinner.finish_and_clear();

        if print_only {
            println!("{commit_message}");
        } else {
            // Handle auto-commit logic here
            crate::ui::print_success(&format!("Generated commit message: {commit_message}"));

            if auto_commit {
                // Perform actual commit - this would need to be implemented
                crate::ui::print_success("Committed changes using agent framework");
            }
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!("Agent framework not available"))
    }
}

/// Handle review command with agent framework
pub async fn handle_review_with_agent(
    common: CommonParams,
    print: bool,
    repository_url: Option<String>,
    include_unstaged: bool,
    commit: Option<String>,
    from: Option<String>,
    to: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    let git_repo = if let Some(_url) = repository_url {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    } else {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    };

    let integration = create_agent_integration_from_context(config, git_repo).await?;

    if let Some(integration) = integration {
        // Initialize Iris status
        crate::agents::status::IRIS_STATUS.planning();

        // Create and start the Iris agent spinner
        let spinner = crate::ui::create_spinner("");

        // Build commit range from parameters
        let commit_range = if let Some(commit) = commit {
            Some(commit)
        } else if let (Some(from), Some(to)) = (from, to) {
            Some(format!("{from}..{to}"))
        } else {
            None
        };

        // Build focus areas and options
        let mut focus_areas = vec!["security", "performance", "maintainability"];
        if include_unstaged {
            focus_areas.push("unstaged_changes");
        }

        let review_data = integration
            .review_changes(commit_range.as_deref(), Some(focus_areas))
            .await?;

        // Mark completion
        crate::agents::status::IRIS_STATUS.completed();

        // Give a moment for the completion message to be seen
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Stop the spinner
        spinner.finish_and_clear();

        // Parse the review data to format it properly
        if let Ok(generated_review) =
            serde_json::from_value::<crate::commit::review::GeneratedReview>(review_data.clone())
        {
            // Use the same beautiful formatting as non-agent review
            let formatted_review = generated_review.format();

            if print {
                println!("{formatted_review}");
            } else {
                crate::ui::print_success("Code review completed using agent framework");
                println!("{formatted_review}");
            }
        } else {
            // Fallback to JSON if parsing fails
            if print {
                println!("{}", serde_json::to_string_pretty(&review_data)?);
            } else {
                crate::ui::print_success("Code review completed using agent framework");
                println!("{}", serde_json::to_string_pretty(&review_data)?);
            }
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!("Agent framework not available"))
    }
}

/// Handle changelog command with agent framework
pub async fn handle_changelog_with_agent(
    common: CommonParams,
    from: String,
    to: Option<String>,
    repository_url: Option<String>,
    update: bool,
    file: Option<String>,
    version_name: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    let git_repo = if let Some(_url) = repository_url {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    } else {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    };

    let integration = create_agent_integration_from_context(config, git_repo).await?;

    if let Some(integration) = integration {
        let version = version_name.as_deref().unwrap_or("Latest");

        let changelog = integration
            .generate_changelog(version, Some(&from), to.as_deref(), Some("markdown"))
            .await?;

        if update {
            let file_path = file.as_deref().unwrap_or("CHANGELOG.md");
            tokio::fs::write(file_path, &changelog).await?;
            crate::ui::print_success(&format!("Updated changelog: {file_path}"));
        } else {
            println!("{changelog}");
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!("Agent framework not available"))
    }
}

/// Handle release notes command with agent framework
pub async fn handle_release_notes_with_agent(
    common: CommonParams,
    from: String,
    to: Option<String>,
    repository_url: Option<String>,
    version_name: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    let git_repo = if let Some(_url) = repository_url {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    } else {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    };

    let integration = create_agent_integration_from_context(config, git_repo).await?;

    if let Some(integration) = integration {
        let version = version_name.as_deref().unwrap_or("Latest");

        let release_notes = integration
            .generate_release_notes(version, Some(&from), to.as_deref())
            .await?;

        println!("{release_notes}");

        Ok(())
    } else {
        Err(anyhow::anyhow!("Agent framework not available"))
    }
}

/// Handle PR command with agent framework
pub async fn handle_pr_with_agent(
    common: CommonParams,
    print: bool,
    repository_url: Option<String>,
    from: Option<String>,
    to: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    let git_repo = if let Some(_url) = repository_url {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    } else {
        crate::git::GitRepo::new(&std::env::current_dir()?)?
    };

    let integration = create_agent_integration_from_context(config, git_repo).await?;

    if let Some(integration) = integration {
        // Build commit range for PR analysis
        let commit_range = if let (Some(from_ref), Some(to_ref)) = (&from, &to) {
            Some(format!("{from_ref}..{to_ref}"))
        } else if let Some(from_ref) = &from {
            Some(format!("{from_ref}..HEAD"))
        } else {
            to.as_ref().map(|to_ref| format!("HEAD..{to_ref}"))
        };

        // For PR description, we use the review agent but with different parameters
        let review_data = integration
            .review_changes(
                commit_range.as_deref(),
                Some(vec!["summary", "changes", "testing"]), // Focus on PR-relevant aspects
            )
            .await?;

        // Format as PR description
        let pr_description = format_pr_description(&review_data);

        if print {
            println!("{pr_description}");
        } else {
            crate::ui::print_success("PR description generated using agent framework");
            println!("{pr_description}");
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!("Agent framework not available"))
    }
}

/// Helper function to format review data as PR description
fn format_pr_description(review_data: &serde_json::Value) -> String {
    use std::fmt::Write;

    let mut description = String::new();

    // Title and summary section
    description.push_str("## Summary\n\n");
    if let Some(summary) = review_data.get("summary").and_then(|v| v.as_str()) {
        writeln!(description, "{summary}\n").unwrap();
    } else {
        description.push_str("*No summary available*\n\n");
    }

    // Changes section with better formatting
    description.push_str("## Changes\n\n");
    if let Some(changes) = review_data.get("changes") {
        match changes {
            serde_json::Value::String(changes_str) => {
                writeln!(description, "{changes_str}\n").unwrap();
            }
            serde_json::Value::Array(changes_array) => {
                for change in changes_array {
                    if let Some(change_str) = change.as_str() {
                        writeln!(description, "- {change_str}").unwrap();
                    }
                }
                description.push('\n');
            }
            _ => {
                writeln!(description, "{changes}\n").unwrap();
            }
        }
    } else {
        description.push_str("*No changes documented*\n\n");
    }

    // Impact section if available
    if let Some(impact) = review_data.get("impact").and_then(|v| v.as_str()) {
        description.push_str("## Impact\n\n");
        writeln!(description, "{impact}\n").unwrap();
    }

    // Testing section with enhanced formatting
    description.push_str("## Testing\n\n");
    if let Some(testing) = review_data.get("testing") {
        match testing {
            serde_json::Value::String(test_str) => {
                writeln!(description, "{test_str}\n").unwrap();
            }
            serde_json::Value::Array(test_array) => {
                description.push_str("### Test Coverage\n\n");
                for test in test_array {
                    if let Some(test_str) = test.as_str() {
                        writeln!(description, "- [ ] {test_str}").unwrap();
                    }
                }
                description.push('\n');
            }
            _ => {
                writeln!(description, "{testing}\n").unwrap();
            }
        }
    } else {
        description.push_str("- [ ] Manual testing completed\n");
        description.push_str("- [ ] Unit tests updated\n");
        description.push_str("- [ ] Integration tests verified\n\n");
    }

    // Additional metadata if available
    if let Some(breaking_changes) = review_data
        .get("breaking_changes")
        .and_then(serde_json::Value::as_bool)
    {
        if breaking_changes {
            description.push_str("## ⚠️ Breaking Changes\n\n");
            description.push_str(
                "This PR contains breaking changes. Please review the migration guide.\n\n",
            );
        }
    }

    // Performance impact if available
    if let Some(performance) = review_data.get("performance").and_then(|v| v.as_str()) {
        description.push_str("## Performance Impact\n\n");
        writeln!(description, "{performance}\n").unwrap();
    }

    description
}
