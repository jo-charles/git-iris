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

/// Specialized agent for code review and analysis
pub struct ReviewAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    backend: AgentBackend,
    tools: Vec<Arc<dyn AgentTool>>,
    initialized: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewResult {
    pub overall_score: f32,
    pub summary: String,
    pub issues: Vec<ReviewIssue>,
    pub suggestions: Vec<ReviewSuggestion>,
    pub approval_recommendation: ApprovalRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSuggestion {
    pub category: SuggestionCategory,
    pub description: String,
    pub impact: ImpactLevel,
    pub effort: EffortLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    Maintainability,
    Reliability,
    Documentation,
    Style,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Architecture,
    Refactoring,
    Optimization,
    Documentation,
    Testing,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalRecommendation {
    Approve,
    ApproveWithComments,
    RequestChanges,
    NeedsDiscussion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    High,
    Medium,
    Low,
}

impl ReviewAgent {
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        let mut agent = Self {
            id: "review_agent".to_string(),
            name: "Review Agent".to_string(),
            description:
                "Specialized agent for code review, quality analysis, and improvement suggestions"
                    .to_string(),
            capabilities: vec![
                "code_review".to_string(),
                "quality_analysis".to_string(),
                "security_analysis".to_string(),
                "performance_analysis".to_string(),
                "documentation_review".to_string(),
            ],
            backend,
            tools,
            initialized: false,
        };

        // Initialize the agent
        agent.initialized = true;
        agent
    }

    /// Perform comprehensive code review
    async fn review_code(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        // Extract parameters
        let _review_type = params
            .get("review_type")
            .and_then(|v| v.as_str())
            .unwrap_or("comprehensive");
        let focus_areas = params
            .get("focus_areas")
            .and_then(|v| v.as_array())
            .map_or_else(
                || vec!["security", "performance", "maintainability"],
                |arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>(),
            );

        // Get changes to review
        let changes = self.get_changes_for_review(context, params).await?;

        // Analyze each file
        let mut file_reviews = Vec::new();
        for change in &changes {
            let file_review = self.review_file(context, change).await?;
            file_reviews.push(file_review);
        }

        // Perform overall analysis
        let overall_review = self.synthesize_review(&file_reviews, &focus_areas).await?;

        Ok(TaskResult::success_with_data(
            "Code review completed".to_string(),
            serde_json::to_value(overall_review)?,
        )
        .with_confidence(0.88))
    }

    /// Get changes that need to be reviewed
    async fn get_changes_for_review(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<FileChange>> {
        let git_tool = self
            .tools
            .iter()
            .find(|t| t.capabilities().contains(&"git".to_string()))
            .ok_or_else(|| anyhow::anyhow!("Git tool not found"))?;

        // Get diff or specific commit range
        let mut git_params = HashMap::new();
        if let Some(commit_range) = params.get("commit_range").and_then(|v| v.as_str()) {
            git_params.insert(
                "operation".to_string(),
                serde_json::Value::String("diff".to_string()),
            );
            git_params.insert(
                "commit_range".to_string(),
                serde_json::Value::String(commit_range.to_string()),
            );
        } else {
            git_params.insert(
                "operation".to_string(),
                serde_json::Value::String("diff".to_string()),
            );
        }

        let diff_result = git_tool.execute(context, &git_params).await?;

        // Parse diff into file changes
        let diff_content = diff_result
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        self.parse_diff_to_changes(diff_content).await
    }

    /// Parse git diff into structured file changes
    async fn parse_diff_to_changes(&self, diff: &str) -> Result<Vec<FileChange>> {
        // Simplified diff parsing - in practice, use a proper diff parser
        let mut changes = Vec::new();
        let lines: Vec<&str> = diff.lines().collect();

        let mut current_file = None;
        let mut added_lines = 0;
        let mut removed_lines = 0;

        for line in lines {
            if line.starts_with("diff --git") {
                // Save previous file if exists
                if let Some(file_path) = current_file.take() {
                    changes.push(FileChange {
                        path: file_path,
                        change_type: ChangeType::Modified,
                        added_lines,
                        removed_lines,
                        content: diff.to_string(), // Simplified - should extract just this file's diff
                    });
                }

                // Extract new file path
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    current_file = Some(parts[3].trim_start_matches("b/").to_string());
                    added_lines = 0;
                    removed_lines = 0;
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                added_lines += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                removed_lines += 1;
            }
        }

        // Don't forget the last file
        if let Some(file_path) = current_file {
            changes.push(FileChange {
                path: file_path,
                change_type: ChangeType::Modified,
                added_lines,
                removed_lines,
                content: diff.to_string(),
            });
        }

        Ok(changes)
    }

    /// Review a single file change
    async fn review_file(&self, context: &AgentContext, change: &FileChange) -> Result<FileReview> {
        // Get file analysis
        let file_analyzer = self
            .tools
            .iter()
            .find(|t| t.capabilities().contains(&"file_analysis".to_string()));

        let mut file_info = None;
        if let Some(analyzer) = file_analyzer {
            let mut analysis_params = HashMap::new();
            analysis_params.insert(
                "path".to_string(),
                serde_json::Value::String(change.path.clone()),
            );
            analysis_params.insert(
                "analysis_type".to_string(),
                serde_json::Value::String("detailed".to_string()),
            );

            if let Ok(analysis) = analyzer.execute(context, &analysis_params).await {
                file_info = Some(analysis);
            }
        }

        // Create review context
        let review_context = self
            .build_review_context(change, file_info.as_ref())
            .await?;

        // Use Rig agent for analysis
        let rig_agent = self.create_rig_agent().await?;
        let review_result = self
            .analyze_with_rig_agent(&rig_agent, &review_context)
            .await?;

        Ok(FileReview {
            file_path: change.path.clone(),
            change_type: change.change_type.clone(),
            issues: review_result.issues,
            suggestions: review_result.suggestions,
            score: review_result.score,
        })
    }

    /// Build context for file review
    async fn build_review_context(
        &self,
        change: &FileChange,
        file_info: Option<&serde_json::Value>,
    ) -> Result<String> {
        let mut context = String::new();

        context.push_str(&format!("Reviewing file: {}\n", change.path));
        context.push_str(&format!("Change type: {:?}\n", change.change_type));
        context.push_str(&format!(
            "Lines added: {}, removed: {}\n\n",
            change.added_lines, change.removed_lines
        ));

        if let Some(info) = file_info {
            if let Some(language) = info.get("language").and_then(|v| v.as_str()) {
                context.push_str(&format!("Language: {language}\n"));
            }
            if let Some(complexity) = info.get("complexity") {
                context.push_str(&format!("Complexity: {complexity:?}\n"));
            }
        }

        context.push_str("\nChanges:\n");
        context.push_str(&change.content);

        Ok(context)
    }

    /// Create a Rig agent configured for code review
    async fn create_rig_agent(&self) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let preamble = r"
You are an expert code reviewer with deep knowledge of software engineering best practices, security, performance, and maintainability.

Your role is to:
1. Identify potential issues in code changes
2. Suggest improvements for code quality
3. Check for security vulnerabilities
4. Evaluate performance implications
5. Assess maintainability and readability
6. Provide constructive feedback

Focus areas:
- Security: Look for vulnerabilities, unsafe practices, input validation
- Performance: Identify bottlenecks, inefficient algorithms, resource usage
- Maintainability: Code clarity, documentation, modularity, naming
- Reliability: Error handling, edge cases, testing coverage
- Best Practices: Language-specific conventions, design patterns

Provide specific, actionable feedback with examples when possible.
        ";

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .temperature(0.3) // Lower temperature for more consistent analysis
                    .build();
                Ok(Box::new(agent))
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .temperature(0.3)
                    .build();
                Ok(Box::new(agent))
            }
        }
    }

    /// Analyze file with Rig agent
    async fn analyze_with_rig_agent(
        &self,
        _rig_agent: &Box<dyn std::any::Any + Send + Sync>,
        context: &str,
    ) -> Result<AnalysisResult> {
        // Use the backend directly to make LLM calls
        let analysis_result = self.analyze_with_backend(context).await?;

        // Parse the result into structured format
        self.parse_analysis_result(&analysis_result).await
    }

    /// Analyze file using the backend directly
    async fn analyze_with_backend(&self, context: &str) -> Result<String> {
        use rig::completion::Prompt;

        let user_prompt = format!(
            "Perform a comprehensive code review of the following changes. Focus on security, performance, maintainability, and best practices:\n\n{context}"
        );

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client.agent(model)
                    .preamble("You are an expert code reviewer. Analyze code changes and provide structured feedback with specific issues and suggestions.")
                    .temperature(0.3)
                    .max_tokens(800)
                    .build();

                let response = agent
                    .prompt(&user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI API error: {}", e))?;

                Ok(response.trim().to_string())
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client.agent(model)
                    .preamble("You are an expert code reviewer. Analyze code changes and provide structured feedback with specific issues and suggestions.")
                    .temperature(0.3)
                    .max_tokens(800)
                    .build();

                let response = agent
                    .prompt(&user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic API error: {}", e))?;

                Ok(response.trim().to_string())
            }
        }
    }

    /// Parse LLM response into structured analysis result
    async fn parse_analysis_result(&self, response: &str) -> Result<AnalysisResult> {
        // For now, create a basic structured response
        // In a real implementation, this would parse the LLM response more sophisticatedly
        let score = if response.contains("critical") || response.contains("severe") {
            5.0
        } else if response.contains("issue") || response.contains("problem") {
            7.0
        } else if response.contains("suggestion") || response.contains("improve") {
            8.5
        } else {
            9.0
        };

        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Extract issues and suggestions from the response
        if response.contains("security") {
            issues.push(ReviewIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Security,
                description: "Security consideration identified in code review".to_string(),
                file_path: None,
                line_number: None,
                suggestion: Some(
                    "Review security implications and implement appropriate safeguards".to_string(),
                ),
            });
        }

        if response.contains("performance") {
            suggestions.push(ReviewSuggestion {
                category: SuggestionCategory::Optimization,
                description: "Performance optimization opportunity identified".to_string(),
                impact: ImpactLevel::Medium,
                effort: EffortLevel::Low,
            });
        }

        if response.contains("test") {
            suggestions.push(ReviewSuggestion {
                category: SuggestionCategory::Testing,
                description: "Consider adding tests for new functionality".to_string(),
                impact: ImpactLevel::High,
                effort: EffortLevel::Medium,
            });
        }

        Ok(AnalysisResult {
            score,
            issues,
            suggestions,
        })
    }

    /// Synthesize individual file reviews into overall review
    async fn synthesize_review(
        &self,
        file_reviews: &[FileReview],
        _focus_areas: &[&str],
    ) -> Result<ReviewResult> {
        let mut all_issues = Vec::new();
        let mut all_suggestions = Vec::new();
        let mut total_score = 0.0;

        for review in file_reviews {
            all_issues.extend(review.issues.clone());
            all_suggestions.extend(review.suggestions.clone());
            total_score += review.score;
        }

        let overall_score = if file_reviews.is_empty() {
            8.0
        } else {
            total_score / file_reviews.len() as f32
        };

        let approval_recommendation = match overall_score {
            score if score >= 8.5 => ApprovalRecommendation::Approve,
            score if score >= 7.0 => ApprovalRecommendation::ApproveWithComments,
            score if score >= 5.0 => ApprovalRecommendation::RequestChanges,
            _ => ApprovalRecommendation::NeedsDiscussion,
        };

        Ok(ReviewResult {
            overall_score,
            summary: format!(
                "Reviewed {} files with overall score of {:.1}/10",
                file_reviews.len(),
                overall_score
            ),
            issues: all_issues,
            suggestions: all_suggestions,
            approval_recommendation,
        })
    }
}

