//! Iris Agent - The main coordination layer for Git-Iris AI operations
//!
//! This is the streamlined version that leverages the new agent architecture
//! with specialized agents, services, and workflows.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent as IrisAgentTrait, TaskResult},
    services::{LLMService, ResponseParser, WorkflowOrchestrator},
    specialized::{ChangelogAgent, CommitAgent, PullRequestAgent, ReviewAgent},
    status::TokenMetrics,
    tools::AgentTool,
    workflows::{ChangelogWorkflow, CommitWorkflow, ReleaseNotesWorkflow, ReviewWorkflow},
};

/// Streaming callback trait for real-time feedback with token counting
#[async_trait]
pub trait StreamingCallback: Send + Sync {
    /// Called when a new chunk of text is received with token information
    async fn on_chunk(&self, chunk: &str, tokens: Option<TokenMetrics>) -> Result<()>;

    /// Called when streaming is complete
    async fn on_complete(&self, full_response: &str, final_tokens: TokenMetrics) -> Result<()>;

    /// Called when an error occurs during streaming
    async fn on_error(&self, error: &anyhow::Error) -> Result<()>;

    /// Called when status message should be updated (LLM-generated, max 80 chars)
    async fn on_status_update(&self, message: &str) -> Result<()>;
}

/// Dynamic streaming callback that updates the UI with live token counting
pub struct IrisStreamingCallback {
    show_chunks: bool,
    current_tokens: Arc<std::sync::Mutex<TokenMetrics>>,
}

impl IrisStreamingCallback {
    pub fn new(show_chunks: bool) -> Self {
        Self {
            show_chunks,
            current_tokens: Arc::new(std::sync::Mutex::new(TokenMetrics::default())),
        }
    }
}

#[async_trait]
impl StreamingCallback for IrisStreamingCallback {
    async fn on_chunk(&self, _chunk: &str, tokens: Option<TokenMetrics>) -> Result<()> {
        if let Some(token_metrics) = tokens {
            // Update token metrics only - no status updates
            if let Ok(mut current) = self.current_tokens.lock() {
                *current = token_metrics;
            }
        }
        Ok(())
    }

    async fn on_complete(&self, _full_response: &str, final_tokens: TokenMetrics) -> Result<()> {
        // Just mark as completed
        if let Ok(mut current) = self.current_tokens.lock() {
            *current = final_tokens;
        }
        crate::iris_status_completed!();
        Ok(())
    }

    async fn on_error(&self, error: &anyhow::Error) -> Result<()> {
        crate::iris_status_error!(&format!("Stream error: {error}"));
        Ok(())
    }

    async fn on_status_update(&self, _message: &str) -> Result<()> {
        // Skip status updates during streaming to prevent flipping
        Ok(())
    }
}

/// The streamlined Iris agent - coordinates specialized agents and workflows
pub struct IrisAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    #[allow(dead_code)]
    backend: AgentBackend,
    #[allow(dead_code)]
    tools: Vec<Arc<dyn AgentTool>>,
    initialized: bool,

    // Services
    #[allow(dead_code)]
    llm_service: LLMService,
    #[allow(dead_code)]
    parser: ResponseParser,
    #[allow(dead_code)]
    orchestrator: WorkflowOrchestrator,

    // Specialized agents
    #[allow(dead_code)]
    commit_agent: CommitAgent,
    review_agent: ReviewAgent,
    #[allow(dead_code)]
    changelog_agent: ChangelogAgent,
    pr_agent: PullRequestAgent,

    // Workflows
    #[allow(dead_code)]
    commit_workflow: CommitWorkflow,
    #[allow(dead_code)]
    review_workflow: ReviewWorkflow,
    changelog_workflow: ChangelogWorkflow,
    release_notes_workflow: ReleaseNotesWorkflow,
}

impl IrisAgent {
    /// Create a new streamlined Iris agent with the new architecture
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        // Initialize services
        let llm_service = LLMService::new(backend.clone());
        let parser = ResponseParser::new();

        // Create tool registry and populate it with provided tools
        let mut tool_registry = crate::agents::tools::ToolRegistry::new();
        for tool in &tools {
            tool_registry.register_tool(tool.clone());
        }

        let tool_registry_arc = std::sync::Arc::new(tool_registry);
        let orchestrator = WorkflowOrchestrator::new(
            std::sync::Arc::new(llm_service.clone()),
            tool_registry_arc.clone(),
        );

