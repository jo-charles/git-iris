use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::agents::core::AgentContext;
use crate::log_debug;

pub mod knowledge;

/// Core trait for agent tools
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Get the tool's unique identifier
    fn id(&self) -> &str;

    /// Get the tool's display name
    fn name(&self) -> &str;

    /// Get the tool's description
    fn description(&self) -> &str;

    /// Get the capabilities this tool provides
    fn capabilities(&self) -> Vec<String>;

    /// Execute the tool with given parameters
    async fn execute(
        &self,
        _context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value>;

    /// Get the tool's parameter schema
    fn parameter_schema(&self) -> serde_json::Value;

    /// Convert to Rig Tool trait object (placeholder for future integration)
    fn as_rig_tool_placeholder(&self) -> String;
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
    capability_index: HashMap<String, Vec<String>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Arc<dyn AgentTool>) {
        let id = tool.id().to_string();

        // Add to capability index
        for capability in tool.capabilities() {
            self.capability_index
                .entry(capability)
                .or_default()
                .push(id.clone());
        }

        // Add to tools registry
        self.tools.insert(id, tool);
    }

    pub fn get_tool(&self, id: &str) -> Option<Arc<dyn AgentTool>> {
        self.tools.get(id).cloned()
    }

    pub async fn get_tools_for_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<Arc<dyn AgentTool>>> {
        let empty_vec = Vec::new();
        let tool_ids = self.capability_index.get(capability).unwrap_or(&empty_vec);

        let mut tools = Vec::new();
        for id in tool_ids {
            if let Some(tool) = self.tools.get(id) {
                tools.push(tool.clone());
            }
        }

        Ok(tools)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn list_capabilities(&self) -> Vec<String> {
        self.capability_index.keys().cloned().collect()
    }
}

/// Git repository operations tool
pub struct GitTool {
    id: String,
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitTool {
    pub fn new() -> Self {
        Self {
            id: "git_operations".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct GitToolArgs {
    pub operation: String,
    pub path: Option<String>,
    pub commit_range: Option<String>,
}

#[async_trait]
impl AgentTool for GitTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Git Operations"
    }

    fn description(&self) -> &'static str {
        "Perform git repository operations like getting diff, status, log, etc."
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "git".to_string(),
            "commit".to_string(),
            "review".to_string(),
        ]
    }

    async fn execute(
        &self,
        context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        log_debug!("🔧 GitTool executing with params: {:?}", params);

        let args: GitToolArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        log_debug!("⚙️  GitTool operation: {}", args.operation);
        let repo = &context.git_repo;

        match args.operation.as_str() {
            "diff" => {
                log_debug!("📄 GitTool: Getting staged files with diffs");
                // Get the actual git context which includes diffs
                let git_context = repo.get_git_info(&context.config).await?;
                let combined_diff = git_context
                    .staged_files
                    .iter()
                    .map(|f| format!("{}:\n{}", f.path, f.diff))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                log_debug!(
                    "✅ GitTool: Diff retrieved, {} staged files, {} total chars",
                    git_context.staged_files.len(),
                    combined_diff.len()
                );

                Ok(serde_json::json!({
                    "operation": "diff",
                    "content": combined_diff,
                }))
            }
            "status" => {
                log_debug!("📊 GitTool: Getting repository status");
                let files = repo.get_unstaged_files()?;
                log_debug!(
                    "✅ GitTool: Status retrieved, {} unstaged files",
                    files.len()
                );

                Ok(serde_json::json!({
                    "operation": "status",
                    "content": files,
                }))
            }
            "log" => {
                log_debug!("📜 GitTool: Getting recent commit history (10 commits)");
                let commits = repo.get_recent_commits(10)?;
                log_debug!(
                    "✅ GitTool: Commit history retrieved, {} commits found",
                    commits.len()
                );

                Ok(serde_json::json!({
                    "operation": "log",
                    "content": commits,
                }))
            }
            "files" => {
                log_debug!("📂 GitTool: Getting list of changed files");
                let git_context = repo.get_git_info(&context.config).await?;
                let files: Vec<String> = git_context
                    .staged_files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect();

                log_debug!(
                    "✅ GitTool: File list retrieved, {} files: {:?}",
                    files.len(),
                    files
                );

                Ok(serde_json::json!({
                    "operation": "files",
                    "content": files,
                }))
            }
            _ => {
                log_debug!("❌ GitTool: Unknown operation: {}", args.operation);
                Err(anyhow::anyhow!("Unknown git operation: {}", args.operation))
            }
        }
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["diff", "status", "log", "files"],
                    "description": "The git operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "Optional path to limit operation scope"
                },
                "commit_range": {
                    "type": "string",
                    "description": "Optional commit range for diff operation (e.g., 'HEAD~1..HEAD')"
                }
            },
            "required": ["operation"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("GitTool: {}", self.name())
    }
}

