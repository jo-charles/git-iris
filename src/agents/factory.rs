use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    specialized::{ChangelogAgent, CommitAgent, PullRequestAgent, ReviewAgent},
    tools::AgentTool,
};
use crate::log_debug;

/// Factory for creating and managing specialized Iris agents
/// Provides a clean interface for agent creation and task routing
pub struct AgentFactory {
    backend: AgentBackend,
    #[allow(dead_code)]
    tools: Vec<Arc<dyn AgentTool>>,
    agents: HashMap<String, Box<dyn IrisAgent>>,
}

impl AgentFactory {
    /// Create a new agent factory with the specified backend and tools
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        Self {
            backend,
            tools,
            agents: HashMap::new(),
        }
    }

    /// Initialize all specialized agents
    pub async fn initialize(&mut self, context: &AgentContext) -> Result<()> {
        log_debug!("🏭 AgentFactory: Initializing specialized agents");

        // Create and initialize each specialized agent
        let mut commit_agent = Box::new(CommitAgent::new(&self.backend));
        commit_agent.initialize(context).await?;
        self.agents.insert("commit".to_string(), commit_agent);

        let mut review_agent = Box::new(ReviewAgent::new(&self.backend));
        review_agent.initialize(context).await?;
        self.agents.insert("review".to_string(), review_agent);

        let mut changelog_agent = Box::new(ChangelogAgent::new(&self.backend));
        changelog_agent.initialize(context).await?;
        self.agents.insert("changelog".to_string(), changelog_agent);

        let mut pr_agent = Box::new(PullRequestAgent::new(&self.backend));
        pr_agent.initialize(context).await?;
        self.agents.insert("pr".to_string(), pr_agent);

        log_debug!(
            "✅ AgentFactory: Initialized {} specialized agents",
            self.agents.len()
        );
        Ok(())
    }

    /// Route a task to the most appropriate specialized agent
    pub async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!(
            "🎯 AgentFactory: Routing task '{}' to specialized agent",
            task
        );

        // Find the best agent for this task
        let selected_agent = self.select_agent_for_task(task)?;

        log_debug!(
            "🤖 AgentFactory: Selected {} agent for task '{}'",
            selected_agent.name(),
            task
        );

        // Execute the task with the selected agent
        selected_agent.execute_task(task, context, params).await
    }

    /// Select the most appropriate agent for a given task
    fn select_agent_for_task(&self, task: &str) -> Result<&dyn IrisAgent> {
        let mut best_agent: Option<&dyn IrisAgent> = None;
        let mut best_priority = 0u8;

        // Check each agent's capability and priority for this task
        for agent in self.agents.values() {
            if agent.can_handle_task(task) {
                let priority = agent.task_priority(task);
                if priority > best_priority {
                    best_priority = priority;
                    best_agent = Some(agent.as_ref());
                }
            }
        }

        best_agent
            .ok_or_else(|| anyhow::anyhow!("No specialized agent found to handle task: {}", task))
    }

    /// Get agent by ID for direct access
    pub fn get_agent(&self, agent_id: &str) -> Option<&dyn IrisAgent> {
        self.agents.get(agent_id).map(std::convert::AsRef::as_ref)
    }

    /// List all available agents with their capabilities
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        self.agents
            .values()
            .map(|agent| AgentInfo {
                id: agent.id().to_string(),
                name: agent.name().to_string(),
                description: agent.description().to_string(),
                capabilities: agent.capabilities(),
            })
            .collect()
    }

    /// Check if any agent can handle the given task
    pub fn can_handle_task(&self, task: &str) -> bool {
        self.agents
            .values()
            .any(|agent| agent.can_handle_task(task))
    }

    /// Get the best agent for a task with its priority score
    pub fn get_task_assignment(&self, task: &str) -> Option<(String, u8)> {
        let mut best_agent_id: Option<String> = None;
        let mut best_priority = 0u8;

        for (agent_id, agent) in &self.agents {
            if agent.can_handle_task(task) {
                let priority = agent.task_priority(task);
                if priority > best_priority {
                    best_priority = priority;
                    best_agent_id = Some(agent_id.clone());
                }
            }
        }

        best_agent_id.map(|id| (id, best_priority))
    }

    /// Get comprehensive task routing information
    pub fn get_routing_info(&self) -> HashMap<String, Vec<(String, u8)>> {
        let mut routing_info = HashMap::new();

        // Common tasks to check
        let tasks = vec![
            "generate_commit_message",
            "generate_code_review",
            "generate_pull_request",
            "generate_changelog",
            "generate_release_notes",
            "analyze_change_impact",
            "analyze_security_issues",
        ];

        for task in tasks {
            let mut agents_for_task = Vec::new();

            for (agent_id, agent) in &self.agents {
                if agent.can_handle_task(task) {
                    let priority = agent.task_priority(task);
                    agents_for_task.push((agent_id.clone(), priority));
                }
            }

            // Sort by priority (highest first)
            agents_for_task.sort_by(|a, b| b.1.cmp(&a.1));
            routing_info.insert(task.to_string(), agents_for_task);
        }

        routing_info
    }

    /// Cleanup all agents
    pub async fn cleanup(&self) -> Result<()> {
        log_debug!("🧹 AgentFactory: Cleaning up all specialized agents");

        for agent in self.agents.values() {
            if let Err(e) = agent.cleanup().await {
                log_debug!(
                    "⚠️ AgentFactory: Error cleaning up agent {}: {}",
                    agent.id(),
                    e
                );
            }
        }

        log_debug!("✅ AgentFactory: Cleanup completed for all agents");
        Ok(())
    }
}

/// Information about an agent
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

/// Helper function to create a fully configured agent factory
pub async fn create_agent_factory(
    backend: AgentBackend,
    tools: Vec<Arc<dyn AgentTool>>,
    context: &AgentContext,
) -> Result<AgentFactory> {
    log_debug!("🏭 Creating and initializing agent factory");

    let mut factory = AgentFactory::new(backend, tools);
    factory.initialize(context).await?;

    log_debug!(
        "✅ Agent factory created with {} specialized agents",
        factory.agents.len()
    );
    Ok(factory)
}