        // Initialize specialized agents
        let commit_agent = CommitAgent::new(&backend);
        let review_agent = ReviewAgent::new_with_tool_registry(&backend, tool_registry_arc.clone());
        let changelog_agent = ChangelogAgent::new(&backend);
        let pr_agent = PullRequestAgent::new(&backend);

        // Initialize workflows
        let commit_workflow =
            CommitWorkflow::new(llm_service.clone(), parser.clone(), orchestrator.clone());
        let review_workflow =
            ReviewWorkflow::new(llm_service.clone(), parser.clone(), orchestrator.clone());
        let changelog_workflow = ChangelogWorkflow::new(llm_service.clone(), parser.clone());
        let release_notes_workflow = ReleaseNotesWorkflow::new(llm_service.clone(), parser.clone());

        Self {
            id: "iris".to_string(),
            name: "Iris".to_string(),
            description: "AI-powered Git workflow automation assistant".to_string(),
            capabilities: vec![
                "commit_generation".to_string(),
                "code_review".to_string(),
                "changelog_generation".to_string(),
                "pr_generation".to_string(),
                "release_notes".to_string(),
            ],
            backend,
            tools,
            initialized: false,
            llm_service,
            parser,
            orchestrator,
            commit_agent,
            review_agent,
            changelog_agent,
            pr_agent,
            commit_workflow,
            review_workflow,
            changelog_workflow,
            release_notes_workflow,
        }
    }

    /// Generate commit message using the new architecture
    pub async fn generate_commit_message(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        self.commit_agent
            .execute_task("generate_commit_message", context, params)
            .await
    }

    /// Generate commit message with streaming support
    pub async fn generate_commit_message_streaming(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
        _callback: &dyn StreamingCallback,
    ) -> Result<TaskResult> {
        // For now, delegate to the commit agent
        // TODO: Add streaming support to specialized agents
        self.commit_agent
            .execute_task("generate_commit_message", context, params)
            .await
    }

    /// Generate code review using the new architecture
    pub async fn generate_code_review(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        self.review_agent
            .execute_task("generate_code_review", context, params)
            .await
    }

    /// Generate pull request using the new architecture
    pub async fn generate_pull_request(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        self.pr_agent
            .execute_task("generate_pr", context, params)
            .await
    }

    /// Generate changelog using the new workflow architecture
    pub async fn generate_changelog(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        let from_ref = params
            .get("from_ref")
            .and_then(|v| v.as_str())
            .unwrap_or("HEAD~1");
        let to_ref = params
            .get("to_ref")
            .and_then(|v| v.as_str())
            .unwrap_or("HEAD");
        let version = params.get("version").and_then(|v| v.as_str());

        self.changelog_workflow
            .generate_changelog(context, from_ref, to_ref, version)
            .await
    }

    /// Generate release notes using the new workflow architecture
    pub async fn generate_release_notes(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        let from_ref = params
            .get("from_ref")
            .and_then(|v| v.as_str())
            .unwrap_or("HEAD~1");
        let to_ref = params
            .get("to_ref")
            .and_then(|v| v.as_str())
            .unwrap_or("HEAD");
        let version = params.get("version").and_then(|v| v.as_str());

        self.release_notes_workflow
            .generate_release_notes(context, from_ref, to_ref, version)
            .await
    }
}

#[async_trait]
impl IrisAgentTrait for IrisAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    async fn execute_task(
        &self,
        task: &str,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        match task {
            "generate_commit_message" => self.generate_commit_message(context, params).await,
            "generate_review" => self.generate_code_review(context, params).await,
            "generate_pr" => self.generate_pull_request(context, params).await,
            "generate_changelog" => self.generate_changelog(context, params).await,
            "generate_release_notes" => self.generate_release_notes(context, params).await,
            _ => Err(anyhow::anyhow!("Unknown task: {}", task)),
        }
    }

    fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_commit_message"
                | "generate_review"
                | "generate_pr"
                | "generate_changelog"
                | "generate_release_notes"
        )
    }

    fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_commit_message" => 10,
            "generate_review" => 9,
            "generate_pr" => 8,
            "generate_changelog" => 7,
            "generate_release_notes" => 6,
            _ => 0,
        }
    }

    async fn initialize(&mut self, _context: &AgentContext) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
