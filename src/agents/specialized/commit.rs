use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    iris::StreamingCallback,
    services::{GenerationRequest, LLMService, ResponseParser, WorkflowOrchestrator},
    status::IrisPhase,
};
use crate::commit::prompt::{create_system_prompt, create_user_prompt};
use crate::commit::types::GeneratedMessage;
use crate::iris_status_completed;
use crate::log_debug;

/// Specialized agent for commit message generation
/// Focused, efficient, and leverages the extracted services layer
pub struct CommitAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    llm_service: LLMService,
    parser: ResponseParser,
    orchestrator: WorkflowOrchestrator,
}

impl CommitAgent {
    #[must_use]
    pub fn new(backend: &AgentBackend) -> Self {
        Self {
            id: "commit_agent".to_string(),
            name: "Iris Commit".to_string(),
            description: "AI assistant specialized in generating intelligent commit messages"
                .to_string(),
            capabilities: vec![
                "commit_message_generation".to_string(),
                "commit_validation".to_string(),
                "gitmoji_integration".to_string(),
                "intelligent_context_analysis".to_string(),
                "diff_analysis".to_string(),
            ],
            llm_service: LLMService::new(backend.clone()),
            parser: ResponseParser::new(),
            orchestrator: WorkflowOrchestrator::new(
                std::sync::Arc::new(LLMService::new(backend.clone())),
                std::sync::Arc::new(crate::agents::tools::create_default_tool_registry()),
            ),
        }
    }

    /// Generate a commit message with intelligent context analysis
    pub async fn generate_commit_message(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        let preset = params
            .get("preset")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let use_gitmoji = params
            .get("gitmoji")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(context.config.use_gitmoji);

        log_debug!(
            "🤖 CommitAgent: Generating commit message with preset: '{}', gitmoji: {}",
            preset,
            use_gitmoji
        );

        // Step 1: Use orchestrator to gather intelligent context
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;

        // Step 2: Build commit context from intelligent analysis
        let commit_context = self
            .orchestrator
            .build_commit_context(context, &intelligent_context)
            .await?;

        // Step 3: Create optimized configuration and prompts
        let mut config_clone = (*context.config).clone();
        config_clone.use_gitmoji = use_gitmoji;

        let system_prompt = create_system_prompt(&config_clone)?;
        let user_prompt = create_user_prompt(&commit_context);

        // Step 4: Generate using LLM service
        let request = GenerationRequest::builder()
            .system_prompt(system_prompt)
            .user_prompt(user_prompt)
            .phase(IrisPhase::Generation)
            .operation_type("commit message generation")
            .with_context("crafting elegant commit message from staged changes")
            .current_step(4)
            .total_steps(Some(5))
            .build()?;

        let generated_message = self.llm_service.generate(request).await?;

        // Step 5: Parse and validate response using parser service
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedMessage>(&generated_message)?;

        log_debug!(
            "✅ CommitAgent: Generated commit message '{}'",
            parsed_response.title
        );

        iris_status_completed!();
        Ok(TaskResult::success_with_data(
            "Commit message generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.92))
    }

    /// Generate a commit message with streaming support
    pub async fn generate_commit_message_streaming(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
        callback: &dyn StreamingCallback,
    ) -> Result<TaskResult> {
        let preset = params
            .get("preset")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let use_gitmoji = params
            .get("gitmoji")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(context.config.use_gitmoji);

        log_debug!(
            "🌊 CommitAgent: Generating commit message with streaming - preset: '{}', gitmoji: {}",
            preset,
            use_gitmoji
        );

        // Step 1: Gather intelligent context
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;

        // Step 2: Build commit context
        let commit_context = self
            .orchestrator
            .build_commit_context(context, &intelligent_context)
            .await?;

        // Step 3: Create prompts
        let mut config_clone = (*context.config).clone();
        config_clone.use_gitmoji = use_gitmoji;

        let system_prompt = create_system_prompt(&config_clone)?;
        let user_prompt = create_user_prompt(&commit_context);

        // Step 4: Generate with streaming using LLM service
        let request = GenerationRequest::builder()
            .system_prompt(system_prompt)
            .user_prompt(user_prompt)
            .phase(IrisPhase::Generation)
            .operation_type("commit message generation")
            .with_context("crafting elegant commit message from staged changes")
            .current_step(4)
            .total_steps(Some(5))
            .with_streaming_callback(callback)
            .build()?;

        let generated_message = self
            .llm_service
            .generate_streaming(request, callback)
            .await?;

        // Step 5: Parse and validate response
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedMessage>(&generated_message)?;

        log_debug!(
            "✅ CommitAgent: Generated streaming commit message '{}'",
            parsed_response.title
        );

        iris_status_completed!();
        Ok(TaskResult::success_with_data(
            "Commit message generated successfully with streaming".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.92))
    }

    /// Check if this agent can handle the given task
    pub fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_commit_message"
                | "generate_commit_message_streaming"
                | "validate_commit"
                | "analyze_commit_changes"
        )
    }

    /// Get task priority for this agent's capabilities
    pub fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_commit_message" | "generate_commit_message_streaming" => 10, // Highest priority
            "validate_commit" => 8,
            "analyze_commit_changes" => 7,
            _ => 0,
        }
    }
}

#[async_trait]
impl IrisAgent for CommitAgent {
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
            _ => {
                anyhow::bail!("CommitAgent cannot handle task: {}", task)
            }
        }
    }

    fn can_handle_task(&self, task: &str) -> bool {
        self.can_handle_task(task)
    }

    fn task_priority(&self, task: &str) -> u8 {
        self.task_priority(task)
    }

    async fn initialize(&mut self, _context: &AgentContext) -> Result<()> {
        log_debug!("🚀 CommitAgent: Initialized and ready for commit message generation");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        log_debug!("🧹 CommitAgent: Cleanup completed");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