/// File reading and analysis tool
pub struct FileAnalyzerTool {
    id: String,
}

impl Default for FileAnalyzerTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FileAnalyzerTool {
    pub fn new() -> Self {
        Self {
            id: "file_analyzer".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct FileAnalyzerArgs {
    pub path: String,
    pub analysis_type: Option<String>,
    pub include_content: Option<bool>,
}

#[async_trait]
impl AgentTool for FileAnalyzerTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "File Analyzer"
    }

    fn description(&self) -> &'static str {
        "Analyze files for content, structure, and language-specific information"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "file_analysis".to_string(),
            "code_review".to_string(),
            "commit".to_string(),
        ]
    }

    async fn execute(
        &self,
        _context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        log_debug!("🔍 FileAnalyzerTool executing with params: {:?}", params);

        let args: FileAnalyzerArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        log_debug!(
            "📄 FileAnalyzerTool: Analyzing file: {} (type: {:?}, include_content: {})",
            args.path,
            args.analysis_type,
            args.include_content.unwrap_or(false)
        );

        let analyzer = crate::file_analyzers::get_analyzer(&args.path);
        let file_type = analyzer.get_file_type();
        let include_content = args.include_content.unwrap_or(false);

        log_debug!("🔬 FileAnalyzerTool: File type detected: {}", file_type);

        let mut result = serde_json::json!({
            "path": args.path,
            "file_type": file_type,
        });

        if include_content {
            log_debug!("📖 FileAnalyzerTool: Reading file content for analysis");
            if let Ok(content) = tokio::fs::read_to_string(&args.path).await {
                let line_count = content.lines().count();
                result["content"] = serde_json::Value::String(content.clone());
                result["lines"] = serde_json::Value::Number(serde_json::Number::from(line_count));

                log_debug!(
                    "✅ FileAnalyzerTool: Content read - {} lines, {} chars",
                    line_count,
                    content.len()
                );
            } else {
                log_debug!(
                    "⚠️  FileAnalyzerTool: Failed to read file content for: {}",
                    args.path
                );
            }
        }

        log_debug!("✅ FileAnalyzerTool: Analysis complete for {}", args.path);
        Ok(result)
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to analyze"
                },
                "analysis_type": {
                    "type": "string",
                    "enum": ["basic", "detailed", "security"],
                    "description": "Type of analysis to perform"
                },
                "include_content": {
                    "type": "boolean",
                    "description": "Whether to include file content in the response"
                }
            },
            "required": ["path"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("FileAnalyzerTool: {}", self.name())
    }
}

/// Code search tool for finding related files and functions
pub struct CodeSearchTool {
    id: String,
}

impl Default for CodeSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSearchTool {
    pub fn new() -> Self {
        Self {
            id: "code_search".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CodeSearchArgs {
    pub query: String,
    pub search_type: String, // "function", "class", "variable", "text"
    pub file_pattern: Option<String>,
    pub max_results: Option<usize>,
}

#[async_trait]
impl AgentTool for CodeSearchTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Code Search"
    }

    fn description(&self) -> &'static str {
        "Search for code patterns, functions, classes, and related files in the repository"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "search".to_string(),
            "code_analysis".to_string(),
            "review".to_string(),
        ]
    }

    async fn execute(
        &self,
        _context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let args: CodeSearchArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        // This would implement actual code search functionality using _context.git_repo
        // For now, return a placeholder structure
        Ok(serde_json::json!({
            "query": args.query,
            "search_type": args.search_type,
            "results": [],
            "total_found": 0,
        }))
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "search_type": {
                    "type": "string",
                    "enum": ["function", "class", "variable", "text", "pattern"],
                    "description": "Type of search to perform"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "Optional file pattern to limit search scope"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return"
                }
            },
            "required": ["query", "search_type"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("CodeSearchTool: {}", self.name())
    }
}

/// Creates the default tool registry with all built-in tools
pub fn create_default_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry.register_tool(Arc::new(GitTool::new()));
    registry.register_tool(Arc::new(FileAnalyzerTool::new()));
    registry.register_tool(Arc::new(CodeSearchTool::new()));
    registry.register_tool(Arc::new(crate::agents::tools::knowledge::WorkspaceTool::new()));

    registry
}
