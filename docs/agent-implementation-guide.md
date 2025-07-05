# Git-Iris Agent Implementation Guide

## Technical Implementation Details

This guide provides concrete implementation details for building the Git-Iris agent framework using Rig.

## Project Structure

```
src/
├── agents/
│   ├── mod.rs              # Agent module exports
│   ├── base.rs             # Base agent traits and structures
│   ├── commit.rs           # Commit message generation agent
│   ├── review.rs           # Code review agent
│   ├── release_notes.rs    # Release notes agent
│   └── changelog.rs        # Changelog generation agent
├── tools/
│   ├── mod.rs              # Tool registry and exports
│   ├── git.rs              # Git operations tool
│   ├── file.rs             # File analysis tool
│   ├── diff.rs             # Diff processing tool
│   ├── search.rs           # Code search tool
│   └── memory.rs           # Agent memory/state tool
├── agent_config.rs         # Agent-specific configuration
└── mcp/
    └── agent_tools.rs      # MCP wrappers for agent tools
```

## Core Implementation

### 1. Base Agent Infrastructure

```rust
// src/agents/base.rs
use rig::completion::{CompletionClient, Prompt};
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub repo: Arc<GitRepository>,
    pub config: Arc<Config>,
    pub llm_client: Arc<dyn CompletionClient>,
}

#[async_trait]
pub trait GitIrisAgent: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn system_prompt(&self) -> String;
    fn tools(&self) -> Vec<Box<dyn Tool>>;
    
    async fn execute(&self, prompt: &str) -> Result<String> {
        let agent = rig::Agent::builder()
            .preamble(&self.system_prompt())
            .tools(self.tools())
            .build();
        
        agent.prompt(prompt).await.map_err(Into::into)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub result: String,
    pub tools_used: Vec<ToolCall>,
    pub iterations: usize,
    pub tokens_used: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: serde_json::Value,
    pub result: serde_json::Value,
}
```

### 2. Git Operations Tool Implementation

```rust
// src/tools/git.rs
use rig::tool::{Tool, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct GitOperationsTool {
    repo: Arc<GitRepository>,
}

#[derive(Deserialize)]
#[serde(tag = "operation")]
enum GitOperation {
    ListCommits { 
        range: Option<String>, 
        limit: Option<usize> 
    },
    GetCommit { 
        sha: String 
    },
    GetDiff { 
        base: Option<String>, 
        head: Option<String>,
        paths: Option<Vec<String>>
    },
    GetStatus,
    GetBranches,
    GetTags { 
        pattern: Option<String> 
    },
}

impl Tool for GitOperationsTool {
    const NAME: &'static str = "git";
    type Error = anyhow::Error;
    type Args = GitOperation;
    type Output = serde_json::Value;
    
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Perform git operations on the repository".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["ListCommits", "GetCommit", "GetDiff", "GetStatus", "GetBranches", "GetTags"]
                    },
                    "range": { "type": "string" },
                    "limit": { "type": "integer" },
                    "sha": { "type": "string" },
                    "base": { "type": "string" },
                    "head": { "type": "string" },
                    "paths": { "type": "array", "items": { "type": "string" } },
                    "pattern": { "type": "string" }
                },
                "required": ["operation"]
            }),
        }
    }
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args {
            GitOperation::ListCommits { range, limit } => {
                let commits = self.repo
                    .list_commits(range.as_deref(), limit.unwrap_or(20))
                    .await?;
                Ok(serde_json::to_value(commits)?)
            }
            GitOperation::GetCommit { sha } => {
                let commit = self.repo.get_commit(&sha).await?;
                Ok(serde_json::to_value(commit)?)
            }
            GitOperation::GetDiff { base, head, paths } => {
                let diff = self.repo
                    .get_diff(base.as_deref(), head.as_deref(), paths)
                    .await?;
                Ok(serde_json::to_value(diff)?)
            }
            GitOperation::GetStatus => {
                let status = self.repo.get_status().await?;
                Ok(serde_json::to_value(status)?)
            }
            GitOperation::GetBranches => {
                let branches = self.repo.get_branches().await?;
                Ok(serde_json::to_value(branches)?)
            }
            GitOperation::GetTags { pattern } => {
                let tags = self.repo.get_tags(pattern.as_deref()).await?;
                Ok(serde_json::to_value(tags)?)
            }
        }
    }
}
```

