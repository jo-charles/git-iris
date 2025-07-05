use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

use crate::log_debug;
use crate::{
    agents::{
        core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
        services::{GenerationRequest, LLMService, ResponseParser, WorkflowOrchestrator},
        status::IrisPhase,
    },
    commit::prompt::{create_pr_system_prompt, create_pr_user_prompt},
    commit::types::GeneratedPullRequest,
};

/// Specialized agent for pull request description generation
/// Focused on creating comprehensive PR descriptions and analysis
pub struct PullRequestAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    llm_service: LLMService,
    parser: ResponseParser,
    orchestrator: WorkflowOrchestrator,
}

impl PullRequestAgent {
    pub fn new(backend: &AgentBackend) -> Self {
        let llm_service = LLMService::new(backend.clone());
        let tool_registry = std::sync::Arc::new(crate::agents::tools::ToolRegistry::new());

        Self {
            id: "pr_agent".to_string(),
            name: "Iris PR".to_string(),
            description:
                "AI assistant specialized in generating comprehensive pull request descriptions"
                    .to_string(),
            capabilities: vec![
                "pull_request_description".to_string(),
                "commit_analysis".to_string(),
                "change_summarization".to_string(),
                "impact_assessment".to_string(),
                "pr_formatting".to_string(),
                "breaking_change_detection".to_string(),
            ],
            llm_service: llm_service.clone(),
            parser: ResponseParser::new(),
            orchestrator: WorkflowOrchestrator::new(
                std::sync::Arc::new(llm_service),
                tool_registry,
            ),
        }
    }

