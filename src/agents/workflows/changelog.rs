use anyhow::Result;

use crate::agents::{
    core::{AgentContext, TaskResult},
    services::{GenerationRequest, LLMService, ResponseParser},
};
use crate::log_debug;

/// Workflow for changelog generation
/// Orchestrates the process of generating comprehensive changelogs from git changes
pub struct ChangelogWorkflow {
    llm_service: LLMService,
    parser: ResponseParser,
}

impl ChangelogWorkflow {
    /// Create a new changelog workflow with the provided services
    pub fn new(llm_service: LLMService, parser: ResponseParser) -> Self {
        Self {
            llm_service,
            parser,
        }
    }

    /// Generate changelog entries from git changes
    pub async fn generate_changelog(
        &self,
        context: &AgentContext,
        from_ref: &str,
        to_ref: &str,
        version: Option<&str>,
    ) -> Result<TaskResult> {
        log_debug!("Starting changelog generation workflow");

        // Phase 1: Analyze git changes
        let commit_analysis = self.analyze_commits(context, from_ref, to_ref).await?;

        // Phase 2: Categorize changes
        let categorized_changes = self.categorize_changes(&commit_analysis).await?;

        // Phase 3: Generate changelog content
        let changelog_content = self
            .generate_changelog_content(&categorized_changes, version)
            .await?;

        // Phase 4: Format and structure output
        let formatted_changelog = Self::format_changelog(&changelog_content);

        Ok(TaskResult::success(formatted_changelog))
    }

    /// Analyze commits between two references
    async fn analyze_commits(
        &self,
        _context: &AgentContext,
        from_ref: &str,
        to_ref: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = Self::create_analysis_system_prompt();
        let user_prompt = Self::create_analysis_user_prompt(from_ref, to_ref);

        let request = GenerationRequest::new(system_prompt, user_prompt).with_temperature(0.3);

        let response = self.llm_service.generate(request).await?;

        self.parser.parse_json_response(&response)
    }

    /// Categorize changes into logical groups
    async fn categorize_changes(&self, analysis: &serde_json::Value) -> Result<serde_json::Value> {
        let system_prompt = Self::create_categorization_system_prompt();
        let user_prompt = format!(
            "Categorize the following changes into logical groups:\n\n{}",
            serde_json::to_string_pretty(analysis)?
        );

        let request = GenerationRequest::new(system_prompt, user_prompt).with_temperature(0.2);

        let response = self.llm_service.generate(request).await?;

        self.parser.parse_json_response(&response)
    }

    /// Generate changelog content from categorized changes
    async fn generate_changelog_content(
        &self,
        categorized_changes: &serde_json::Value,
        version: Option<&str>,
    ) -> Result<String> {
        let system_prompt = Self::create_content_generation_system_prompt();
        let user_prompt = format!(
            "Generate changelog content for version {}:\n\n{}",
            version.unwrap_or("Next"),
            serde_json::to_string_pretty(categorized_changes)?
        );

        let request = GenerationRequest::new(system_prompt, user_prompt).with_temperature(0.4);

        self.llm_service.generate(request).await
    }

    /// Format changelog into final structure
    fn format_changelog(content: &str) -> String {
        // Additional formatting and structure improvements
        let lines: Vec<&str> = content.lines().collect();
        let mut formatted = String::new();

        for line in lines {
            if line.trim().is_empty() {
                formatted.push('\n');
            } else {
                formatted.push_str(line);
                formatted.push('\n');
            }
        }

        formatted
    }

    /// Create system prompt for commit analysis
    fn create_analysis_system_prompt() -> String {
        r"You are an expert at analyzing git commits and understanding code changes.
Your task is to analyze a series of git commits and extract meaningful information about:
1. The type of changes (features, bug fixes, refactoring, etc.)
2. The scope and impact of changes
3. Any breaking changes or notable modifications
4. Dependencies and relationships between changes

Provide structured JSON output with detailed analysis."
            .to_string()
    }

    /// Create user prompt for commit analysis
    fn create_analysis_user_prompt(from_ref: &str, to_ref: &str) -> String {
        format!(
            "Analyze the git commits between {from_ref} and {to_ref} and provide a structured analysis of the changes."
        )
    }

    /// Create system prompt for change categorization
    fn create_categorization_system_prompt() -> String {
        r"You are an expert at organizing and categorizing software changes for changelog generation.
Your task is to group related changes into logical categories such as:
- Added (new features)
- Changed (modifications to existing functionality)
- Deprecated (features marked for removal)
- Removed (deleted features)
- Fixed (bug fixes)
- Security (security-related changes)

Provide well-organized JSON output with clear categorization.".to_string()
    }

    /// Create system prompt for content generation
    fn create_content_generation_system_prompt() -> String {
        r"You are an expert technical writer specializing in changelog generation.
Your task is to create clear, concise, and informative changelog entries that:
1. Follow semantic versioning principles
2. Are written for both technical and non-technical audiences
3. Highlight important changes and their impact
4. Use consistent formatting and structure
5. Include relevant context and motivation for changes

Generate clean, well-formatted changelog content in markdown format."
            .to_string()
    }
}
