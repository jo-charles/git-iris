use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    services::{GenerationRequest, LLMService, ResponseParser},
    status::IrisPhase,
};
use crate::log_debug;

/// Specialized agent for changelog generation
/// Focused on creating comprehensive and well-formatted changelogs
pub struct ChangelogAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    llm_service: LLMService,
    #[allow(dead_code)]
    parser: ResponseParser,
}

impl ChangelogAgent {
    #[must_use]
    pub fn new(backend: &AgentBackend) -> Self {
        Self {
            id: "changelog_agent".to_string(),
            name: "Iris Changelog".to_string(),
            description:
                "AI assistant specialized in generating comprehensive changelogs and release notes"
                    .to_string(),
            capabilities: vec![
                "changelog_generation".to_string(),
                "git_history_analysis".to_string(),
                "version_management".to_string(),
                "changelog_formatting".to_string(),
                "commit_categorization".to_string(),
                "breaking_change_detection".to_string(),
            ],
            llm_service: LLMService::new(backend.clone()),
            parser: ResponseParser::new(),
        }
    }

    /// Generate a changelog between Git references
    pub async fn generate_changelog(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("📝 ChangelogAgent: Starting changelog generation");

        let from_tag = params.get("from").and_then(|v| v.as_str());
        let to_tag = params.get("to").and_then(|v| v.as_str());
        let version = params
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unreleased");

        log_debug!(
            "📋 ChangelogAgent: Generating changelog from {:?} to {:?}, version: {}",
            from_tag,
            to_tag,
            version
        );

        // Step 1: Gather Git changelog context
        let git_context = self
            .gather_git_changelog_context(context, from_tag, to_tag)
            .await?;

        // Step 2: Generate changelog content
        let changelog_content = self
            .generate_changelog_content(context, &git_context, version)
            .await?;

        // Step 3: Update changelog file if specified
        if let Some(file_path) = params.get("file").and_then(|v| v.as_str()) {
            self.update_changelog_file(
                &changelog_content,
                file_path,
                context,
                to_tag,
                Some(version.to_string()),
            )
            .await?;
        }

        log_debug!("✅ ChangelogAgent: Changelog generated successfully");

        Ok(TaskResult::success_with_data(
            "Changelog generated successfully".to_string(),
            serde_json::json!({
                "content": changelog_content,
                "version": version,
                "from": from_tag,
                "to": to_tag
            }),
        )
        .with_confidence(0.90))
    }

    /// Generate release notes between Git references
    pub async fn generate_release_notes(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("🚀 ChangelogAgent: Starting release notes generation");

        let from_tag = params.get("from").and_then(|v| v.as_str());
        let to_tag = params.get("to").and_then(|v| v.as_str());
        let version = params
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unreleased");

        log_debug!(
            "📋 ChangelogAgent: Generating release notes from {:?} to {:?}, version: {}",
            from_tag,
            to_tag,
            version
        );

        // Step 1: Gather Git context
        let git_context = self
            .gather_git_changelog_context(context, from_tag, to_tag)
            .await?;

        // Step 2: Generate release notes content
        let release_notes_content = self
            .generate_release_notes_content(context, &git_context, version)
            .await?;

        log_debug!("✅ ChangelogAgent: Release notes generated successfully");

        Ok(TaskResult::success_with_data(
            "Release notes generated successfully".to_string(),
            serde_json::json!({
                "content": release_notes_content,
                "version": version,
                "from": from_tag,
                "to": to_tag
            }),
        )
        .with_confidence(0.90))
    }

