//! Parallel Analysis Tool
//!
//! Enables Iris to spawn multiple independent subagents that analyze different
//! portions of a codebase concurrently. This prevents context overflow when
//! dealing with large changesets by distributing work across separate context windows.

use anyhow::Result;
use rig::{
    client::CompletionClient,
    completion::{Prompt, ToolDefinition},
    providers::{anthropic, openai},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{CodeSearch, FileRead, GitChangedFiles, GitDiff, GitLog, GitStatus, ProjectDocs};
use crate::agents::debug_tool::DebugTool;

/// Arguments for parallel analysis
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ParallelAnalyzeArgs {
    /// List of analysis tasks to run in parallel.
    /// Each task should be a focused prompt describing what to analyze.
    /// Example: `["Analyze security changes in auth/", "Review performance in db/"]`
    pub tasks: Vec<String>,
}

/// Result from a single subagent analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubagentResult {
    /// The original task prompt
    pub task: String,
    /// The analysis result
    pub result: String,
    /// Whether the analysis succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Aggregated results from all parallel analyses
#[derive(Debug, Serialize, Deserialize)]
pub struct ParallelAnalyzeResult {
    /// Results from each subagent
    pub results: Vec<SubagentResult>,
    /// Number of successful analyses
    pub successful: usize,
    /// Number of failed analyses
    pub failed: usize,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Provider-specific subagent runner
#[derive(Clone)]
enum SubagentRunner {
    OpenAI {
        client: openai::Client,
        model: String,
    },
    Anthropic {
        client: anthropic::Client,
        model: String,
    },
}

impl SubagentRunner {
    fn new(provider: &str, model: &str) -> Result<Self> {
        match provider {
            "openai" => {
                let client = openai::Client::from_env();
                Ok(Self::OpenAI {
                    client,
                    model: model.to_string(),
                })
            }
            "anthropic" => {
                let client = anthropic::Client::from_env();
                Ok(Self::Anthropic {
                    client,
                    model: model.to_string(),
                })
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported provider for parallel analysis: {}",
                provider
            )),
        }
    }

    async fn run_task(&self, task: &str) -> SubagentResult {
        let preamble = "You are a specialized analysis sub-agent. Complete the assigned \
            task thoroughly and return a focused summary.\n\n\
            Guidelines:\n\
            - Use the available tools to gather necessary information\n\
            - Focus only on what's asked\n\
            - Return a clear, structured summary\n\
            - Be concise but comprehensive";

        let result = match self {
            Self::OpenAI { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .max_tokens(4096)
                    .tool(DebugTool::new(GitStatus))
                    .tool(DebugTool::new(GitDiff))
                    .tool(DebugTool::new(GitLog))
                    .tool(DebugTool::new(GitChangedFiles))
                    .tool(DebugTool::new(FileRead))
                    .tool(DebugTool::new(CodeSearch))
                    .tool(DebugTool::new(ProjectDocs))
                    .build();

                agent.prompt(task).await
            }
            Self::Anthropic { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(preamble)
                    .max_tokens(4096)
                    .tool(DebugTool::new(GitStatus))
                    .tool(DebugTool::new(GitDiff))
                    .tool(DebugTool::new(GitLog))
                    .tool(DebugTool::new(GitChangedFiles))
                    .tool(DebugTool::new(FileRead))
                    .tool(DebugTool::new(CodeSearch))
                    .tool(DebugTool::new(ProjectDocs))
                    .build();

                agent.prompt(task).await
            }
        };

        match result {
            Ok(response) => SubagentResult {
                task: task.to_string(),
                result: response,
                success: true,
                error: None,
            },
            Err(e) => SubagentResult {
                task: task.to_string(),
                result: String::new(),
                success: false,
                error: Some(e.to_string()),
            },
        }
    }
}

/// Parallel analysis tool
/// Spawns multiple subagents to analyze different aspects concurrently
pub struct ParallelAnalyze {
    runner: SubagentRunner,
}

impl ParallelAnalyze {
    pub fn new(provider: &str, model: &str) -> Self {
        // Default to openai if creation fails
        let runner = SubagentRunner::new(provider, model).unwrap_or_else(|_| {
            tracing::warn!(
                "Failed to create {} runner, falling back to openai",
                provider
            );
            SubagentRunner::new("openai", "gpt-4o").expect("OpenAI fallback should work")
        });

        Self { runner }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Parallel analysis error: {0}")]
pub struct ParallelAnalyzeError(String);

impl Tool for ParallelAnalyze {
    const NAME: &'static str = "parallel_analyze";
    type Error = ParallelAnalyzeError;
    type Args = ParallelAnalyzeArgs;
    type Output = ParallelAnalyzeResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Run multiple analysis tasks in parallel using independent subagents. \
                         Each subagent has its own context window, preventing overflow when \
                         analyzing large changesets. Use this when you have multiple independent \
                         analysis tasks that can run concurrently.\n\n\
                         Best for:\n\
                         - Analyzing different directories/modules separately\n\
                         - Processing many commits in batches\n\
                         - Running different types of analysis (security, performance, style) in parallel\n\n\
                         Each task should be a focused prompt. Results are aggregated and returned."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "tasks": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of analysis task prompts to run in parallel. Each task runs in its own subagent with independent context.",
                        "minItems": 1,
                        "maxItems": 10
                    }
                },
                "required": ["tasks"]
            }),
        }
    }

    #[allow(clippy::cognitive_complexity)]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        use std::time::Instant;

        let start = Instant::now();
        let tasks = args.tasks;
        let num_tasks = tasks.len();

        tracing::info!(
            "🔀 ParallelAnalyze: Spawning {} subagents for parallel analysis",
            num_tasks
        );

        // Collect results using Arc<Mutex> for thread-safe access
        let results: Arc<Mutex<Vec<SubagentResult>>> = Arc::new(Mutex::new(Vec::new()));

        // Spawn all tasks as parallel tokio tasks
        let mut handles = Vec::new();
        for task in tasks {
            let runner = self.runner.clone();
            let results = Arc::clone(&results);

            let handle = tokio::spawn(async move {
                tracing::debug!("🔹 Subagent starting: {}", &task[..task.len().min(50)]);

                let result = runner.run_task(&task).await;

                tracing::debug!(
                    "🔹 Subagent completed: {} (success: {})",
                    &task[..task.len().min(50)],
                    result.success
                );

                let mut guard = results.lock().await;
                guard.push(result);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                tracing::warn!("Subagent task panicked: {}", e);
            }
        }

        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        let execution_time_ms = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let final_results = Arc::try_unwrap(results)
            .map_err(|_| ParallelAnalyzeError("Failed to unwrap results".to_string()))?
            .into_inner();

        let successful = final_results.iter().filter(|r| r.success).count();
        let failed = final_results.iter().filter(|r| !r.success).count();

        tracing::info!(
            "🔀 ParallelAnalyze complete: {}/{} successful in {}ms",
            successful,
            num_tasks,
            execution_time_ms
        );

        Ok(ParallelAnalyzeResult {
            results: final_results,
            successful,
            failed,
            execution_time_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_analyze_args_schema() {
        let schema = schemars::schema_for!(ParallelAnalyzeArgs);
        let json = serde_json::to_string_pretty(&schema).expect("schema should serialize");
        assert!(json.contains("tasks"));
    }
}
