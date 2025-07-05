use anyhow::Result;
use async_trait::async_trait;
use rig::client::CompletionClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    tools::AgentTool,
};

/// Specialized agent for changelog and release notes generation
pub struct ChangelogAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    backend: AgentBackend,
    tools: Vec<Arc<dyn AgentTool>>,
    initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub sections: HashMap<String, Vec<ChangeItem>>,
    pub breaking_changes: Vec<BreakingChange>,
    pub migration_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeItem {
    pub description: String,
    pub commit_hash: Option<String>,
    pub author: Option<String>,
    pub pr_number: Option<u32>,
    pub impact: ChangeImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub description: String,
    pub migration_guide: String,
    pub affected_apis: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeImpact {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseNotes {
    pub version: String,
    pub title: String,
    pub summary: String,
    pub highlights: Vec<String>,
    pub changes: ChangelogEntry,
    pub acknowledgments: Vec<String>,
}

impl ChangelogAgent {
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        let mut agent = Self {
            id: "changelog_agent".to_string(),
            name: "Changelog Agent".to_string(),
            description: "Specialized agent for generating changelogs and release notes"
                .to_string(),
            capabilities: vec![
                "changelog_generation".to_string(),
                "release_notes_generation".to_string(),
                "commit_categorization".to_string(),
                "version_analysis".to_string(),
            ],
            backend,
            tools,
            initialized: false,
        };

        // Initialize the agent
        agent.initialized = true;
        agent
    }

    /// Generate changelog entries from commit history
    async fn generate_changelog(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        // Extract parameters
        let version = params
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Version parameter required"))?;

        let from_tag = params.get("from_tag").and_then(|v| v.as_str());
        let to_tag = params
            .get("to_tag")
            .and_then(|v| v.as_str())
            .unwrap_or("HEAD");

        let format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("keep-a-changelog");

        // Get commit history for the range
        let commits = self.get_commit_history(context, from_tag, to_tag).await?;

        // Categorize commits
        let categorized_commits = self.categorize_commits(&commits).await?;

        // Generate changelog entry
        let changelog_entry = self
            .build_changelog_entry(version, &categorized_commits, format)
            .await?;

        // Format the changelog
        let formatted_changelog = self.format_changelog(&changelog_entry, format).await?;

        Ok(TaskResult::success_with_data(
            "Changelog generated successfully".to_string(),
            serde_json::json!({
                "changelog": changelog_entry,
                "formatted": formatted_changelog,
                "version": version,
                "commit_count": commits.len(),
            }),
        )
        .with_confidence(0.9))
    }

    /// Generate release notes
    async fn generate_release_notes(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        let version = params
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Version parameter required"))?;

        // First generate the changelog
        let changelog_result = self.generate_changelog(context, params).await?;
        let changelog_data = changelog_result
            .data
            .as_ref()
            .and_then(|d| d.get("changelog"))
            .ok_or_else(|| anyhow::anyhow!("Failed to generate changelog"))?;

        let changelog_entry: ChangelogEntry = serde_json::from_value(changelog_data.clone())?;

        // Generate release notes with enhanced narrative
        let release_notes = self.build_release_notes(version, &changelog_entry).await?;

        // Format release notes
        let formatted_notes = self.format_release_notes(&release_notes).await?;

        Ok(TaskResult::success_with_data(
            "Release notes generated successfully".to_string(),
            serde_json::json!({
                "release_notes": release_notes,
                "formatted": formatted_notes,
                "version": version,
            }),
        )
        .with_confidence(0.85))
    }

    /// Get commit history for a range
    async fn get_commit_history(
        &self,
        context: &AgentContext,
        from_tag: Option<&str>,
        to_tag: &str,
    ) -> Result<Vec<GitCommit>> {
        let git_tool = self
            .tools
            .iter()
            .find(|t| t.capabilities().contains(&"git".to_string()))
            .ok_or_else(|| anyhow::anyhow!("Git tool not found"))?;

        let mut git_params = HashMap::new();
        git_params.insert(
            "operation".to_string(),
            serde_json::Value::String("log".to_string()),
        );

        if let Some(from) = from_tag {
            let range = format!("{from}..{to_tag}");
            git_params.insert("commit_range".to_string(), serde_json::Value::String(range));
        }

        let log_result = git_tool.execute(context, &git_params).await?;

        // Parse commit log
        let log_content = log_result
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        self.parse_commit_log(log_content).await
    }

    /// Parse git log output into structured commits
    async fn parse_commit_log(&self, log: &str) -> Result<Vec<GitCommit>> {
        // Simplified commit parsing - in practice, use structured git log format
        let mut commits = Vec::new();
        let lines: Vec<&str> = log.lines().collect();

        let mut current_commit = None;

        for line in lines {
            if line.starts_with("commit ") {
                // Save previous commit
                if let Some(commit) = current_commit.take() {
                    commits.push(commit);
                }

                // Start new commit
                let hash = line
                    .strip_prefix("commit ")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                current_commit = Some(GitCommit {
                    hash,
                    message: String::new(),
                    author: String::new(),
                    date: String::new(),
                    files_changed: Vec::new(),
                });
            } else if line.starts_with("Author: ") && current_commit.is_some() {
                if let Some(ref mut commit) = current_commit {
                    commit.author = line
                        .strip_prefix("Author: ")
                        .unwrap_or("")
                        .trim()
                        .to_string();
                }
            } else if line.starts_with("Date: ") && current_commit.is_some() {
                if let Some(ref mut commit) = current_commit {
                    commit.date = line.strip_prefix("Date: ").unwrap_or("").trim().to_string();
                }
            } else if !line.trim().is_empty() && current_commit.is_some() {
                if let Some(ref mut commit) = current_commit {
                    if commit.message.is_empty() {
                        commit.message = line.trim().to_string();
                    }
                }
            }
        }

        // Don't forget the last commit
        if let Some(commit) = current_commit {
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Categorize commits based on conventional commit format and content
    async fn categorize_commits(&self, commits: &[GitCommit]) -> Result<CategorizedCommits> {
        let mut categorized = CategorizedCommits::new();

        for commit in commits {
            let category = self.classify_commit_type(&commit.message).await?;
            let change_item = ChangeItem {
                description: commit.message.clone(),
                commit_hash: Some(commit.hash.clone()),
                author: Some(commit.author.clone()),
                pr_number: self.extract_pr_number(&commit.message),
                impact: self.assess_change_impact(&commit.message).await?,
            };

            categorized.add_item(category, change_item);
        }

        Ok(categorized)
    }

    /// Classify commit type from message
    async fn classify_commit_type(&self, message: &str) -> Result<String> {
        let message_lower = message.to_lowercase();

        // Check conventional commit prefixes
        if message_lower.starts_with("feat") || message_lower.starts_with("feature") {
            return Ok("Added".to_string());
        }
        if message_lower.starts_with("fix") || message_lower.starts_with("bugfix") {
            return Ok("Fixed".to_string());
        }
        if message_lower.starts_with("docs") || message_lower.starts_with("doc") {
            return Ok("Documentation".to_string());
        }
        if message_lower.starts_with("refactor") || message_lower.starts_with("refact") {
            return Ok("Changed".to_string());
        }
        if message_lower.starts_with("perf") || message_lower.starts_with("performance") {
            return Ok("Performance".to_string());
        }
        if message_lower.starts_with("test") {
            return Ok("Testing".to_string());
        }
        if message_lower.starts_with("chore")
            || message_lower.starts_with("build")
            || message_lower.starts_with("ci")
        {
            return Ok("Maintenance".to_string());
        }
        if message_lower.contains("breaking") || message_lower.contains("!:") {
            return Ok("Breaking Changes".to_string());
        }
        if message_lower.starts_with("remove") || message_lower.starts_with("delete") {
            return Ok("Removed".to_string());
        }

        // Default categorization
        Ok("Changed".to_string())
    }

    /// Extract PR number from commit message
    fn extract_pr_number(&self, message: &str) -> Option<u32> {
        // Look for patterns like "(#123)" or "PR #123"
        let re = regex::Regex::new(r"(?:PR|#)\s*#?(\d+)").ok()?;
        re.captures(message)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Assess the impact level of a change
    async fn assess_change_impact(&self, message: &str) -> Result<ChangeImpact> {
        let message_lower = message.to_lowercase();

        if message_lower.contains("breaking") || message_lower.contains("!:") {
            return Ok(ChangeImpact::Major);
        }

        if message_lower.starts_with("feat") || message_lower.contains("new feature") {
            return Ok(ChangeImpact::Minor);
        }

        Ok(ChangeImpact::Patch)
    }

    /// Build changelog entry from categorized commits
    async fn build_changelog_entry(
        &self,
        version: &str,
        categorized: &CategorizedCommits,
        _format: &str,
    ) -> Result<ChangelogEntry> {
        Ok(ChangelogEntry {
            version: version.to_string(),
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            sections: categorized.sections.clone(),
            breaking_changes: categorized.breaking_changes.clone(),
            migration_notes: Vec::new(), // Would be generated based on breaking changes
        })
    }

    /// Build release notes from changelog entry
    async fn build_release_notes(
        &self,
        version: &str,
        changelog: &ChangelogEntry,
    ) -> Result<ReleaseNotes> {
        // Use Rig agent to generate narrative content
        let rig_agent = self.create_rig_agent().await?;
        let narrative_content = self
            .generate_narrative_content(&rig_agent, changelog)
            .await?;

        Ok(ReleaseNotes {
            version: version.to_string(),
            title: format!("Release {version}"),
            summary: narrative_content.summary,
            highlights: narrative_content.highlights,
            changes: changelog.clone(),
            acknowledgments: self.extract_contributors(changelog).await?,
        })
    }

    /// Create a Rig agent configured for changelog generation
    async fn create_rig_agent(&self) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let preamble = r"
You are an expert technical writer specializing in changelogs and release notes. Your role is to:

1. Transform technical commit messages into user-friendly descriptions
2. Create compelling release narratives that highlight value to users
3. Organize changes in a logical, easy-to-understand format
4. Identify the most important changes for highlighting
5. Write clear migration guides for breaking changes

Guidelines:
- Focus on user impact rather than technical implementation details
- Use present tense and active voice
- Group related changes together
- Highlight new features and improvements prominently
- Be clear about breaking changes and required actions
- Keep descriptions concise but informative

Follow semantic versioning principles and conventional changelog formats.
        ";

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .temperature(0.6) // Balanced creativity and consistency
                    .build();
                Ok(Box::new(agent))
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .temperature(0.6)
                    .build();
                Ok(Box::new(agent))
            }
        }
    }

    /// Generate narrative content using Rig agent
    async fn generate_narrative_content(
        &self,
        _rig_agent: &Box<dyn std::any::Any + Send + Sync>,
        changelog: &ChangelogEntry,
    ) -> Result<NarrativeContent> {
        // Use the backend directly to generate narrative content
        let narrative_result = self.generate_narrative_with_backend(changelog).await?;

        // Parse the result into structured format
        self.parse_narrative_result(&narrative_result, changelog)
            .await
    }

    /// Generate narrative content using the backend directly
    async fn generate_narrative_with_backend(&self, changelog: &ChangelogEntry) -> Result<String> {
        use rig::completion::Prompt;

        // Build context for the LLM
        let mut context = String::new();
        context.push_str(&format!("Version: {}\n", changelog.version));
        context.push_str(&format!("Date: {}\n\n", changelog.date));

        for (section, items) in &changelog.sections {
            if !items.is_empty() {
                context.push_str(&format!("{section}:\n"));
                for item in items.iter().take(10) {
                    // Limit to avoid overwhelming
                    context.push_str(&format!("- {}\n", item.description));
                }
                context.push('\n');
            }
        }

        let user_prompt = format!(
            "Create compelling release notes narrative from the following changelog data. Generate a summary and key highlights that focus on user value:\n\n{context}"
        );

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client.agent(model)
                    .preamble("You are an expert technical writer. Create engaging release notes that highlight user value and key improvements.")
                    .temperature(0.6)
                    .max_tokens(500)
                    .build();

                let response = agent
                    .prompt(&user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI API error: {}", e))?;

                Ok(response.trim().to_string())
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client.agent(model)
                    .preamble("You are an expert technical writer. Create engaging release notes that highlight user value and key improvements.")
                    .temperature(0.6)
                    .max_tokens(500)
                    .build();

                let response = agent
                    .prompt(&user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic API error: {}", e))?;

                Ok(response.trim().to_string())
            }
        }
    }

    /// Parse LLM response into structured narrative content
    async fn parse_narrative_result(
        &self,
        response: &str,
        changelog: &ChangelogEntry,
    ) -> Result<NarrativeContent> {
        // Extract summary and highlights from the response
        let lines: Vec<&str> = response.lines().collect();
        let mut summary = String::new();
        let mut highlights = Vec::new();

        // Try to identify summary and highlights in the response
        let mut in_highlights = false;

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.to_lowercase().contains("highlight") || line.to_lowercase().contains("key") {
                in_highlights = true;
                continue;
            }

            if in_highlights {
                if line.starts_with('-') || line.starts_with('*') || line.starts_with('•') {
                    highlights.push(
                        line.trim_start_matches('-')
                            .trim_start_matches('*')
                            .trim_start_matches('•')
                            .trim()
                            .to_string(),
                    );
                } else if !line.is_empty() {
                    highlights.push(line.to_string());
                }
            } else if summary.is_empty() {
                summary = line.to_string();
            }
        }

        // Fallback to basic highlights if none were extracted
        if highlights.is_empty() {
            if let Some(added) = changelog.sections.get("Added") {
                if !added.is_empty() {
                    highlights.push(format!("🎉 {} new features added", added.len()));
                }
            }

            if let Some(fixed) = changelog.sections.get("Fixed") {
                if !fixed.is_empty() {
                    highlights.push(format!("🐛 {} bugs fixed", fixed.len()));
                }
            }
        }

        // Fallback summary if none was extracted
        if summary.is_empty() {
            summary = format!(
                "This release includes {} changes across multiple areas",
                changelog
                    .sections
                    .values()
                    .map(std::vec::Vec::len)
                    .sum::<usize>()
            );
        }

        Ok(NarrativeContent {
            summary,
            highlights,
        })
    }

    /// Extract contributors from changelog
    async fn extract_contributors(&self, changelog: &ChangelogEntry) -> Result<Vec<String>> {
        let mut contributors = std::collections::HashSet::new();

        for section in changelog.sections.values() {
            for item in section {
                if let Some(author) = &item.author {
                    contributors.insert(author.clone());
                }
            }
        }

        Ok(contributors.into_iter().collect())
    }

    /// Format changelog entry as text
    async fn format_changelog(&self, changelog: &ChangelogEntry, format: &str) -> Result<String> {
        match format {
            "keep-a-changelog" => self.format_keep_a_changelog(changelog).await,
            "conventional" => self.format_conventional_changelog(changelog).await,
            _ => self.format_keep_a_changelog(changelog).await,
        }
    }

    /// Format using Keep a Changelog format
    async fn format_keep_a_changelog(&self, changelog: &ChangelogEntry) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "## [{}] - {}\n\n",
            changelog.version, changelog.date
        ));

        for (section, items) in &changelog.sections {
            if !items.is_empty() {
                output.push_str(&format!("### {section}\n\n"));
                for item in items {
                    output.push_str(&format!("- {}\n", item.description));
                }
                output.push('\n');
            }
        }

        if !changelog.breaking_changes.is_empty() {
            output.push_str("### Breaking Changes\n\n");
            for change in &changelog.breaking_changes {
                output.push_str(&format!("- {}\n", change.description));
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Format using conventional changelog format
    async fn format_conventional_changelog(&self, changelog: &ChangelogEntry) -> Result<String> {
        // Similar to keep-a-changelog but with different section ordering
        self.format_keep_a_changelog(changelog).await
    }

    /// Format release notes as text
    async fn format_release_notes(&self, notes: &ReleaseNotes) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!("# {}\n\n", notes.title));
        output.push_str(&format!("{}\n\n", notes.summary));

        if !notes.highlights.is_empty() {
            output.push_str("## Highlights\n\n");
            for highlight in &notes.highlights {
                output.push_str(&format!("- {highlight}\n"));
            }
            output.push('\n');
        }

        // Include the formatted changelog
        let changelog_text = self
            .format_changelog(&notes.changes, "keep-a-changelog")
            .await?;
        output.push_str(&changelog_text);

        if !notes.acknowledgments.is_empty() {
            output.push_str("## Contributors\n\n");
            output.push_str("Thanks to all contributors who made this release possible:\n\n");
            for contributor in &notes.acknowledgments {
                output.push_str(&format!("- {contributor}\n"));
            }
        }

        Ok(output)
    }
}

