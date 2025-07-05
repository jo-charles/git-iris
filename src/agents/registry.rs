use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agents::{
    core::{AgentContext, IrisAgent, TaskResult},
    tools::ToolRegistry,
};

/// Central registry for managing agents and their capabilities
pub struct AgentRegistry {
    agents: RwLock<HashMap<String, Arc<dyn IrisAgent>>>,
    capability_index: RwLock<HashMap<String, Vec<String>>>,
    tool_registry: Arc<ToolRegistry>,
    initialized: bool,
}

impl AgentRegistry {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            capability_index: RwLock::new(HashMap::new()),
            tool_registry,
            initialized: false,
        }
    }

    /// Register a new agent in the registry
    pub async fn register_agent(&self, agent: Arc<dyn IrisAgent>) -> Result<()> {
        let agent_id = agent.id().to_string();

        // Update capability index
        {
            let mut capability_index = self.capability_index.write().await;
            for capability in agent.capabilities() {
                capability_index
                    .entry(capability)
                    .or_insert_with(Vec::new)
                    .push(agent_id.clone());
            }
        }

        // Add to agents registry
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id, agent);
        }

        Ok(())
    }

    /// Unregister an agent from the registry
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        let agent = {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id)
        };

        if let Some(agent) = agent {
            // Clean up capability index
            let mut capability_index = self.capability_index.write().await;
            for capability in agent.capabilities() {
                if let Some(agent_ids) = capability_index.get_mut(&capability) {
                    agent_ids.retain(|id| id != agent_id);
                    if agent_ids.is_empty() {
                        capability_index.remove(&capability);
                    }
                }
            }

            // Allow agent to clean up
            agent.cleanup().await?;
        }

        Ok(())
    }

    /// Get an agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Option<Arc<dyn IrisAgent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Find the best agent for a specific task
    pub async fn find_agent_for_task(&self, task: &str) -> Option<Arc<dyn IrisAgent>> {
        let agents = self.agents.read().await;

        let mut best_agent: Option<Arc<dyn IrisAgent>> = None;
        let mut best_priority = 0u8;

        for agent in agents.values() {
            if agent.can_handle_task(task) {
                let priority = agent.task_priority(task);
                if priority > best_priority {
                    best_priority = priority;
                    best_agent = Some(agent.clone());
                }
            }
        }

        best_agent
    }

    /// Get all agents that can handle a specific capability
    pub async fn get_agents_for_capability(&self, capability: &str) -> Vec<Arc<dyn IrisAgent>> {
        let capability_index = self.capability_index.read().await;
        let agents = self.agents.read().await;

        if let Some(agent_ids) = capability_index.get(capability) {
            agent_ids
                .iter()
                .filter_map(|id| agents.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Execute a task by automatically selecting the best agent
    pub async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        let agent = self
            .find_agent_for_task(task)
            .await
            .ok_or_else(|| anyhow::anyhow!("No agent found capable of handling task: {}", task))?;

        agent.execute_task(task, context, params).await
    }

    /// List all registered agents
    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;

        agents
            .values()
            .map(|agent| AgentInfo {
                id: agent.id().to_string(),
                name: agent.name().to_string(),
                description: agent.description().to_string(),
                capabilities: agent.capabilities(),
            })
            .collect()
    }

    /// List all available capabilities
    pub async fn list_capabilities(&self) -> Vec<String> {
        let capability_index = self.capability_index.read().await;
        capability_index.keys().cloned().collect()
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let agents = self.agents.read().await;
        let capability_index = self.capability_index.read().await;

        RegistryStatistics {
            total_agents: agents.len(),
            total_capabilities: capability_index.len(),
            total_tools: self.tool_registry.list_tools().len(),
        }
    }

    /// Initialize all registered agents
    pub async fn initialize_all_agents(&mut self, _context: &AgentContext) -> Result<()> {
        let agent_ids: Vec<String> = {
            let agents = self.agents.read().await;
            agents.keys().cloned().collect()
        };

        for agent_id in agent_ids {
            if let Some(_agent) = self.get_agent(&agent_id).await {
                // Note: This is a simplified approach. In practice, we'd need a way to
                // get mutable access to the agent for initialization
                // agent.initialize(context).await?;
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Shut down the registry and clean up all agents
    pub async fn shutdown(&self) -> Result<()> {
        let agent_ids: Vec<String> = {
            let agents = self.agents.read().await;
            agents.keys().cloned().collect()
        };

        for agent_id in agent_ids {
            self.unregister_agent(&agent_id).await?;
        }

        Ok(())
    }

    /// Health check for all agents
    pub async fn health_check(&self) -> HealthStatus {
        let agents = self.agents.read().await;
        let total = agents.len();

        // In a real implementation, we'd ping each agent to check its status
        // For now, assume all are healthy if registered
        let healthy = total;
        let unhealthy = 0;

        HealthStatus {
            total_agents: total,
            healthy_agents: healthy,
            unhealthy_agents: unhealthy,
            last_check: chrono::Utc::now(),
        }
    }

    /// Get tool registry
    pub fn tool_registry(&self) -> &Arc<ToolRegistry> {
        &self.tool_registry
    }
}

/// Information about a registered agent
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_agents: usize,
    pub total_capabilities: usize,
    pub total_tools: usize,
}

/// Health status of the registry
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub unhealthy_agents: usize,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Agent factory for creating and registering agents
pub struct AgentRegistryBuilder {
    registry: AgentRegistry,
}

impl AgentRegistryBuilder {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry: AgentRegistry::new(tool_registry),
        }
    }

    /// Register the default set of agents
    pub async fn with_default_agents(
        self,
        backend: crate::agents::core::AgentBackend,
    ) -> Result<AgentRegistry> {
        let tool_registry = self.registry.tool_registry.clone();

        // Create and register the unified Iris agent with all tools
        let mut all_tools: Vec<Arc<dyn crate::agents::tools::AgentTool>> = Vec::new();

        // Gather all available tools for comprehensive capability
        let git_tools = tool_registry.get_tools_for_capability("git")?;
        let commit_tools = tool_registry.get_tools_for_capability("commit")?;
        let review_tools = tool_registry.get_tools_for_capability("review")?;
        let file_analysis_tools = tool_registry.get_tools_for_capability("file_analysis")?;

        all_tools.extend(git_tools);
        all_tools.extend(commit_tools);
        all_tools.extend(review_tools);
        all_tools.extend(file_analysis_tools);

        // Remove duplicates by tool ID
        all_tools.sort_by(|a, b| a.id().cmp(b.id()));
        all_tools.dedup_by(|a, b| a.id() == b.id());

        let iris_agent = Arc::new(crate::agents::iris::IrisAgent::new(backend, all_tools));
        self.registry.register_agent(iris_agent).await?;

        Ok(self.registry)
    }

    /// Add a custom agent to the registry
    pub async fn with_agent(self, agent: Arc<dyn IrisAgent>) -> Result<Self> {
        self.registry.register_agent(agent).await?;
        Ok(self)
    }

    /// Build the final registry
    pub fn build(self) -> AgentRegistry {
        self.registry
    }
}

/// Convenience functions for creating pre-configured registries
impl AgentRegistry {
    /// Create a registry with default agents and tools
    pub async fn create_default(backend: crate::agents::core::AgentBackend) -> Result<Self> {
        let tool_registry = Arc::new(crate::agents::tools::create_default_tool_registry());

        AgentRegistryBuilder::new(tool_registry)
            .with_default_agents(backend)
            .await
    }

    /// Create an empty registry for custom configuration
    pub fn create_empty() -> Self {
        let tool_registry = Arc::new(crate::agents::tools::create_default_tool_registry());
        Self::new(tool_registry)
    }
}