### 3. Commit Message Agent Implementation

```rust
// src/agents/commit.rs
use crate::agents::base::{AgentContext, GitIrisAgent};
use crate::tools::{GitOperationsTool, FileAnalysisTool, MemoryTool};

pub struct CommitAgent {
    context: AgentContext,
    options: CommitOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitOptions {
    pub gitmoji: bool,
    pub conventional: bool,
    pub max_length: Option<usize>,
    pub instructions: Option<String>,
    pub preset: Option<String>,
}

impl CommitAgent {
    pub fn new(context: AgentContext, options: CommitOptions) -> Self {
        Self { context, options }
    }
    
    pub async fn generate(&self) -> Result<CommitMessage> {
        let prompt = self.build_prompt()?;
        let response = self.execute(&prompt).await?;
        
        // Parse the response into a structured commit message
        let commit_message: CommitMessage = serde_json::from_str(&response)?;
        
        // Validate the message
        self.validate_message(&commit_message)?;
        
        Ok(commit_message)
    }
    
    fn build_prompt(&self) -> Result<String> {
        let mut prompt = String::from("Generate a commit message for the current staged changes.\n\n");
        
        if self.options.gitmoji {
            prompt.push_str("Include an appropriate gitmoji at the beginning.\n");
        }
        
        if self.options.conventional {
            prompt.push_str("Follow the conventional commits specification.\n");
        }
        
        if let Some(max_length) = self.options.max_length {
            prompt.push_str(&format!("Keep the message under {} characters.\n", max_length));
        }
        
        if let Some(instructions) = &self.options.instructions {
            prompt.push_str(&format!("\nAdditional instructions: {}\n", instructions));
        }
        
        prompt.push_str("\nAnalyze the changes thoroughly and generate an appropriate commit message.");
        
        Ok(prompt)
    }
}

#[async_trait]
impl GitIrisAgent for CommitAgent {
    fn name(&self) -> &str {
        "CommitAgent"
    }
    
    fn description(&self) -> &str {
        "Generates commit messages by analyzing staged changes"
    }
    
    fn system_prompt(&self) -> String {
        format!(r#"You are an expert Git commit message generator. Your task is to analyze code changes and create clear, concise commit messages.

Instructions:
1. Use the git tool to get the current status and staged changes
2. Analyze the diff to understand what changed
3. Use the file tool to read relevant files for context
4. Use the memory tool to track important findings
5. Generate a commit message that accurately describes the changes

Guidelines:
- Be concise but descriptive
- Focus on the "why" not just the "what"
- Group related changes
{}
{}

Output format: Return a JSON object with the following structure:
{{
    "title": "The commit title (50 chars max)",
    "body": "Optional detailed description",
    "type": "feat|fix|docs|style|refactor|test|chore",
    "scope": "optional scope",
    "breaking": false,
    "issues": ["#123", "#456"]
}}"#,
            if self.options.gitmoji { "- Include appropriate gitmoji" } else { "" },
            if self.options.conventional { "- Follow conventional commits format" } else { "" }
        )
    }
    
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GitOperationsTool { repo: self.context.repo.clone() }),
            Box::new(FileAnalysisTool { repo: self.context.repo.clone() }),
            Box::new(MemoryTool::new()),
        ]
    }
}
```

### 4. Advanced Diff Processing Tool

