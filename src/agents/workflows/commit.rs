use anyhow::Result;
use serde_json;
use std::collections::HashMap;

use crate::agents::{
    core::{AgentContext, TaskResult},
    iris::StreamingCallback,
    services::{GenerationRequest, LLMService, ResponseParser, WorkflowOrchestrator},
    status::IrisPhase,
};
use crate::commit::prompt::{create_system_prompt, create_user_prompt};
use crate::commit::types::GeneratedMessage;
use crate::context::CommitContext;
use crate::iris_status_completed;
use crate::log_debug;

/// Workflow for commit message generation
/// Orchestrates the complex multi-step process of generating intelligent commit messages
pub struct CommitWorkflow {
    llm_service: LLMService,
    parser: ResponseParser,
    orchestrator: WorkflowOrchestrator,
}

impl CommitWorkflow {
    /// Create a new commit workflow with the provided services
    pub fn new(
        llm_service: LLMService,
        parser: ResponseParser,
        orchestrator: WorkflowOrchestrator,
    ) -> Self {
        Self {
            llm_service,
            parser,
            orchestrator,
        }
    }

    /// Generate a commit message using the full intelligent workflow
    pub async fn generate_commit_message(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("🚀 CommitWorkflow: Starting commit message generation");

        let use_gitmoji = params
            .get("use_gitmoji")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // Step 1: Gather intelligent context using orchestrator
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;
        log_debug!("🧠 CommitWorkflow: Intelligent context gathered");

        // Step 2: Build commit context from intelligent analysis
        let commit_context = self
            .build_commit_context(context, &intelligent_context)
            .await?;
        log_debug!("📝 CommitWorkflow: Commit context built");

        // Step 3: Create prompts with gitmoji configuration
        let mut config_clone = (*context.config).clone();
        config_clone.use_gitmoji = use_gitmoji;

        let system_prompt = create_system_prompt(&config_clone)?;
        let user_prompt = create_user_prompt(&commit_context);
        log_debug!("💭 CommitWorkflow: Prompts created");

        // Step 4: Generate with LLM
        let request = GenerationRequest::new(system_prompt, user_prompt)
            .with_phase(IrisPhase::Generation)
            .with_context("commit message generation")
            .with_max_tokens(4096);

        let generated_message = self.llm_service.generate(request).await?;
        log_debug!("✨ CommitWorkflow: Message generated");

        // Step 5: Parse and validate response
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedMessage>(&generated_message)?;

        log_debug!(
            "✅ CommitWorkflow: Generated - Title: '{}', {} chars total",
            parsed_response.title,
            parsed_response.message.len()
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
        log_debug!("🚀 CommitWorkflow: Starting streaming commit message generation");

        let use_gitmoji = params
            .get("use_gitmoji")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // Step 1: Gather intelligent context
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;

        // Step 2: Build commit context
        let commit_context = self
            .build_commit_context(context, &intelligent_context)
            .await?;

        // Step 3: Create prompts
        let mut config_clone = (*context.config).clone();
        config_clone.use_gitmoji = use_gitmoji;

        let system_prompt = create_system_prompt(&config_clone)?;
        let user_prompt = create_user_prompt(&commit_context);

        // Step 4: Generate with streaming
        let request = GenerationRequest::new(system_prompt, user_prompt)
            .with_phase(IrisPhase::Generation)
            .with_context("commit message generation")
            .with_streaming_callback(callback)
            .with_max_tokens(4096);

        let generated_message = self
            .llm_service
            .generate_streaming(request, callback)
            .await?;

        // Step 5: Parse response
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedMessage>(&generated_message)?;

        log_debug!(
            "✅ CommitWorkflow: Streaming complete - Title: '{}', {} chars total",
            parsed_response.title,
            parsed_response.message.len()
        );

        iris_status_completed!();
        Ok(TaskResult::success_with_data(
            "Commit message generated successfully with streaming".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.92))
    }

    /// Build commit context from intelligent analysis
    async fn build_commit_context(
        &self,
        context: &AgentContext,
        intelligent_context: &crate::agents::services::IntelligentContext,
    ) -> Result<CommitContext> {
        log_debug!("🏗️ CommitWorkflow: Building enhanced commit context");

        // Start with standard git context
        let mut git_context = context.git_repo.get_git_info(&context.config).await?;

        // Enhance staged files with intelligent analysis
        for file in &mut git_context.staged_files {
            if let Some(relevance) = intelligent_context
                .files_with_relevance
                .iter()
                .find(|f| f.path == file.path)
            {
                // Merge intelligent analysis into file analysis
                file.analysis
                    .push(format!("Relevance: {:.2}", relevance.relevance_score));
                file.analysis
                    .push(format!("Impact: {}", relevance.impact_assessment));
                file.analysis.extend(relevance.key_changes.clone());
                file.analysis.push(relevance.analysis.clone());
            }
        }

        log_debug!(
            "✅ CommitWorkflow: Enhanced {} files with intelligent analysis",
            git_context.staged_files.len()
        );

        Ok(git_context)
    }
}