    /// Gather Git context for changelog generation
    async fn gather_git_changelog_context(
        &self,
        context: &AgentContext,
        from_tag: Option<&str>,
        to_tag: Option<&str>,
    ) -> Result<String> {
        log_debug!("🔍 ChangelogAgent: Gathering Git changelog context");

        // Build git log command based on provided tags
        let git_command = match (from_tag, to_tag) {
            (Some(from), Some(to)) => {
                format!("git log --oneline --pretty=format:'%h %s' {from}..{to}")
            }
            (Some(from), None) => format!("git log --oneline --pretty=format:'%h %s' {from}..HEAD"),
            (None, Some(to)) => format!("git log --oneline --pretty=format:'%h %s' HEAD..{to}"),
            (None, None) => "git log --oneline --pretty=format:'%h %s' -n 50".to_string(),
        };

        // Execute git command to get commit history
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&git_command)
            .current_dir(context.git_repo.repo_path())
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute git command: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git command failed: {}", error_msg);
        }

        let git_log = String::from_utf8_lossy(&output.stdout);
        log_debug!(
            "📊 ChangelogAgent: Retrieved {} lines of Git history",
            git_log.lines().count()
        );

        Ok(git_log.to_string())
    }

    /// Generate changelog content using LLM
    async fn generate_changelog_content(
        &self,
        context: &AgentContext,
        git_context: &str,
        version: &str,
    ) -> Result<String> {
        log_debug!("🤖 ChangelogAgent: Generating changelog content using LLM");

        let system_prompt = self.create_changelog_system_prompt(context);
        let user_prompt = format!(
            "Generate a comprehensive changelog for version {version} based on the following Git history:\n\n{git_context}\n\nInclude appropriate categorization (Added, Changed, Fixed, Deprecated, Removed, Security) and ensure the format is clean and professional."
        );

        let request = GenerationRequest::builder()
            .system_prompt(system_prompt)
            .user_prompt(user_prompt)
            .phase(IrisPhase::Generation)
            .operation_type("changelog_generation")
            .with_context("creating comprehensive changelog from git history")
            .current_step(1)
            .total_steps(Some(1))
            .build()?;

        let response = self.llm_service.generate(request).await?;
        let formatted_response = self.format_changelog_response(&response)?;

        Ok(formatted_response)
    }

    /// Generate release notes content using LLM
    async fn generate_release_notes_content(
        &self,
        _context: &AgentContext,
        git_context: &str,
        version: &str,
    ) -> Result<String> {
        log_debug!("🤖 ChangelogAgent: Generating release notes content using LLM");

        let system_prompt = self.create_release_notes_system_prompt();
        let user_prompt = format!(
            "Generate professional release notes for version {version} based on the following Git history:\n\n{git_context}\n\nFocus on user-facing changes, improvements, and important technical updates. Make it engaging and informative for users."
        );

        let request = GenerationRequest::builder()
            .system_prompt(system_prompt)
            .user_prompt(user_prompt)
            .phase(IrisPhase::Generation)
            .operation_type("release_notes_generation")
            .context_hint("creating engaging release notes from git history")
            .current_step(1)
            .total_steps(Some(1))
            .build()?;

        let response = self.llm_service.generate(request).await?;
        let formatted_response = self.format_release_notes_response(&response, version)?;

        Ok(formatted_response)
    }

    /// Create system prompt for changelog generation
    fn create_changelog_system_prompt(&self, _context: &AgentContext) -> String {
        "You are an expert at creating comprehensive, well-structured changelogs from Git commit history.

GUIDELINES:
- Follow Keep a Changelog format (https://keepachangelog.com/)
- Categorize changes appropriately: Added, Changed, Fixed, Deprecated, Removed, Security
- Write clear, user-friendly descriptions
- Group related changes together
- Highlight breaking changes
- Use consistent formatting
- Focus on user-facing changes

OUTPUT FORMAT:
Return ONLY the changelog content in proper markdown format, ready to be inserted into a CHANGELOG.md file.".to_string()
    }

    /// Create system prompt for release notes generation
    fn create_release_notes_system_prompt(&self) -> String {
        "You are an expert at creating engaging, informative release notes that communicate changes effectively to users.

GUIDELINES:
- Write for end users, not just developers
- Highlight the most important and exciting changes first
- Explain the impact and benefits of changes
- Use clear, accessible language
- Include relevant technical details where helpful
- Organize content logically
- Make it engaging and professional

OUTPUT FORMAT:
Return well-formatted release notes in markdown that can be used for GitHub releases, blog posts, or announcements.".to_string()
    }

    /// Format changelog response from LLM
    fn format_changelog_response(&self, json_response: &str) -> Result<String> {
        log_debug!("🔧 ChangelogAgent: Formatting changelog response");

        // Try to parse as JSON first, fallback to plain text
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_response) {
            if let Some(content) = parsed.get("content").and_then(|v| v.as_str()) {
                return Ok(self.add_changelog_borders(content));
            }
            if let Some(changelog) = parsed.get("changelog").and_then(|v| v.as_str()) {
                return Ok(self.add_changelog_borders(changelog));
            }
        }

        // Fallback to using the raw response
        let cleaned = self.clean_json_response(json_response);
        Ok(self.add_changelog_borders(&cleaned))
    }

    /// Format release notes response from LLM
    fn format_release_notes_response(&self, json_response: &str, version: &str) -> Result<String> {
        log_debug!("🔧 ChangelogAgent: Formatting release notes response");

        // Try to parse as JSON first, fallback to plain text
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_response) {
            if let Some(content) = parsed.get("content").and_then(|v| v.as_str()) {
                return Ok(format!("# Release Notes - {version}\n\n{content}"));
            }
            if let Some(notes) = parsed.get("release_notes").and_then(|v| v.as_str()) {
                return Ok(format!("# Release Notes - {version}\n\n{notes}"));
            }
        }

        // Fallback to using the raw response
        let cleaned = self.clean_json_response(json_response);
        Ok(format!("# Release Notes - {version}\n\n{cleaned}"))
    }

    /// Add borders around changelog content
    fn add_changelog_borders(&self, content: &str) -> String {
        format!(
            "<!-- CHANGELOG_START -->\n{}\n<!-- CHANGELOG_END -->",
            content.trim()
        )
    }

    /// Clean JSON response for plain text usage
    fn clean_json_response(&self, response: &str) -> String {
        response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string()
    }

    /// Update changelog file with new content
    async fn update_changelog_file(
        &self,
        changelog_content: &str,
        file_path: &str,
        _context: &AgentContext,
        _to_ref: Option<&str>,
        _version_name: Option<String>,
    ) -> Result<()> {
        log_debug!("📝 ChangelogAgent: Updating changelog file: {}", file_path);

        // Read existing file or create new one
        let existing_content = std::fs::read_to_string(file_path).unwrap_or_else(|_| {
            "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n".to_string()
        });

        // Merge new content with existing
        let updated_content =
            self.merge_with_existing_changelog(&existing_content, changelog_content, "# Changelog");

        // Write updated content back to file
        std::fs::write(file_path, updated_content)
            .map_err(|e| anyhow::anyhow!("Failed to write changelog file: {}", e))?;

        log_debug!("✅ ChangelogAgent: Changelog file updated successfully");
        Ok(())
    }

    /// Merge new changelog content with existing file
    fn merge_with_existing_changelog(
        &self,
        existing: &str,
        new_version: &str,
        default_header: &str,
    ) -> String {
        if existing.trim().is_empty() || existing.trim() == default_header {
            format!("{default_header}\n\n{new_version}")
        } else {
            // Insert new version after the header
            let lines: Vec<&str> = existing.lines().collect();
            if lines.is_empty() {
                format!("{default_header}\n\n{new_version}")
            } else {
                let mut result = String::new();
                result.push_str(lines[0]); // Header
                result.push('\n');
                if lines.len() > 1 && lines[1].trim().is_empty() {
                    result.push('\n');
                }
                result.push_str(new_version);
                result.push('\n');

                // Add remaining content
                for (i, line) in lines.iter().enumerate() {
                    if i > 0 {
                        result.push('\n');
                        result.push_str(line);
                    }
                }

                result
            }
        }
    }

    /// Check if this agent can handle the given task
    pub fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_changelog"
                | "generate_release_notes"
                | "analyze_git_history"
                | "categorize_commits"
                | "detect_breaking_changes"
        )
    }

    /// Get task priority for this agent's capabilities
    pub fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_changelog" | "generate_release_notes" => 10, // Highest priority
            "analyze_git_history" => 8,
            "categorize_commits" => 7,
            "detect_breaking_changes" => 9,
            _ => 0,
        }
    }
}

#[async_trait]
impl IrisAgent for ChangelogAgent {
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
            "generate_changelog" => self.generate_changelog(context, params).await,
            "generate_release_notes" => self.generate_release_notes(context, params).await,
            _ => {
                anyhow::bail!("ChangelogAgent cannot handle task: {}", task)
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
        log_debug!("🚀 ChangelogAgent: Initialized and ready for changelog generation");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        log_debug!("🧹 ChangelogAgent: Cleanup completed");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
