use anyhow::Result;

use crate::agents::{
    core::{AgentContext, TaskResult},
    services::{GenerationRequest, LLMService, ResponseParser},
    status::IrisPhase,
};
use crate::log_debug;

/// Workflow for release notes generation
/// Orchestrates the process of generating comprehensive release notes from git changes
pub struct ReleaseNotesWorkflow {
    llm_service: LLMService,
    parser: ResponseParser,
}

impl ReleaseNotesWorkflow {
    /// Create a new release notes workflow with the provided services
    pub fn new(llm_service: LLMService, parser: ResponseParser) -> Self {
        Self {
            llm_service,
            parser,
        }
    }

    /// Generate release notes from git changes
    pub async fn generate_release_notes(
        &self,
        context: &AgentContext,
        from_ref: &str,
        to_ref: &str,
        version: Option<&str>,
    ) -> Result<TaskResult> {
        log_debug!("Starting release notes generation workflow");

        // Phase 1: Analyze git changes comprehensively
        let comprehensive_analysis = self
            .analyze_changes_comprehensive(context, from_ref, to_ref)
            .await?;

        // Phase 2: Extract key highlights and impacts
        let highlights = self.extract_highlights(&comprehensive_analysis).await?;

        // Phase 3: Generate structured release notes
        let release_notes = self
            .generate_structured_notes(&comprehensive_analysis, &highlights, version)
            .await?;

        // Phase 4: Format and finalize
        let formatted_notes = self.format_release_notes(&release_notes).await?;

        Ok(TaskResult::success(formatted_notes))
    }

    /// Comprehensive analysis of changes for release notes
    async fn analyze_changes_comprehensive(
        &self,
        _context: &AgentContext,
        from_ref: &str,
        to_ref: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = self.create_comprehensive_analysis_prompt();
        let user_prompt = format!(
            "Analyze the git changes between {from_ref} and {to_ref} for release notes generation. \
             Focus on impact, user-facing changes, and technical improvements."
        );

        let request = GenerationRequest::new(system_prompt, user_prompt)
            .with_temperature(0.3)
            .with_phase(IrisPhase::Analysis);

        let response = self.llm_service.generate(request).await?;

        self.parser.parse_json_response(&response)
    }

    /// Extract key highlights from comprehensive analysis
    async fn extract_highlights(&self, analysis: &serde_json::Value) -> Result<serde_json::Value> {
        let system_prompt = self.create_highlights_extraction_prompt();
        let user_prompt = format!(
            "Extract key highlights and impactful changes from this analysis:\n\n{}",
            serde_json::to_string_pretty(analysis)?
        );

        let request = GenerationRequest::new(system_prompt, user_prompt)
            .with_temperature(0.4)
            .with_phase(IrisPhase::Synthesis);

        let response = self.llm_service.generate(request).await?;

        self.parser.parse_json_response(&response)
    }

    /// Generate structured release notes content
    async fn generate_structured_notes(
        &self,
        analysis: &serde_json::Value,
        highlights: &serde_json::Value,
        version: Option<&str>,
    ) -> Result<String> {
        let system_prompt = self.create_structured_notes_prompt();
        let user_prompt = format!(
            "Generate comprehensive release notes for version {}:\n\n\
             Analysis:\n{}\n\nHighlights:\n{}",
            version.unwrap_or("Next"),
            serde_json::to_string_pretty(analysis)?,
            serde_json::to_string_pretty(highlights)?
        );

        let request = GenerationRequest::new(system_prompt, user_prompt)
            .with_temperature(0.5)
            .with_phase(IrisPhase::Generation);

        self.llm_service.generate(request).await
    }

    /// Format release notes for final output
    async fn format_release_notes(&self, content: &str) -> Result<String> {
        // Structure and format the release notes
        let lines: Vec<&str> = content.lines().collect();
        let mut formatted = String::new();
        let mut in_code_block = false;

        for line in lines {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                formatted.push_str(&format!("{line}\n"));
            } else if in_code_block {
                formatted.push_str(&format!("{line}\n"));
            } else if line.trim().is_empty() {
                formatted.push('\n');
            } else if line.starts_with("# ") {
                formatted.push_str(&format!("{line}\n\n"));
            } else if line.starts_with("## ") {
                formatted.push_str(&format!("{line}\n"));
            } else if line.starts_with("### ") {
                formatted.push_str(&format!("{line}\n"));
            } else if line.starts_with("- ") || line.starts_with("* ") {
                formatted.push_str(&format!("{line}\n"));
            } else {
                formatted.push_str(&format!("{line}\n"));
            }
        }

        Ok(formatted)
    }

    /// Create comprehensive analysis system prompt
    fn create_comprehensive_analysis_prompt(&self) -> String {
        r"You are an expert technical writer and software release analyst.
Your task is to analyze git changes and understand their impact for release notes generation.

Focus on:
1. User-facing changes and new features
2. Breaking changes and their implications
3. Performance improvements and optimizations
4. Bug fixes and stability improvements
5. Developer experience enhancements
6. Dependencies and infrastructure changes

Provide detailed JSON analysis with clear categorization and impact assessment."
            .to_string()
    }

    /// Create highlights extraction system prompt
    fn create_highlights_extraction_prompt(&self) -> String {
        r"You are an expert at identifying the most important and impactful changes in software releases.
Your task is to extract key highlights that users and stakeholders care about most.

Focus on:
1. Major new features and capabilities
2. Significant improvements and enhancements
3. Important bug fixes and stability improvements
4. Performance gains and optimizations
5. User experience improvements
6. Breaking changes that require attention

Provide JSON output with clear, concise highlights that tell the story of this release.".to_string()
    }

    /// Create structured notes generation system prompt
    fn create_structured_notes_prompt(&self) -> String {
        r"You are an expert technical writer specializing in release notes.
Your task is to create comprehensive, well-structured release notes that:

1. Start with an executive summary of the release
2. Highlight major features and improvements
3. Detail breaking changes with migration guidance
4. List bug fixes and stability improvements
5. Include performance improvements and optimizations
6. Provide upgrade instructions and compatibility notes
7. Acknowledge contributors and community involvement

Write in a clear, professional tone suitable for both technical and non-technical audiences.
Use proper markdown formatting with clear sections and bullet points."
            .to_string()
    }
}
