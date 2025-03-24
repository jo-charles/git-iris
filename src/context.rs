use crate::token_optimizer::TokenOptimizer;
use colored::Colorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use textwrap::wrap;

#[derive(Serialize, Debug, Clone)]
pub struct CommitContext {
    pub branch: String,
    pub recent_commits: Vec<RecentCommit>,
    pub staged_files: Vec<StagedFile>,
    pub project_metadata: ProjectMetadata,
    pub user_name: String,
    pub user_email: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct RecentCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct StagedFile {
    pub path: String,
    pub change_type: ChangeType,
    pub diff: String,
    pub analysis: Vec<String>,
    pub content: Option<String>,
    pub content_excluded: bool,
}

/// Model for commit message generation results
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct GeneratedMessage {
    /// Optional emoji for the commit message
    pub emoji: Option<String>,
    /// Commit message title/subject line
    pub title: String,
    /// Detailed commit message body
    pub message: String,
}

/// Model for code review generation results
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct GeneratedReview {
    /// Brief summary of the code changes and overall review
    pub summary: String,
    /// Detailed assessment of the overall code quality
    pub code_quality: String,
    /// List of specific suggestions for improving the code
    pub suggestions: Vec<String>,
    /// List of identified issues or problems in the code
    pub issues: Vec<String>,
    /// List of positive aspects or good practices in the code
    pub positive_aspects: Vec<String>,
}

impl From<String> for GeneratedMessage {
    fn from(s: String) -> Self {
        match serde_json::from_str(&s) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse JSON: {e}\nInput was: {s}");
                Self {
                    emoji: None,
                    title: "Error parsing commit message".to_string(),
                    message: "There was an error parsing the commit message from the AI. Please try again.".to_string(),
                }
            }
        }
    }
}

impl From<String> for GeneratedReview {
    fn from(s: String) -> Self {
        match serde_json::from_str(&s) {
            Ok(review) => review,
            Err(e) => {
                crate::log_debug!("Failed to parse review JSON: {}", e);
                crate::log_debug!("Input was: {}", s);
                Self {
                    summary: "Error parsing code review".to_string(),
                    code_quality: "There was an error parsing the code review from the AI."
                        .to_string(),
                    suggestions: vec!["Please try again.".to_string()],
                    issues: vec![],
                    positive_aspects: vec![],
                }
            }
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Added => write!(f, "Added"),
            Self::Modified => write!(f, "Modified"),
            Self::Deleted => write!(f, "Deleted"),
        }
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct ProjectMetadata {
    pub language: Option<String>,
    pub framework: Option<String>,
    pub dependencies: Vec<String>,
    pub version: Option<String>,
    pub build_system: Option<String>,
    pub test_framework: Option<String>,
    pub plugins: Vec<String>,
}

impl ProjectMetadata {
    pub fn merge(&mut self, new: ProjectMetadata) {
        if let Some(new_lang) = new.language {
            match &mut self.language {
                Some(lang) if !lang.contains(&new_lang) => {
                    lang.push_str(", ");
                    lang.push_str(&new_lang);
                }
                None => self.language = Some(new_lang),
                _ => {}
            }
        }
        self.dependencies.extend(new.dependencies.clone());
        self.framework = self.framework.take().or(new.framework);
        self.version = self.version.take().or(new.version);
        self.build_system = self.build_system.take().or(new.build_system);
        self.test_framework = self.test_framework.take().or(new.test_framework);
        self.plugins.extend(new.plugins);
        self.dependencies.sort();
        self.dependencies.dedup();
    }
}

impl CommitContext {
    pub fn new(
        branch: String,
        recent_commits: Vec<RecentCommit>,
        staged_files: Vec<StagedFile>,
        project_metadata: ProjectMetadata,
        user_name: String,
        user_email: String,
    ) -> Self {
        Self {
            branch,
            recent_commits,
            staged_files,
            project_metadata,
            user_name,
            user_email,
        }
    }
    pub fn optimize(&mut self, max_tokens: usize) {
        let optimizer = TokenOptimizer::new(max_tokens);
        optimizer.optimize_context(self);
    }
}

/// Formats a commit message from a `GeneratedMessage`
pub fn format_commit_message(response: &GeneratedMessage) -> String {
    let mut message = String::new();

    if let Some(emoji) = &response.emoji {
        message.push_str(&format!("{emoji} "));
    }

    message.push_str(&response.title);
    message.push_str("\n\n");

    let wrapped_message = wrap(&response.message, 78);
    for line in wrapped_message {
        message.push_str(&line);
        message.push('\n');
    }

    message
}

impl GeneratedReview {
    /// Formats the review into a readable string with colors and emojis for terminal display
    pub fn format(&self) -> String {
        let mut formatted = String::new();

        formatted.push_str(&format!(
            "{}\n\n{}\n\n",
            "✨ Code Review Summary ✨".bright_magenta().bold(),
            self.summary.bright_white()
        ));

        formatted.push_str(&format!(
            "{}\n\n{}\n\n",
            "🔍 Code Quality Assessment".bright_cyan().bold(),
            self.code_quality.bright_white()
        ));

        if !self.positive_aspects.is_empty() {
            formatted.push_str(&format!("{}\n\n", "✅ Positive Aspects".green().bold()));
            for (i, aspect) in self.positive_aspects.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, aspect.green()));
            }
            formatted.push('\n');
        }

        if !self.issues.is_empty() {
            formatted.push_str(&format!("{}\n\n", "❌ Issues Identified".yellow().bold()));
            for (i, issue) in self.issues.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, issue.yellow()));
            }
            formatted.push('\n');
        }

        if !self.suggestions.is_empty() {
            formatted.push_str(&format!(
                "{}\n\n",
                "💡 Suggestions for Improvement".bright_blue().bold()
            ));
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, suggestion.bright_blue()));
            }
        }

        formatted
    }
}