#[derive(Debug, Clone)]
struct FileChange {
    path: String,
    change_type: ChangeType,
    added_lines: u32,
    removed_lines: u32,
    content: String,
}

#[derive(Debug, Clone)]
enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug)]
struct FileReview {
    file_path: String,
    change_type: ChangeType,
    issues: Vec<ReviewIssue>,
    suggestions: Vec<ReviewSuggestion>,
    score: f32,
}

#[derive(Debug)]
struct AnalysisResult {
    score: f32,
    issues: Vec<ReviewIssue>,
    suggestions: Vec<ReviewSuggestion>,
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
        if !self.initialized {
            return Err(anyhow::anyhow!("Agent not initialized"));
        }

        match task {
            "review_code" | "code_review" => self.review_code(context, params).await,
            "analyze_security" => {
                // Implement security-specific analysis
                Ok(TaskResult::success(
                    "Security analysis not yet implemented".to_string(),
                ))
            }
            "analyze_performance" => {
                // Implement performance-specific analysis
                Ok(TaskResult::success(
                    "Performance analysis not yet implemented".to_string(),
                ))
            }
            _ => Err(anyhow::anyhow!("Unknown task: {}", task)),
        }
    }

    fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "review_code"
                | "code_review"
                | "analyze_security"
                | "analyze_performance"
                | "quality_analysis"
        )
    }

    fn task_priority(&self, task: &str) -> u8 {
        match task {
            "review_code" | "code_review" => 10,
            "analyze_security" => 9,
            "analyze_performance" => 8,
            "quality_analysis" => 7,
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