```rust
// src/tools/diff.rs
use rig::tool::{Tool, ToolDefinition};

#[derive(Clone)]
pub struct DiffTool {
    repo: Arc<GitRepository>,
    chunk_size: usize,
}

#[derive(Deserialize)]
#[serde(tag = "operation")]
enum DiffOperation {
    GetChunk {
        offset: usize,
        limit: Option<usize>,
    },
    GetFileChanges {
        path: String,
        context_lines: Option<usize>,
    },
    GetStats,
    SearchInDiff {
        pattern: String,
    },
}

impl Tool for DiffTool {
    const NAME: &'static str = "diff";
    type Error = anyhow::Error;
    type Args = DiffOperation;
    type Output = serde_json::Value;
    
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Analyze diffs with chunking support for large changes".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["GetChunk", "GetFileChanges", "GetStats", "SearchInDiff"]
                    },
                    "offset": { "type": "integer" },
                    "limit": { "type": "integer" },
                    "path": { "type": "string" },
                    "context_lines": { "type": "integer" },
                    "pattern": { "type": "string" }
                },
                "required": ["operation"]
            }),
        }
    }
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args {
            DiffOperation::GetChunk { offset, limit } => {
                let diff = self.repo.get_staged_diff().await?;
                let chunk = self.extract_chunk(&diff, offset, limit.unwrap_or(self.chunk_size))?;
                Ok(json!({
                    "chunk": chunk,
                    "total_lines": diff.lines().count(),
                    "has_more": offset + limit.unwrap_or(self.chunk_size) < diff.lines().count()
                }))
            }
            DiffOperation::GetFileChanges { path, context_lines } => {
                let changes = self.repo
                    .get_file_diff(&path, context_lines.unwrap_or(3))
                    .await?;
                Ok(serde_json::to_value(changes)?)
            }
            DiffOperation::GetStats => {
                let stats = self.repo.get_diff_stats().await?;
                Ok(serde_json::to_value(stats)?)
            }
            DiffOperation::SearchInDiff { pattern } => {
                let matches = self.search_in_diff(&pattern).await?;
                Ok(serde_json::to_value(matches)?)
            }
        }
    }
}
```

### 5. Release Notes Agent with Advanced Features

```rust
// src/agents/release_notes.rs
use crate::agents::base::{AgentContext, GitIrisAgent};

pub struct ReleaseNotesAgent {
    context: AgentContext,
    from_ref: String,
    to_ref: String,
    options: ReleaseNotesOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseNotesOptions {
    pub group_by_type: bool,
    pub include_stats: bool,
    pub include_contributors: bool,
    pub highlight_breaking_changes: bool,
    pub template: Option<String>,
}

impl ReleaseNotesAgent {
    pub async fn generate(&self) -> Result<ReleaseNotes> {
        // Build a comprehensive prompt for the agent
        let prompt = self.build_analysis_prompt()?;
        
        // Execute the agent
        let response = self.execute(&prompt).await?;
        
        // Parse and structure the response
        let release_notes: ReleaseNotes = serde_json::from_str(&response)?;
        
        Ok(release_notes)
    }
    
    fn build_analysis_prompt(&self) -> Result<String> {
        Ok(format!(
            r#"Generate comprehensive release notes for commits between {} and {}.

Instructions:
1. List all commits in the range
2. Analyze each commit to understand its impact
3. Group changes by type (features, fixes, etc.)
4. Identify breaking changes
5. Extract key improvements and highlights
6. Generate a well-structured release notes document

Options:
- Group by type: {}
- Include statistics: {}
- Include contributors: {}
- Highlight breaking changes: {}

Use the available tools to explore the repository and gather all necessary information."#,
            self.from_ref,
            self.to_ref,
            self.options.group_by_type,
            self.options.include_stats,
            self.options.include_contributors,
            self.options.highlight_breaking_changes
        ))
    }
}

#[async_trait]
impl GitIrisAgent for ReleaseNotesAgent {
    fn name(&self) -> &str {
        "ReleaseNotesAgent"
    }
    
    fn description(&self) -> &str {
        "Generates comprehensive release notes by analyzing commits and changes"
    }
    
    fn system_prompt(&self) -> String {
        r#"You are an expert at creating comprehensive release notes. Your task is to analyze all commits in a given range and produce well-structured, informative release notes.

Process:
1. Use the git tool to list all commits in the specified range
2. For each significant commit, analyze its changes using the diff tool
3. Use the file tool to understand the context of changes
4. Use the memory tool to categorize and organize findings
5. Search for patterns indicating breaking changes
6. Generate structured release notes

Output format: Return a JSON object with:
{
    "version": "version string",
    "date": "release date",
    "summary": "brief summary",
    "breaking_changes": ["list of breaking changes"],
    "features": ["list of new features"],
    "fixes": ["list of bug fixes"],
    "improvements": ["list of improvements"],
    "contributors": ["list of contributors"],
    "statistics": {
        "commits": 0,
        "files_changed": 0,
        "insertions": 0,
        "deletions": 0
    }
}"#.to_string()
    }
    
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GitOperationsTool { repo: self.context.repo.clone() }),
            Box::new(DiffTool { 
                repo: self.context.repo.clone(),
                chunk_size: 2000,
            }),
            Box::new(FileAnalysisTool { repo: self.context.repo.clone() }),
            Box::new(SearchTool { repo: self.context.repo.clone() }),
            Box::new(MemoryTool::new()),
        ]
    }
}
```

