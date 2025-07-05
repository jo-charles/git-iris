use anyhow::Result;
use std::collections::HashMap;

use crate::agents::{
    core::{AgentContext, TaskResult},
    services::{GenerationRequest, LLMService, ResponseParser, WorkflowOrchestrator},
    status::IrisPhase,
};
use crate::commit::prompt::create_review_system_prompt;
use crate::commit::review::GeneratedReview;
use crate::context::CommitContext;
use crate::log_debug;

/// Workflow for code review generation
/// Orchestrates intelligent code review analysis and generation
pub struct ReviewWorkflow {
    llm_service: LLMService,
    parser: ResponseParser,
    orchestrator: WorkflowOrchestrator,
}

impl ReviewWorkflow {
    /// Create a new review workflow with the provided services
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

    /// Generate a comprehensive code review with intelligent analysis
    pub async fn generate_code_review(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("🔍 ReviewWorkflow: Starting intelligent code review");

        // Step 1: Gather intelligent context using orchestrator
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;
        log_debug!("🧠 ReviewWorkflow: Intelligent context gathered");

        // Step 2: Build commit context with intelligent enhancements
        let commit_context = self
            .build_enhanced_commit_context(context, &intelligent_context)
            .await?;
        log_debug!("📝 ReviewWorkflow: Enhanced commit context built");

        // Step 3: Create review-specific prompts
        let system_prompt = create_review_system_prompt(&context.config)?;
        let managed_user_prompt = Self::manage_review_context(&commit_context);
        log_debug!("💭 ReviewWorkflow: Review prompts created");

        // Step 4: Generate review with LLM
        let request = GenerationRequest::new(system_prompt, managed_user_prompt)
            .with_phase(IrisPhase::Analysis)
            .with_context("code review analysis")
            .with_max_tokens(6144);

        let generated_review = self.llm_service.generate(request).await?;
        log_debug!("✨ ReviewWorkflow: Review generated");

        // Step 5: Parse and validate review response
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedReview>(&generated_review)?;

        log_debug!(
            "✅ ReviewWorkflow: Code review completed with {} issues",
            parsed_response.issues.len()
        );

        Ok(TaskResult::success_with_data(
            "Code review generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.88))
    }

    /// Build enhanced commit context with intelligent analysis
    async fn build_enhanced_commit_context(
        &self,
        context: &AgentContext,
        intelligent_context: &crate::agents::services::IntelligentContext,
    ) -> Result<CommitContext> {
        log_debug!("🏗️ ReviewWorkflow: Building enhanced commit context for review");

        // Start with standard git context
        let mut git_context = context.git_repo.get_git_info(&context.config).await?;

        // Enhance with intelligent analysis
        for file in &mut git_context.staged_files {
            if let Some(relevance) = intelligent_context
                .files_with_relevance
                .iter()
                .find(|f| f.path == file.path)
            {
                // Add review-specific analysis
                file.analysis
                    .push(format!("Review Priority: {:.2}", relevance.relevance_score));
                file.analysis
                    .push(format!("Security Impact: {}", relevance.impact_assessment));
                file.analysis.extend(relevance.key_changes.clone());

                // Add technical analysis for review
                if !intelligent_context.technical_analysis.is_empty() {
                    file.analysis.push(format!(
                        "Technical Context: {}",
                        intelligent_context.technical_analysis
                    ));
                }
            }
        }

        log_debug!(
            "✅ ReviewWorkflow: Enhanced {} files for code review",
            git_context.staged_files.len()
        );

        Ok(git_context)
    }

    /// Manage review context for optimal token usage
    fn manage_review_context(context: &CommitContext) -> String {
        use std::fmt::Write;
        log_debug!("📊 ReviewWorkflow: Managing review context for optimal analysis");

        let mut managed_prompt = String::new();

        // Add project context
        writeln!(
            managed_prompt,
            "Project: {} ({})",
            context
                .project_metadata
                .language
                .as_deref()
                .unwrap_or("Unknown"),
            context
                .project_metadata
                .framework
                .as_deref()
                .unwrap_or("No framework")
        )
        .unwrap();

        // Add summary with priority on critical information
        write!(
            managed_prompt,
            "\n## Changes Summary\n{}\n",
            context.summary
        )
        .unwrap();

        // Include diff statistics if available and meaningful
        if !context.diff_stat.is_empty() {
            write!(
                managed_prompt,
                "\n## Diff Statistics\n{}\n",
                context.diff_stat
            )
            .unwrap();
        }

        // Add recent commits with focus on patterns
        if !context.recent_commits.is_empty() {
            write!(managed_prompt, "\n## Recent Commits\n").unwrap();
            for commit in context.recent_commits.iter().take(5) {
                // Limit to last 5 commits
                writeln!(
                    managed_prompt,
                    "- {}: {}",
                    &commit.hash[..8],
                    commit.message
                )
                .unwrap();
            }
        }

        // Add staged files if available
        if !context.staged_files.is_empty() {
            write!(managed_prompt, "\n## Files in Scope\n").unwrap();
            for file in context.staged_files.iter().take(10) {
                // Limit to 10 files for readability
                writeln!(managed_prompt, "- {} ({:?})", file.path, file.change_type).unwrap();
            }
        }

        // Add metadata context
        if !context.project_metadata.dependencies.is_empty() {
            write!(managed_prompt, "\n## Key Dependencies\n").unwrap();
            for dep in context.project_metadata.dependencies.iter().take(8) {
                writeln!(managed_prompt, "- {dep}").unwrap();
            }
        }

        // Add focused instructions
        write!(
            managed_prompt,
            "\n## Review Focus\n\
            Please analyze the changes with attention to:\n\
            - Code quality and maintainability\n\
            - Potential bugs or security issues\n\
            - Performance implications\n\
            - Test coverage and reliability\n\
            - Documentation and clarity\n\
            - Adherence to project patterns\n"
        )
        .unwrap();

        log_debug!(
            "📝 ReviewWorkflow: Context managed - {} characters total",
            managed_prompt.len()
        );

        managed_prompt
    }
}
