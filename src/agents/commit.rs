use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::{
    core::{AgentBackend, AgentContext, IrisAgent, TaskResult},
    tools::AgentTool,
};

/// Specialized agent for commit message generation and commit operations
pub struct CommitAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    backend: AgentBackend,
    tools: Vec<Arc<dyn AgentTool>>,
    initialized: bool,
}

impl CommitAgent {
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        Self {
            id: "commit_agent".to_string(),
            name: "Commit Agent".to_string(),
            description: "Specialized agent for generating commit messages and performing commit operations".to_string(),
            capabilities: vec![
                "commit_message_generation".to_string(),
                "diff_analysis".to_string(),
                "change_summarization".to_string(),
                "commit_validation".to_string(),
            ],
            backend,
            tools,
            initialized: false,
        }
    }

    /// Generate a commit message based on repository changes
    async fn generate_commit_message(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        // Extract parameters
        let preset = params.get("preset")
            .and_then(|v| v.as_str())
            .unwrap_or("conventional");
        let custom_instructions = params.get("instructions")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Get git diff using tools
        let mut git_params = HashMap::new();
        git_params.insert("operation".to_string(), serde_json::Value::String("diff".to_string()));
        
        let git_tool = self.tools.iter()
            .find(|t| t.capabilities().contains(&"git".to_string()))
            .ok_or_else(|| anyhow::anyhow!("Git tool not found"))?;
        
        let diff_result = git_tool.execute(context, &git_params).await?;
        let diff_content = diff_result.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if diff_content.is_empty() {
            return Ok(TaskResult::failure("No changes to commit".to_string()));
        }

        // Get file analysis for changed files
        let mut files_params = HashMap::new();
        files_params.insert("operation".to_string(), serde_json::Value::String("files".to_string()));
        
        let files_result = git_tool.execute(context, &files_params).await?;
        let empty_vec = vec![];
        let changed_files = files_result.get("content")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        // Analyze each changed file
        let mut file_analyses = Vec::new();
        if let Some(file_analyzer) = self.tools.iter()
            .find(|t| t.capabilities().contains(&"file_analysis".to_string())) {
            
            for file in changed_files.iter().take(10) { // Limit to avoid overwhelming
                if let Some(file_path) = file.as_str() {
                    let mut analysis_params = HashMap::new();
                    analysis_params.insert("path".to_string(), serde_json::Value::String(file_path.to_string()));
                    analysis_params.insert("analysis_type".to_string(), serde_json::Value::String("basic".to_string()));
                    
                    if let Ok(analysis) = file_analyzer.execute(context, &analysis_params).await {
                        file_analyses.push(analysis);
                    }
                }
            }
        }

        // Build context for the LLM
        let commit_context = self.build_commit_context(
            diff_content,
            &file_analyses,
            preset,
            custom_instructions,
        ).await?;

        // Use Rig agent to generate commit message
        let rig_agent = self.create_rig_agent().await?;
        
        // This is a simplified approach - in practice, we'd use the Rig agent more directly
        let commit_message = self.generate_with_rig_agent(&rig_agent, &commit_context).await?;

        // Validate and enhance the commit message
        let validated_message = self.validate_commit_message(&commit_message, context).await?;

        Ok(TaskResult::success_with_data(
            "Commit message generated successfully".to_string(),
            serde_json::json!({
                "commit_message": validated_message,
                "files_changed": changed_files.len(),
                "preset_used": preset,
            })
        ).with_confidence(0.85))
    }

    /// Build context for commit message generation
    async fn build_commit_context(
        &self,
        diff: &str,
        file_analyses: &[serde_json::Value],
        preset: &str,
        instructions: &str,
    ) -> Result<String> {
        let mut context = String::new();
        
        // Add preset-specific instructions
        context.push_str(&format!("Generate a commit message using the '{}' preset.\n\n", preset));
        
        if !instructions.is_empty() {
            context.push_str(&format!("Additional instructions: {}\n\n", instructions));
        }

        // Add file analysis summary
        if !file_analyses.is_empty() {
            context.push_str("Changed files analysis:\n");
            for analysis in file_analyses {
                if let (Some(path), Some(language)) = (
                    analysis.get("path").and_then(|v| v.as_str()),
                    analysis.get("language").and_then(|v| v.as_str())
                ) {
                    context.push_str(&format!("- {} ({})\n", path, language));
                }
            }
            context.push('\n');
        }

        // Add diff (truncated if too long)
        context.push_str("Changes:\n");
        if diff.len() > 4000 {
            context.push_str(&diff[..4000]);
            context.push_str("\n... (diff truncated for brevity)");
        } else {
            context.push_str(diff);
        }

        Ok(context)
    }

    /// Create a Rig agent configured for commit message generation
    async fn create_rig_agent(&self) -> Result<String> {
        let preamble = r#"
You are an expert Git commit message writer. Your role is to analyze code changes and generate clear, informative commit messages that follow best practices.

Guidelines:
1. Use conventional commit format when specified
2. Be concise but descriptive
3. Focus on the "what" and "why" of changes
4. Use imperative mood ("Add feature" not "Added feature")
5. Include scope when relevant
6. Mention breaking changes if any

Available formats:
- conventional: type(scope): description
- simple: Brief descriptive message
- detailed: Multi-line with body and footer
        "#;

        // This would use the backend to create the actual Rig agent
        // For now, return a placeholder
        match &self.backend {
            AgentBackend::OpenAI { model, .. } => {
                Ok(format!("OpenAI agent with model: {} and preamble: {}", model, preamble))
            }
            AgentBackend::Anthropic { model, .. } => {
                Ok(format!("Anthropic agent with model: {} and preamble: {}", model, preamble))
            }
        }
    }

    /// Generate commit message using Rig agent
    async fn generate_with_rig_agent(
        &self,
        _rig_agent: &str,
        context: &str,
    ) -> Result<String> {
        // This would use the Rig agent to generate the actual commit message
        // For now, return a placeholder
        Ok(format!("feat: implement agent-based commit message generation\n\nGenerated from context:\n{}", 
            context.chars().take(100).collect::<String>()))
    }

    /// Validate and enhance the generated commit message
    async fn validate_commit_message(
        &self,
        message: &str,
        _context: &AgentContext,
    ) -> Result<String> {
        // Basic validation and enhancement
        let mut validated = message.trim().to_string();
        
        // Ensure first line is not too long
        let lines: Vec<&str> = validated.lines().collect();
        if let Some(first_line) = lines.first() {
            if first_line.len() > 72 {
                // Truncate and add appropriate ending
                validated = format!("{}\n\n{}", 
                    &first_line[..69].trim_end(),
                    lines[1..].join("\n"));
            }
        }

        Ok(validated)
    }

    /// Analyze repository changes for commit preparation
    async fn analyze_changes(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        // Get current status and diff
        let git_tool = self.tools.iter()
            .find(|t| t.capabilities().contains(&"git".to_string()))
            .ok_or_else(|| anyhow::anyhow!("Git tool not found"))?;

        let mut status_params = HashMap::new();
        status_params.insert("operation".to_string(), serde_json::Value::String("status".to_string()));
        
        let status_result = git_tool.execute(context, &status_params).await?;

        let mut diff_params = HashMap::new();
        diff_params.insert("operation".to_string(), serde_json::Value::String("diff".to_string()));
        
        let diff_result = git_tool.execute(context, &diff_params).await?;

        Ok(TaskResult::success_with_data(
            "Repository analysis complete".to_string(),
            serde_json::json!({
                "status": status_result,
                "diff": diff_result,
                "analysis_timestamp": chrono::Utc::now().to_rfc3339(),
            })
        ))
    }
}

#[async_trait]
impl IrisAgent for CommitAgent {
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
            "generate_commit_message" => self.generate_commit_message(context, params).await,
            "analyze_changes" => self.analyze_changes(context, params).await,
            "validate_commit" => {
                // Implement commit validation logic
                Ok(TaskResult::success("Commit validation not yet implemented".to_string()))
            }
            _ => Err(anyhow::anyhow!("Unknown task: {}", task)),
        }
    }

    fn can_handle_task(&self, task: &str) -> bool {
        matches!(task, 
            "generate_commit_message" | 
            "analyze_changes" | 
            "validate_commit" |
            "commit_message_generation" |
            "diff_analysis"
        )
    }

    fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_commit_message" | "commit_message_generation" => 10,
            "analyze_changes" | "diff_analysis" => 8,
            "validate_commit" => 7,
            _ => 0,
        }
    }

    async fn initialize(&mut self, _context: &AgentContext) -> Result<()> {
        // Perform any initialization needed
        self.initialized = true;
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        // Clean up resources
        Ok(())
    }
}