### 6. MCP Integration

```rust
// src/mcp/agent_tools.rs
use rmcp::{Tool as McpTool, ToolDefinition as McpToolDefinition};

pub struct AgentBasedCommitTool {
    context: Arc<AgentContext>,
}

#[async_trait]
impl McpTool for AgentBasedCommitTool {
    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "git_iris_agent_commit".to_string(),
            description: Some("Generate commit messages using intelligent agent analysis".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "repository": {
                        "type": "string",
                        "description": "Repository path or URL"
                    },
                    "use_agent": {
                        "type": "boolean",
                        "description": "Use agent-based analysis (default: true)"
                    },
                    "max_iterations": {
                        "type": "integer",
                        "description": "Maximum agent iterations"
                    },
                    "options": {
                        "type": "object",
                        "properties": {
                            "gitmoji": { "type": "boolean" },
                            "conventional": { "type": "boolean" },
                            "instructions": { "type": "string" }
                        }
                    }
                },
                "required": ["repository"]
            }),
        }
    }
    
    async fn call(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let repo_path = input["repository"].as_str().unwrap();
        let use_agent = input["use_agent"].as_bool().unwrap_or(true);
        
        if use_agent {
            // Use agent-based approach
            let agent = CommitAgent::new(
                self.context.clone(),
                serde_json::from_value(input["options"].clone()).unwrap_or_default(),
            );
            
            let result = agent.generate().await?;
            
            Ok(json!({
                "commit_message": result,
                "agent_used": true,
                "method": "agent-based"
            }))
        } else {
            // Fall back to traditional approach
            let service = IrisCommitService::new(self.context.config.clone());
            let result = service.generate_commit_message(repo_path).await?;
            
            Ok(json!({
                "commit_message": result,
                "agent_used": false,
                "method": "traditional"
            }))
        }
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_git_tool_list_commits() {
        let repo = create_test_repo().await;
        let tool = GitOperationsTool { repo };
        
        let args = GitOperation::ListCommits {
            range: Some("HEAD~5..HEAD".to_string()),
            limit: Some(5),
        };
        
        let result = tool.call(args).await.unwrap();
        assert!(result.is_array());
    }
    
    #[tokio::test]
    async fn test_commit_agent_generation() {
        let context = create_test_context().await;
        let agent = CommitAgent::new(context, Default::default());
        
        let message = agent.generate().await.unwrap();
        assert!(!message.title.is_empty());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_large_diff_handling() {
    // Create a test repo with large changes
    let repo = create_large_test_repo().await;
    let context = AgentContext {
        repo: Arc::new(repo),
        config: Arc::new(Config::default()),
        llm_client: Arc::new(MockLlmClient::new()),
    };
    
    let agent = ReleaseNotesAgent::new(
        context,
        "v1.0.0".to_string(),
        "v2.0.0".to_string(),
        Default::default(),
    );
    
    let notes = agent.generate().await.unwrap();
    assert!(!notes.features.is_empty());
}
```

## Performance Considerations

1. **Token Optimization**: Implement smart chunking to stay within token limits
2. **Caching**: Cache frequently accessed data (commits, file contents)
3. **Parallel Processing**: Use tokio for concurrent tool execution
4. **Early Termination**: Stop agent iterations when sufficient information is gathered

## Deployment

1. **Feature Flag**: Add `--use-agent` flag to CLI commands
2. **Gradual Rollout**: Start with opt-in, then make default
3. **Metrics**: Track agent performance and token usage
4. **Fallback**: Always maintain traditional mode as fallback