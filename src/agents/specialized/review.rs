use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    services::{GenerationRequest, LLMService, ResponseParser, WorkflowOrchestrator},
    status::IrisPhase,
};
use crate::commit::prompt::{create_review_system_prompt, create_review_user_prompt};
use crate::commit::review::GeneratedReview;
use crate::context::CommitContext;
use crate::log_debug;

/// Specialized agent for code review generation
/// Focused on intelligent code analysis and review generation
pub struct ReviewAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    llm_service: LLMService,
    parser: ResponseParser,
    orchestrator: WorkflowOrchestrator,
}

impl ReviewAgent {
    #[must_use]
    pub fn new(backend: &AgentBackend) -> Self {
        Self {
            id: "review_agent".to_string(),
            name: "Iris Review".to_string(),
            description: "AI assistant specialized in comprehensive code review and analysis"
                .to_string(),
            capabilities: vec![
                "code_review".to_string(),
                "security_analysis".to_string(),
                "performance_analysis".to_string(),
                "context_management".to_string(),
                "diff_analysis".to_string(),
                "issue_detection".to_string(),
                "code_quality_assessment".to_string(),
            ],
            llm_service: LLMService::new(backend.clone()),
            parser: ResponseParser::new(),
            orchestrator: WorkflowOrchestrator::new(
                std::sync::Arc::new(LLMService::new(backend.clone())),
                std::sync::Arc::new(crate::agents::tools::ToolRegistry::new()),
            ),
        }
    }

    /// Generate a comprehensive code review with intelligent analysis
    pub async fn generate_code_review(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("🔍 ReviewAgent: Starting intelligent code review");

        // Step 1: Gather intelligent context using orchestrator
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;

        // Step 2: Build commit context from intelligent analysis
        let commit_context = self
            .orchestrator
            .build_commit_context(context, &intelligent_context)
            .await?;

        // Step 3: Generate review using intelligent context management
        let system_prompt = create_review_system_prompt(&context.config)?;
        let managed_user_prompt = self.manage_review_context(&commit_context).await?;

        log_debug!(
            "📏 ReviewAgent: Managed context size: {} chars",
            managed_user_prompt.len()
        );

        // Step 4: Generate review using LLM service
        let request = GenerationRequest::builder()
            .system_prompt(system_prompt)
            .user_prompt(managed_user_prompt)
            .phase(IrisPhase::Analysis)
            .operation_type("code_review")
            .with_context("reviewing code changes for issues and improvements")
            .current_step(1)
            .total_steps(Some(1))
            .build()?;

        let generated_review = self.llm_service.generate(request).await?;

        // Step 5: Parse and validate response
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedReview>(&generated_review)?;

        log_debug!(
            "✅ ReviewAgent: Code review completed with {} issues, {} suggestions",
            parsed_response.issues.len(),
            parsed_response.suggestions.len()
        );

        Ok(TaskResult::success_with_data(
            "Code review generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.88))
    }

    /// Use LLM to intelligently manage context for code reviews
    async fn manage_review_context(&self, context: &CommitContext) -> Result<String> {
        log_debug!("🧠 ReviewAgent: Using LLM to intelligently manage review context");

        let full_context = create_review_user_prompt(context);
        let context_size = full_context.len();

        log_debug!(
            "📏 ReviewAgent: Full context size: {} characters",
            context_size
        );

        // If context is reasonable size, use it directly
        if context_size < 8000 {
            log_debug!("✅ ReviewAgent: Context size manageable, proceeding with full review");
            return Ok(full_context);
        }

        // Let LLM intelligently summarize and prioritize
        let smart_context_prompt = format!(
            "You are Iris, an expert code reviewer. The code context below is too large for optimal review. 
            
            Your task: Create a focused, intelligent summary that preserves all critical information needed for a comprehensive code review.
            
            **What to preserve:**
            - All security-critical changes
            - Complex logic that needs careful review  
            - Performance-sensitive code
            - Error handling patterns
            - API changes or breaking changes
            - Critical diff sections (keep exact code)
            
            **What to summarize:**
            - Repetitive patterns
            - Simple refactoring
            - Formatting changes
            - Non-critical utility functions
            
            **Original Context ({context_size} chars):**
            {full_context}
            
            Create an intelligent, focused review context that captures everything important while being concise enough for thorough analysis."
        );

        let request = GenerationRequest::builder()
            .system_prompt("You are an expert at context management for code reviews.".to_string())
            .user_prompt(smart_context_prompt)
            .phase(IrisPhase::Analysis)
            .operation_type("context_management")
            .with_context("optimizing context for comprehensive review")
            .current_step(1)
            .total_steps(Some(1))
            .build()?;

        let managed_context = self.llm_service.generate(request).await?;

        log_debug!(
            "✅ ReviewAgent: Created LLM-managed context - {} chars (reduced from {})",
            managed_context.len(),
            context_size
        );

        Ok(managed_context)
    }

    /// Check if this agent can handle the given task
    pub fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_code_review"
                | "analyze_security_issues"
                | "analyze_performance_issues"
                | "detect_code_smells"
                | "review_diff"
        )
    }

    /// Get task priority for this agent's capabilities
    pub fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_code_review" => 10, // Highest priority
            "analyze_security_issues" => 9,
            "analyze_performance_issues" => 8,
            "detect_code_smells" => 7,
            "review_diff" => 6,
            _ => 0,
        }
    }
}

#[async_trait]
impl IrisAgent for ReviewAgent {
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
            "generate_code_review" => self.generate_code_review(context, params).await,
            _ => {
                anyhow::bail!("ReviewAgent cannot handle task: {}", task)
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
        log_debug!("🚀 ReviewAgent: Initialized and ready for code review");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        log_debug!("🧹 ReviewAgent: Cleanup completed");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