#[derive(Debug)]
struct GitCommit {
    hash: String,
    message: String,
    author: String,
    date: String,
    files_changed: Vec<String>,
}

#[derive(Debug)]
struct CategorizedCommits {
    sections: HashMap<String, Vec<ChangeItem>>,
    breaking_changes: Vec<BreakingChange>,
}

impl CategorizedCommits {
    fn new() -> Self {
        Self {
            sections: HashMap::new(),
            breaking_changes: Vec::new(),
        }
    }

    fn add_item(&mut self, category: String, item: ChangeItem) {
        self.sections.entry(category).or_default().push(item);
    }
}

#[derive(Debug)]
struct NarrativeContent {
    summary: String,
    highlights: Vec<String>,
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
        if !self.initialized {
            return Err(anyhow::anyhow!("Agent not initialized"));
        }

        match task {
            "generate_changelog" | "changelog_generation" => {
                self.generate_changelog(context, params).await
            }
            "generate_release_notes" | "release_notes_generation" => {
                self.generate_release_notes(context, params).await
            }
            "categorize_commits" | "commit_categorization" => {
                // Implement standalone commit categorization
                Ok(TaskResult::success(
                    "Commit categorization not yet implemented".to_string(),
                ))
            }
            _ => Err(anyhow::anyhow!("Unknown task: {}", task)),
        }
    }

    fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_changelog"
                | "changelog_generation"
                | "generate_release_notes"
                | "release_notes_generation"
                | "categorize_commits"
                | "commit_categorization"
        )
    }

    fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_changelog" | "changelog_generation" => 10,
            "generate_release_notes" | "release_notes_generation" => 9,
            "categorize_commits" | "commit_categorization" => 7,
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
}
