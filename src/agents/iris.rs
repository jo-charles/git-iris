use anyhow::Result;
use async_trait::async_trait;
use rig::client::CompletionClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::{
    core::{AgentBackend, AgentContext, TaskResult},
    tools::AgentTool,
};
use crate::commit::types::{GeneratedMessage, GeneratedPullRequest};
use crate::commit::review::GeneratedReview;
use crate::commit::prompt::{create_system_prompt, create_user_prompt, create_review_system_prompt, create_review_user_prompt, create_pr_system_prompt, create_pr_user_prompt};
use crate::context::CommitContext;
use crate::log_debug;

/// The unified Iris agent - an AI assistant for all Git-Iris operations
pub struct IrisAgent {
    id: String,
    name: String,
    description: String,
    capabilities: Vec<String>,
    backend: AgentBackend,
    tools: Vec<Arc<dyn AgentTool>>,
    initialized: bool,
}

/// Intelligence context gathered through LLM analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentContext {
    pub files_with_relevance: Vec<FileRelevance>,
    pub change_summary: String,
    pub technical_analysis: String,
    pub project_insights: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRelevance {
    pub path: String,
    pub relevance_score: f32,
    pub analysis: String,
    pub key_changes: Vec<String>,
    pub impact_assessment: String,
}

impl IrisAgent {
    pub fn new(backend: AgentBackend, tools: Vec<Arc<dyn AgentTool>>) -> Self {
        let mut agent = Self {
            id: "iris_agent".to_string(),
            name: "Iris".to_string(),
            description: "AI assistant for intelligent Git workflow automation and analysis".to_string(),
            capabilities: vec![
                "commit_message_generation".to_string(),
                "code_review".to_string(),
                "pull_request_description".to_string(),
                "changelog_generation".to_string(),
                "file_analysis".to_string(),
                "relevance_scoring".to_string(),
                "intelligent_context_gathering".to_string(),
                "diff_analysis".to_string(),
                "change_summarization".to_string(),
                "commit_validation".to_string(),
                "security_analysis".to_string(),
                "performance_analysis".to_string(),
                "documentation_review".to_string(),
            ],
            backend,
            tools,
            initialized: false,
        };

        agent.initialized = true;
        agent
    }

