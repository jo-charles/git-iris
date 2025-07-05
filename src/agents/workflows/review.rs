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
        let managed_user_prompt = self.manage_review_context(&commit_context).await?;
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
    async fn manage_review_context(&self, context: &CommitContext) -> Result<String> {
        log_debug!("📊 ReviewWorkflow: Managing review context for optimal analysis");

        let mut managed_prompt = String::new();

        // Add project context
        managed_prompt.push_str(&format!(
            "Project: {} ({})\n",
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
        ));

        // Add branch context
        managed_prompt.push_str(&format!("Branch: {}\n", context.branch));

        // Add recent commits for context
        if !context.recent_commits.is_empty() {
            managed_prompt.push_str("\nRecent commits:\n");
            for commit in context.recent_commits.iter().take(3) {
                managed_prompt.push_str(&format!("- {}: {}\n", &commit.hash[..8], commit.message));
            }
        }

        // Add files with analysis (prioritize by complexity)
        managed_prompt.push_str("\nFiles to review:\n");
        for (idx, file) in context.staged_files.iter().enumerate() {
            managed_prompt.push_str(&format!("\n{}. File: {}\n", idx + 1, file.path));
            managed_prompt.push_str(&format!("   Change: {:?}\n", file.change_type));

            // Add analysis if available
            if !file.analysis.is_empty() {
                managed_prompt.push_str("   Analysis:\n");
                for analysis in &file.analysis {
                    managed_prompt.push_str(&format!("   - {analysis}\n"));
                }
            }

            // Add diff (truncated for large files)
            if !file.diff.is_empty() {
                let diff_preview = if file.diff.len() > 2000 {
                    format!("{}...[truncated]", &file.diff[..2000])
                } else {
                    file.diff.clone()
                };
                managed_prompt.push_str(&format!("   Diff:\n{diff_preview}\n"));
            }
        }

        log_debug!(
            "✅ ReviewWorkflow: Managed context created - {} chars",
            managed_prompt.len()
        );

        Ok(managed_prompt)
    }
}
