use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::Config;
use crate::git::GitRepo;

/// Simplified Agent Backend that works with Rig's provider system
#[derive(Debug, Clone)]
pub struct AgentBackend {
    pub provider_name: String,
    /// Primary model for complex tasks
    pub model: String,
    /// Fast model for simple/bounded tasks (subagents, parsing, etc.)
    pub fast_model: String,
}

impl AgentBackend {
    pub fn new(provider_name: String, model: String, fast_model: String) -> Self {
        Self {
            provider_name,
            model,
            fast_model,
        }
    }

    /// Create backend from Git-Iris configuration
    pub fn from_config(config: &Config) -> Result<Self> {
        let provider: crate::providers::Provider = config
            .default_provider
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid provider: {}", config.default_provider))?;

        let provider_config = config
            .get_provider_config(&config.default_provider)
            .ok_or_else(|| {
                anyhow::anyhow!("No configuration for provider: {}", config.default_provider)
            })?;

        Ok(Self {
            provider_name: config.default_provider.clone(),
            model: provider_config.effective_model(provider).to_string(),
            fast_model: provider_config.effective_fast_model(provider).to_string(),
        })
    }
}

/// Agent context containing Git repository and configuration
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub config: Config,
    pub git_repo: Arc<GitRepo>,
}

impl AgentContext {
    pub fn new(config: Config, git_repo: GitRepo) -> Self {
        Self {
            config,
            git_repo: Arc::new(git_repo),
        }
    }

    pub fn repo(&self) -> &GitRepo {
        &self.git_repo
    }

    pub fn config(&self) -> &Config {
        &self.config
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