    /// Generate a commit message with intelligent context analysis
    pub async fn generate_commit_message(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        let preset = params
            .get("preset")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        let _custom_instructions = params
            .get("instructions")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let use_gitmoji = params
            .get("gitmoji")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(context.config.use_gitmoji);

        log_debug!(
            "🤖 Iris: Generating commit message with preset: '{}', gitmoji: {}",
            preset,
            use_gitmoji
        );

        // Step 1: Gather intelligent context using LLM analysis
        log_debug!("🧠 Iris: Gathering intelligent context through LLM analysis");
        let intelligent_context = self.gather_intelligent_context(context).await?;
        
        // Step 2: Build Git context from intelligent analysis
        let commit_context = self.build_commit_context(context, &intelligent_context).await?;

        // Step 3: Generate commit message using existing prompt system
        log_debug!("📝 Iris: Building system and user prompts");
        let system_prompt = create_system_prompt(&context.config)?;
        let user_prompt = create_user_prompt(&commit_context);

        log_debug!(
            "📏 Iris: Prompts built - System: {} chars, User: {} chars",
            system_prompt.len(),
            user_prompt.len()
        );

        // Step 4: Generate with LLM
        log_debug!("🤖 Iris: Generating commit message with LLM");
        let generated_message = self.generate_with_backend(&system_prompt, &user_prompt).await?;

        // Step 5: Parse and validate response
        let parsed_response = self.parse_json_response::<GeneratedMessage>(&generated_message).await?;

        log_debug!(
            "✅ Iris: Commit message generated - Title: '{}', {} chars total",
            parsed_response.title,
            parsed_response.message.len()
        );

        Ok(TaskResult::success_with_data(
            "Commit message generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.92))
    }

    /// Generate a code review with intelligent analysis
    pub async fn generate_code_review(
        &self,
        context: &AgentContext,
        _params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("🔍 Iris: Starting intelligent code review");

        // Gather intelligent context
        let intelligent_context = self.gather_intelligent_context(context).await?;
        let commit_context = self.build_commit_context(context, &intelligent_context).await?;

        // Generate review using existing prompt system
        let system_prompt = create_review_system_prompt(&context.config)?;
        let user_prompt = create_review_user_prompt(&commit_context);

        let generated_review = self.generate_with_backend(&system_prompt, &user_prompt).await?;
        let parsed_response = self.parse_json_response::<GeneratedReview>(&generated_review).await?;

        log_debug!("✅ Iris: Code review completed with {} issues", parsed_response.issues.len());

        Ok(TaskResult::success_with_data(
            "Code review generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.88))
    }

    /// Generate a pull request description with intelligent analysis
    pub async fn generate_pull_request(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<TaskResult> {
        log_debug!("📋 Iris: Starting pull request description generation");

        // Get commit messages from params
        let commit_messages = params
            .get("commit_messages")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Gather intelligent context
        let intelligent_context = self.gather_intelligent_context(context).await?;
        let commit_context = self.build_commit_context(context, &intelligent_context).await?;

        // Generate PR description using existing prompt system
        let system_prompt = create_pr_system_prompt(&context.config)?;
        let user_prompt = create_pr_user_prompt(&commit_context, &commit_messages);

        let generated_pr = self.generate_with_backend(&system_prompt, &user_prompt).await?;
        let parsed_response = self.parse_json_response::<GeneratedPullRequest>(&generated_pr).await?;

        log_debug!("✅ Iris: PR description generated - Title: '{}'", parsed_response.title);

        Ok(TaskResult::success_with_data(
            "Pull request description generated successfully".to_string(),
            serde_json::to_value(parsed_response)?,
        )
        .with_confidence(0.90))
    }

    /// Gather intelligent context using LLM analysis instead of deterministic rules
    async fn gather_intelligent_context(&self, context: &AgentContext) -> Result<IntelligentContext> {
        log_debug!("🧠 Iris: Starting intelligent context analysis");

        // Step 1: Get raw Git data
        log_debug!("📊 Iris: Gathering Git context data");
        let git_context = context.git_repo.get_git_info(&context.config).await?;
        log_debug!("📊 Iris: Found {} staged files to analyze", git_context.staged_files.len());
        
        // Step 2: Use LLM to analyze and score relevance
        log_debug!("🛠️ Iris: Building context analysis prompt");
        let analysis_prompt = self.build_context_analysis_prompt(&git_context).await?;
        log_debug!("🛠️ Iris: Analysis prompt built - {} chars", analysis_prompt.len());
        
        log_debug!("🤖 Iris: Sending context analysis request to LLM");
        let intelligence_result = self.analyze_with_backend(&analysis_prompt).await?;
        log_debug!("🤖 Iris: LLM analysis response received - {} chars", intelligence_result.len());
        
        // Step 3: Parse LLM analysis into structured context
        log_debug!("🔍 Iris: Parsing LLM analysis result");
        let intelligent_context = self.parse_intelligence_result(&intelligence_result, &git_context).await?;

        log_debug!(
            "✅ Iris: Intelligent context gathered - {} files analyzed, avg relevance: {:.2}",
            intelligent_context.files_with_relevance.len(),
            if intelligent_context.files_with_relevance.is_empty() {
                0.0
            } else {
                intelligent_context.files_with_relevance.iter()
                    .map(|f| f.relevance_score)
                    .sum::<f32>() / intelligent_context.files_with_relevance.len() as f32
            }
        );

        Ok(intelligent_context)
    }

    /// Build context analysis prompt for LLM
    async fn build_context_analysis_prompt(&self, git_context: &CommitContext) -> Result<String> {
        log_debug!("🛠️ Iris: Building context analysis prompt for {} files", git_context.staged_files.len());
        
        let mut prompt = String::from(
            "You are an expert software engineer analyzing Git changes for context and relevance. \
            Your task is to intelligently analyze the provided changes and score their relevance \
            to understanding the overall purpose and impact of this commit.\n\n\
            For each file, provide:\n\
            1. Relevance score (0.0-1.0) based on how important this file is to understanding the change\n\
            2. Analysis of what changed and why it matters\n\
            3. Key changes that are most significant\n\
            4. Impact assessment on the overall system\n\n\
            Also provide:\n\
            - Overall change summary (what is the main purpose)\n\
            - Technical analysis (implementation details, patterns, architecture)\n\
            - Project insights (how this fits into the larger codebase)\n\n\
            Files to analyze:\n\n"
        );

        for (index, file) in git_context.staged_files.iter().enumerate() {
            log_debug!("📄 Iris: Adding file {} to analysis prompt: {}", index + 1, file.path);
            prompt.push_str(&format!(
                "=== FILE {} ===\n\
                Path: {}\n\
                Change Type: {:?}\n\
                Diff:\n{}\n\n",
                index + 1,
                file.path,
                file.change_type,
                file.diff
            ));

            if let Some(content) = &file.content {
                log_debug!("📄 Iris: Including full content for file: {} ({} chars)", file.path, content.len());
                prompt.push_str(&format!(
                    "Full Content:\n{}\n\
                    --- End of File ---\n\n",
                    content
                ));
            }
        }

        prompt.push_str(
            "\nRespond with a JSON object in this exact format:\n\
            {\n\
              \"files\": [\n\
                {\n\
                  \"path\": \"file_path\",\n\
                  \"relevance_score\": 0.85,\n\
                  \"analysis\": \"What changed and why it matters\",\n\
                  \"key_changes\": [\"change 1\", \"change 2\"],\n\
                  \"impact_assessment\": \"How this affects the system\"\n\
                }\n\
              ],\n\
              \"change_summary\": \"Overall purpose of these changes\",\n\
              \"technical_analysis\": \"Implementation details and patterns\",\n\
              \"project_insights\": \"How this fits into the larger codebase\"\n\
            }"
        );

        log_debug!("🛠️ Iris: Context analysis prompt complete - {} chars total", prompt.len());
        Ok(prompt)
    }

    /// Parse LLM intelligence result into structured context
    async fn parse_intelligence_result(
        &self,
        result: &str,
        git_context: &CommitContext,
    ) -> Result<IntelligentContext> {
        log_debug!("🔍 Iris: Parsing LLM analysis result: {}", result.chars().take(200).collect::<String>());
        
        // Try to parse JSON response
        log_debug!("🔍 Iris: Attempting to parse JSON response");
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result) {
            log_debug!("✅ Iris: Successfully parsed LLM analysis JSON");
            
            let files_with_relevance: Vec<FileRelevance> = parsed
                .get("files")
                .and_then(|f| f.as_array())
                .map(|files| {
                    log_debug!("📊 Iris: Processing {} file analyses from LLM", files.len());
                    files
                        .iter()
                        .enumerate()
                        .filter_map(|(i, file)| {
                            let file_result = Some(FileRelevance {
                                path: file.get("path")?.as_str()?.to_string(),
                                relevance_score: file.get("relevance_score")?.as_f64()? as f32,
                                analysis: file.get("analysis")?.as_str()?.to_string(),
                                key_changes: file
                                    .get("key_changes")?
                                    .as_array()?
                                    .iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect(),
                                impact_assessment: file.get("impact_assessment")?.as_str()?.to_string(),
                            });
                            
                            if let Some(ref fr) = file_result {
                                log_debug!("📄 Iris: File {} analysis - relevance: {:.2}, {} key changes", 
                                    fr.path, fr.relevance_score, fr.key_changes.len());
                            } else {
                                log_debug!("⚠️ Iris: Failed to parse file analysis #{}", i + 1);
                            }
                            
                            file_result
                        })
                        .collect()
                })
                .unwrap_or_default();

            let change_summary = parsed
                .get("change_summary")
                .and_then(|v| v.as_str())
                .unwrap_or("Changes analyzed")
                .to_string();

            let technical_analysis = parsed
                .get("technical_analysis")
                .and_then(|v| v.as_str())
                .unwrap_or("Technical implementation details")
                .to_string();

            let project_insights = parsed
                .get("project_insights")
                .and_then(|v| v.as_str())
                .unwrap_or("Project context and fit")
                .to_string();

            log_debug!("✅ Iris: Successfully parsed intelligent context with {} file analyses", files_with_relevance.len());
            log_debug!("📝 Iris: Change summary: {}", change_summary.chars().take(100).collect::<String>());

            Ok(IntelligentContext {
                files_with_relevance,
                change_summary,
                technical_analysis,
                project_insights,
            })
        } else {
            // Fallback: create basic context with equal relevance
            log_debug!("⚠️ Iris: Failed to parse LLM analysis JSON, using fallback context");
            log_debug!("⚠️ Iris: JSON parsing error - raw response: {}", result.chars().take(500).collect::<String>());
            
            let files_with_relevance: Vec<FileRelevance> = git_context
                .staged_files
                .iter()
                .enumerate()
                .map(|(i, file)| {
                    log_debug!("📄 Iris: Creating fallback analysis for file {}: {}", i + 1, file.path);
                    FileRelevance {
                        path: file.path.clone(),
                        relevance_score: 0.7, // Default relevance
                        analysis: format!("File {} was modified", file.path),
                        key_changes: vec!["Content changes detected".to_string()],
                        impact_assessment: "Part of the overall changeset".to_string(),
                    }
                })
                .collect();

            log_debug!("⚠️ Iris: Created fallback context with {} files", files_with_relevance.len());

            Ok(IntelligentContext {
                files_with_relevance,
                change_summary: "Multiple files changed".to_string(),
                technical_analysis: "Various technical changes applied".to_string(),
                project_insights: "Changes contribute to project evolution".to_string(),
            })
        }
    }

    /// Build commit context from intelligent analysis
    async fn build_commit_context(
        &self,
        context: &AgentContext,
        intelligent_context: &IntelligentContext,
    ) -> Result<CommitContext> {
        log_debug!("🏗️ Iris: Building commit context from intelligent analysis");
        
        let git_context = context.git_repo.get_git_info(&context.config).await?;
        log_debug!("🏗️ Iris: Git context retrieved with {} staged files", git_context.staged_files.len());
        
        // The git_context already contains the staged files we need
        // We just need to enhance them with intelligent analysis
        let mut staged_files = git_context.staged_files.clone();
        
        // Enhance with intelligent analysis
        let mut enhanced_count = 0;
        for staged_file in &mut staged_files {
            if let Some(relevance_info) = intelligent_context
                .files_with_relevance
                .iter()
                .find(|f| f.path == staged_file.path)
            {
                // Replace analysis with intelligent insights
                staged_file.analysis = relevance_info.key_changes.clone();
                enhanced_count += 1;
                log_debug!("🔧 Iris: Enhanced file {} with {} key changes (relevance: {:.2})", 
                    staged_file.path, relevance_info.key_changes.len(), relevance_info.relevance_score);
            } else {
                log_debug!("⚠️ Iris: No intelligent analysis found for file: {}", staged_file.path);
            }
        }

        log_debug!("🏗️ Iris: Enhanced {} of {} files with intelligent analysis", enhanced_count, staged_files.len());

        // Return the enhanced git_context with intelligent analysis
        Ok(CommitContext {
            branch: git_context.branch,
            staged_files,
            recent_commits: git_context.recent_commits,
            project_metadata: git_context.project_metadata,
            user_name: git_context.user_name,
            user_email: git_context.user_email,
        })
    }

    /// Generate text using the configured backend
    async fn generate_with_backend(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        use rig::completion::Prompt;

        log_debug!("🔮 Iris: Preparing LLM request with backend");
        log_debug!("📊 System prompt: {} chars", system_prompt.len());
        log_debug!("👤 User prompt: {} chars", user_prompt.len());

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                log_debug!("🤖 Using OpenAI backend with model: {}", model);
                let agent = client
                    .agent(model)
                    .preamble(system_prompt)
                    .temperature(0.7)
                    .max_tokens(800)
                    .build();

                log_debug!("🚀 Sending OpenAI API request...");
                let response = agent
                    .prompt(user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI API error: {}", e))?;

                log_debug!("✅ OpenAI response received: {} chars", response.len());
                log_debug!("📝 Response preview: {}", response.chars().take(100).collect::<String>());
                Ok(response.trim().to_string())
            }
            AgentBackend::Anthropic { client, model } => {
                log_debug!("🧠 Using Anthropic backend with model: {}", model);
                let agent = client
                    .agent(model)
                    .preamble(system_prompt)
                    .temperature(0.7)
                    .max_tokens(800)
                    .build();

                log_debug!("🚀 Sending Anthropic API request...");
                let response = agent
                    .prompt(user_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic API error: {}", e))?;

                log_debug!("✅ Anthropic response received: {} chars", response.len());
                log_debug!("📝 Response preview: {}", response.chars().take(100).collect::<String>());
                Ok(response.trim().to_string())
            }
        }
    }

    /// Analyze context using the backend (for intelligence gathering)
    async fn analyze_with_backend(&self, prompt: &str) -> Result<String> {
        use rig::completion::Prompt;

        let system_prompt = "You are an expert software engineer analyzing Git changes. \
                            Provide intelligent, structured analysis in the requested JSON format.";

        log_debug!("🤖 Iris: Preparing intelligence analysis request");
        log_debug!("📊 Analysis prompt: {} chars", prompt.len());

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                log_debug!("🤖 Using OpenAI backend for intelligence analysis with model: {}", model);
                let agent = client
                    .agent(model)
                    .preamble(system_prompt)
                    .temperature(0.3) // Lower temperature for analysis
                    .max_tokens(1500) // More tokens for detailed analysis
                    .build();

                log_debug!("🚀 Sending OpenAI intelligence analysis request...");
                let response = agent
                    .prompt(prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI API error: {}", e))?;

                log_debug!("✅ OpenAI intelligence analysis response received: {} chars", response.len());
                log_debug!("📝 Analysis response preview: {}", response.chars().take(200).collect::<String>());
                Ok(response.trim().to_string())
            }
            AgentBackend::Anthropic { client, model } => {
                log_debug!("🧠 Using Anthropic backend for intelligence analysis with model: {}", model);
                let agent = client
                    .agent(model)
                    .preamble(system_prompt)
                    .temperature(0.3)
                    .max_tokens(1500)
                    .build();

                log_debug!("🚀 Sending Anthropic intelligence analysis request...");
                let response = agent
                    .prompt(prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic API error: {}", e))?;

                log_debug!("✅ Anthropic intelligence analysis response received: {} chars", response.len());
                log_debug!("📝 Analysis response preview: {}", response.chars().take(200).collect::<String>());
                Ok(response.trim().to_string())
            }
        }
    }

    /// Parse JSON response with fallback handling
    async fn parse_json_response<T>(&self, response: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        log_debug!("🔍 Iris: Parsing JSON response - {} chars", response.len());
        log_debug!("📝 Iris: Response preview: {}", response.chars().take(200).collect::<String>());
        
        // First try to parse the response directly
        log_debug!("🎯 Iris: Attempting direct JSON parsing");
        if let Ok(parsed) = serde_json::from_str::<T>(response) {
            log_debug!("✅ Iris: Direct JSON parsing successful");
            return Ok(parsed);
        }
        log_debug!("❌ Iris: Direct JSON parsing failed, trying markdown extraction");

        // Try to extract JSON from markdown code blocks
        if let Some(json_start) = response.find("```json") {
            log_debug!("🔍 Iris: Found markdown JSON block at position {}", json_start);
            if let Some(json_end) = response[json_start..].find("```") {
                let json_content = &response[json_start + 7..json_start + json_end];
                log_debug!("📄 Iris: Extracted JSON content - {} chars", json_content.trim().len());
                if let Ok(parsed) = serde_json::from_str::<T>(json_content.trim()) {
                    log_debug!("✅ Iris: Markdown JSON parsing successful");
                    return Ok(parsed);
                }
                log_debug!("❌ Iris: Markdown JSON parsing failed");
            } else {
                log_debug!("❌ Iris: Found ```json but no closing ```");
            }
        } else {
            log_debug!("🔍 Iris: No markdown JSON blocks found");
        }

        // Try to find any JSON object in the response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let potential_json = &response[start..=end];
                log_debug!("🔍 Iris: Found potential JSON object from {} to {} - {} chars", 
                    start, end, potential_json.len());
                log_debug!("📄 Iris: Potential JSON preview: {}", 
                    potential_json.chars().take(100).collect::<String>());
                if let Ok(parsed) = serde_json::from_str::<T>(potential_json) {
                    log_debug!("✅ Iris: Extracted JSON parsing successful");
                    return Ok(parsed);
                }
                log_debug!("❌ Iris: Extracted JSON parsing failed");
            } else {
                log_debug!("❌ Iris: Found opening {{ but no closing }}");
            }
        } else {
            log_debug!("❌ Iris: No JSON objects found in response");
        }

        log_debug!("🚨 Iris: All JSON parsing attempts failed");
        Err(anyhow::anyhow!(
            "Failed to parse JSON response. Raw response: {}",
            response
        ))
    }
}

#[async_trait]
impl super::core::IrisAgent for IrisAgent {
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
            log_debug!("❌ Iris: Agent not initialized, cannot execute task: {}", task);
            return Err(anyhow::anyhow!("Iris agent not initialized"));
        }

        log_debug!("🎯 Iris: Starting task execution: {}", task);
        log_debug!("📋 Iris: Task parameters: {} keys", params.len());
        
        let start_time = std::time::Instant::now();
        
        let result = match task {
            "generate_commit_message" | "commit_message_generation" => {
                log_debug!("📝 Iris: Executing commit message generation");
                self.generate_commit_message(context, params).await
            }
            "generate_code_review" | "code_review" | "review_code" => {
                log_debug!("🔍 Iris: Executing code review generation");
                self.generate_code_review(context, params).await
            }
            "generate_pull_request" | "pull_request_description" => {
                log_debug!("📋 Iris: Executing pull request description generation");
                self.generate_pull_request(context, params).await
            }
            _ => {
                log_debug!("❌ Iris: Unknown task requested: {}", task);
                Err(anyhow::anyhow!("Unknown task for Iris: {}", task))
            }
        };
        
        let duration = start_time.elapsed();
        
        match &result {
            Ok(task_result) => {
                log_debug!("✅ Iris: Task '{}' completed successfully in {:.2}s (confidence: {:.2})", 
                    task, duration.as_secs_f64(), task_result.confidence);
            }
            Err(e) => {
                log_debug!("❌ Iris: Task '{}' failed after {:.2}s: {}", 
                    task, duration.as_secs_f64(), e);
            }
        }
        
        result
    }

    fn can_handle_task(&self, task: &str) -> bool {
        matches!(
            task,
            "generate_commit_message"
                | "commit_message_generation"
                | "generate_code_review"
                | "code_review"
                | "review_code"
                | "generate_pull_request"
                | "pull_request_description"
                | "changelog_generation"
                | "file_analysis"
                | "relevance_scoring"
        )
    }

    fn task_priority(&self, task: &str) -> u8 {
        match task {
            "generate_commit_message" | "commit_message_generation" => 10,
            "generate_code_review" | "code_review" | "review_code" => 10,
            "generate_pull_request" | "pull_request_description" => 10,
            "changelog_generation" => 9,
            "file_analysis" | "relevance_scoring" => 8,
            _ => 0,
        }
    }

    async fn initialize(&mut self, _context: &AgentContext) -> Result<()> {
        self.initialized = true;
        log_debug!("🤖 Iris: Agent initialized successfully");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        log_debug!("🤖 Iris: Agent cleanup completed");
        Ok(())
    }
}