    /// Generate a pull request description with intelligent analysis
    pub async fn generate_pull_request(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("📋 PullRequestAgent: Starting pull request description generation");

        // Get commit messages from params
        let commit_messages = params
            .get("commit_messages")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        log_debug!(
            "📝 PullRequestAgent: Processing {} commit messages for PR description",
            commit_messages.len()
        );

        // Step 1: Gather intelligent context if no commit messages provided
        let intelligent_context = if commit_messages.is_empty() {
            Some(
                self.orchestrator
                    .gather_intelligent_context(context)
                    .await?,
            )
        } else {
            None
        };

        // Step 2: Build commit context
        let commit_context = if let Some(ref intel_ctx) = intelligent_context {
            self.orchestrator
                .build_commit_context(context, intel_ctx)
                .await?
        } else {
            // Use commit messages to build a simpler context
            Self::build_context_from_commit_messages(&commit_messages, context)
        };

        // Step 3: Generate PR description using existing prompt system
        let system_prompt = create_pr_system_prompt(&context.config)?;
        let user_prompt = create_pr_user_prompt(&commit_context, &commit_messages);

        // Step 4: Generate using LLM service
        let request = GenerationRequest::builder()
            .system_prompt(system_prompt)
            .user_prompt(user_prompt)
            .phase(IrisPhase::Generation)
            .operation_type("pull request generation")
            .with_context("generating comprehensive PR description")
            .current_step(4)
            .total_steps(Some(5))
            .build()?;

        let generated_description = self.llm_service.generate(request).await?;

        // Step 5: Parse and validate response
        let parsed_response = self
            .parser
            .parse_json_response::<GeneratedPullRequest>(&generated_description)?;

        log_debug!(
            "✅ PullRequestAgent: Generated PR '{}'",
            parsed_response.title
        );

        Ok(TaskResult::success_with_data(
            "Pull request description generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.88))
    }

    /// Build context from provided commit messages
    fn build_context_from_commit_messages(
        commit_messages: &[String],
        context: &AgentContext,
    ) -> crate::context::CommitContext {
        log_debug!(
            "🔧 PullRequestAgent: Building context from {} commit messages",
            commit_messages.len()
        );

        // Create a simplified commit context when we have commit messages
        let mut commit_context = crate::context::CommitContext::new(
            "main".to_string(), // branch name - we'll need to get this from git
            Vec::new(),         // recent_commits
            Vec::new(),         // staged_files
            crate::context::ProjectMetadata::default(), // project_metadata
            "Unknown".to_string(), // user_name - we'll need to get this from git config
            "unknown@example.com".to_string(), // user_email - we'll need to get this from git config
        );

        // Add commit messages as the main context
        let commits_summary = commit_messages.join("\n");
        commit_context.summary = format!(
            "Pull Request Changes:\n{}\n\nBased on {} commits in this PR.",
            commits_summary,
            commit_messages.len()
        );

        // Try to get basic diff information if available
        if let Ok(output) = std::process::Command::new("git")
            .args(["diff", "--stat", "HEAD~{}", "HEAD"])
            .arg(commit_messages.len().to_string())
            .current_dir(context.git_repo.repo_path())
            .output()
        {
            if output.status.success() {
                let diff_stat = String::from_utf8_lossy(&output.stdout);
                if !diff_stat.trim().is_empty() {
                    commit_context.diff_stat = diff_stat.to_string();
                }
            }
        }

        commit_context
    }

    /// Analyze the impact and scope of changes for PR description
    pub async fn analyze_change_impact(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("🔍 PullRequestAgent: Analyzing change impact");

        // Use orchestrator to gather intelligent context
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;

        // Analyze the impact using LLM
        let impact_prompt = format!(
            "Analyze the impact and scope of the following changes:\n\n{}\n\nProvide:\n1. Impact level (Low/Medium/High)\n2. Affected components\n3. Potential risks\n4. Breaking changes (if any)\n5. Testing recommendations",
            intelligent_context.change_summary
        );

        let request = GenerationRequest::builder()
            .system_prompt(
                "You are an expert at analyzing code changes and their impact.".to_string(),
            )
            .user_prompt(impact_prompt)
            .phase(IrisPhase::Analysis)
            .operation_type("impact_analysis")
            .context_hint("analyzing change impact and scope")
            .current_step(1)
            .total_steps(Some(1))
            .build()?;

        let analysis_result = self.llm_service.generate(request).await?;

        log_debug!("✅ PullRequestAgent: Change impact analysis completed");

        Ok(TaskResult::success_with_data(
            "Change impact analysis completed".to_string(),
            serde_json::json!({
                "analysis": analysis_result,
                "files_analyzed": intelligent_context.files_with_relevance.len(),
                "change_summary": intelligent_context.change_summary
            }),
        )
        .with_confidence(0.85))
    }

    /// Generate a summary of changes for quick review
    pub async fn generate_change_summary(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("📄 PullRequestAgent: Generating change summary");

        // Use orchestrator to gather context
        let intelligent_context = self
            .orchestrator
            .gather_intelligent_context(context)
            .await?;

        // Create a concise summary
        let summary_prompt = format!(
            "Create a concise summary of the following changes for a pull request:\n\n{}\n\nFocus on:\n1. Main purpose of changes\n2. Key modifications\n3. Files affected\n4. Notable improvements or fixes",
            intelligent_context.change_summary
        );

        let request = GenerationRequest::builder()
            .system_prompt(
                "You are an expert at creating concise, informative change summaries.".to_string(),
            )
            .user_prompt(summary_prompt)
            .phase(IrisPhase::Synthesis)
            .operation_type("change_summary")
            .context_hint("creating concise change summary")
            .current_step(1)
            .total_steps(Some(1))
            .build()?;

        let summary_result = self.llm_service.generate(request).await?;

        log_debug!("✅ PullRequestAgent: Change summary generated");

        Ok(TaskResult::success_with_data(
            "Change summary generated successfully".to_string(),
            serde_json::json!({
                "summary": summary_result,
                "files_count": intelligent_context.files_with_relevance.len(),
                "technical_analysis": intelligent_context.technical_analysis
            }),
        )
        .with_confidence(0.87))
    }

    /// Check if this agent can handle the given task
    pub fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_pull_request"
                | "analyze_change_impact"
                | "generate_change_summary"
                | "detect_breaking_changes"
                | "format_pr_description"
        )
    }

    /// Get task priority for this agent's capabilities
    pub fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_pull_request" => 10, // Highest priority
            "analyze_change_impact" | "detect_breaking_changes" => 9,
            "generate_change_summary" => 8,
            "format_pr_description" => 7,
            _ => 0,
        }
    }
}

#[async_trait]
impl IrisAgent for PullRequestAgent {
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
            "generate_pull_request" => self.generate_pull_request(context, params).await,
            "analyze_change_impact" => self.analyze_change_impact(context, params).await,
            "generate_change_summary" => self.generate_change_summary(context, params).await,
            _ => {
                anyhow::bail!("PullRequestAgent cannot handle task: {}", task)
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
        log_debug!("🚀 PullRequestAgent: Initialized and ready for PR generation");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        log_debug!("🧹 PullRequestAgent: Cleanup completed");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
