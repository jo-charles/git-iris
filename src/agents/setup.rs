//! Agent setup service for Git-Iris
//!
//! This service handles all the setup and initialization for the agent framework,
//! including configuration loading, client creation, and agent setup.

use anyhow::Result;
use rig::client::builder::DynClientBuilder;
use std::collections::HashMap;

use crate::agents::{AgentBackend, IrisAgent, IrisAgentBuilder};
use crate::common::CommonParams;
use crate::config::Config;
use crate::git::GitRepo;
use crate::llm::{get_combined_config, validate_provider_config};

/// Service for setting up agents with proper configuration
pub struct AgentSetupService {
    config: Config,
    git_repo: Option<GitRepo>,
    client_builder: Option<DynClientBuilder>,
}

impl AgentSetupService {
    /// Create a new setup service with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            git_repo: None,
            client_builder: None,
        }
    }

    /// Create setup service from common parameters (following existing patterns)
    pub fn from_common_params(
        common_params: &CommonParams,
        repository_url: Option<String>,
    ) -> Result<Self> {
        let mut config = Config::load()?;

        // Apply common parameters to config (following existing pattern)
        common_params.apply_to_config(&mut config)?;

        let mut setup_service = Self::new(config);

        // Setup git repo if needed
        if let Some(repo_url) = repository_url {
            // Handle remote repository setup (following existing pattern)
            setup_service.git_repo = Some(GitRepo::new_from_url(Some(repo_url))?);
        } else {
            // Use local repository
            setup_service.git_repo = Some(GitRepo::new(&std::env::current_dir()?)?);
        }

        Ok(setup_service)
    }

    /// Create a configured Iris agent
    pub async fn create_iris_agent(&mut self) -> Result<IrisAgent> {
        let backend = AgentBackend::from_config(&self.config)?;
        let client_builder = self.create_client_builder(&backend)?;

        IrisAgentBuilder::new()
            .with_client(client_builder)
            .with_provider(&backend.provider_name)
            .with_model(&backend.model)
            .build()
    }

    /// Create a Rig client builder based on the backend configuration
    fn create_client_builder(&mut self, backend: &AgentBackend) -> Result<DynClientBuilder> {
        // Validate provider configuration
        validate_provider_config(&self.config, &backend.provider_name)?;

        // Get combined configuration parameters
        let combined_config =
            get_combined_config(&self.config, &backend.provider_name, &HashMap::new());

        // Validate API key exists
        self.validate_provider_config(&backend.provider_name, &combined_config)?;

        // Create client builder - Rig will read from environment variables
        let client_builder = DynClientBuilder::new();

        // Create a new client builder for storage
        let stored_builder = DynClientBuilder::new();
        self.client_builder = Some(stored_builder);

        Ok(client_builder)
    }

    /// Validate provider configuration has required fields
    fn validate_provider_config(
        &self,
        provider: &str,
        config: &HashMap<String, String>,
    ) -> Result<()> {
        match provider {
            "openai" => {
                if config.get("api_key").is_none() {
                    return Err(anyhow::anyhow!(
                        "No API key found for OpenAI. Please set OPENAI_API_KEY environment variable or configure it in your config file."
                    ));
                }
            }
            "anthropic" => {
                if config.get("api_key").is_none() {
                    return Err(anyhow::anyhow!(
                        "No API key found for Anthropic. Please set ANTHROPIC_API_KEY environment variable or configure it in your config file."
                    ));
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported provider: {}", provider));
            }
        }
        Ok(())
    }

    /// Get the git repository instance
    pub fn git_repo(&self) -> Option<&GitRepo> {
        self.git_repo.as_ref()
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the client builder
    pub fn client_builder(&self) -> Option<&DynClientBuilder> {
        self.client_builder.as_ref()
    }
}

/// High-level function to handle tasks with agents using a common pattern
/// This is a convenience function that sets up an agent and executes a task
pub async fn handle_with_agent<F, Fut, T>(
    common_params: CommonParams,
    repository_url: Option<String>,
    capability: &str,
    task_prompt: &str,
    handler: F,
) -> Result<T>
where
    F: FnOnce(crate::agents::iris::StructuredResponse) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    // Create setup service
    let mut setup_service = AgentSetupService::from_common_params(&common_params, repository_url)?;

    // Create agent
    let mut agent = setup_service.create_iris_agent().await?;

    // Execute task with capability - now returns StructuredResponse
    let result = agent.execute_task(capability, task_prompt).await?;

    // Call the handler with the result
    handler(result).await
}

/// Simple factory function for creating agents with minimal configuration
pub async fn create_agent_with_defaults(provider: &str, model: &str) -> Result<IrisAgent> {
    let client_builder = DynClientBuilder::new();

    IrisAgentBuilder::new()
        .with_client(client_builder)
        .with_provider(provider)
        .with_model(model)
        .build()
}

/// Create an agent from environment variables
pub async fn create_agent_from_env() -> Result<IrisAgent> {
    let provider = std::env::var("IRIS_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    let model = std::env::var("IRIS_MODEL").unwrap_or_else(|_| {
        use crate::llm::get_default_model_for_provider;
        get_default_model_for_provider(&provider).to_string()
    });

    create_agent_with_defaults(&provider, &model).await
}
