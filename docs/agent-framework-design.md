# Git-Iris Agent Framework Design with Rig

## Overview

This document outlines the design for transforming Git-Iris from a single LLM call architecture to an agent-based system using Rig. The agent framework will enable Git-Iris to handle large codebases and complex commits by iteratively exploring and analyzing code changes. The framework will also integrate seamlessly with the existing MCP server implementation.

## Motivation

Current limitations:
- Single LLM calls hit token limits with large diffs
- Cannot explore related files or broader context
- Limited ability to deep-dive into specific changes
- Difficulty generating comprehensive release notes for large releases

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                        Git-Iris CLI                         │
├─────────────────────────────────────────────────────────────┤
│                      Agent Orchestrator                      │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Commit Agent│  │ Review Agent │  │Release Notes Agent│  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Rig Tool Registry                         │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│  │ Git  │ │ File │ │ Diff │ │Search│ │ LLM  │ │Memory│   │
│  │Tools │ │Tools │ │Tools │ │Tools │ │Tools │ │Tools │   │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
├─────────────────────────────────────────────────────────────┤
│                     MCP Server Layer                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Exposes agent capabilities through MCP protocol     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Rig Integration

### Dependencies
```toml
[dependencies]
rig-core = "0.2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Agent Implementation with Rig

#### Base Agent Structure
```rust
use rig::{Agent, Tool, ToolDefinition};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GitIrisAgent {
    pub name: String,
    pub llm_client: Box<dyn rig::completion::CompletionClient>,
    pub tools: Vec<Box<dyn Tool>>,
}

impl GitIrisAgent {
    pub async fn execute<T: Serialize>(&self, task: T) -> Result<String> {
        let agent = rig::Agent::builder()
            .preamble(&self.get_system_prompt())
            .tool(self.tools.clone())
            .build();
        
        let response = agent
            .prompt(&serde_json::to_string(&task)?)
            .await?;
        
        Ok(response)
    }
}
```

## Tool Design Using Rig's Tool Trait

### Git Operations Tool
```rust
use rig::{Tool, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct GitCommitListArgs {
    range: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct CommitInfo {
    sha: String,
    message: String,
    author: String,
    date: String,
    files_changed: usize,
}

struct GitTool {
    repo: Arc<GitRepository>,
}

impl Tool for GitTool {
    const NAME: &'static str = "git_operations";
    type Error = anyhow::Error;
    type Args = serde_json::Value;
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
                        "enum": ["list_commits", "get_commit", "get_diff", "get_status"],
                        "description": "The git operation to perform"
                    },
                    "args": {
                        "type": "object",
                        "description": "Operation-specific arguments"
                    }
                },
                "required": ["operation"]
            }),
        }
    }
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let operation = args["operation"].as_str().unwrap();
        
        match operation {
            "list_commits" => {
                let range = args["args"]["range"].as_str();
                let limit = args["args"]["limit"].as_u64().unwrap_or(10) as usize;
                
                let commits = self.repo.list_commits(range, limit).await?;
                Ok(serde_json::to_value(commits)?)
            }
            "get_diff" => {
                let path = args["args"]["path"].as_str();
                let base = args["args"]["base"].as_str();
                
                let diff = self.repo.get_diff(path, base).await?;
                Ok(serde_json::to_value(diff)?)
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", operation))
        }
    }
}
```

### File Analysis Tool
```rust
#[derive(Clone)]
struct FileAnalysisTool {
    repo: Arc<GitRepository>,
}

impl Tool for FileAnalysisTool {
    const NAME: &'static str = "file_analysis";
    type Error = anyhow::Error;
    type Args = serde_json::Value;
    type Output = serde_json::Value;
    
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Analyze files in the repository".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read_file", "search_files", "get_file_history"],
                        "description": "The file operation to perform"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path (for read_file and get_file_history)"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (for search_files)"
                    },
                    "revision": {
                        "type": "string",
                        "description": "Git revision (optional)"
                    }
                },
                "required": ["operation"]
            }),
        }
    }
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let operation = args["operation"].as_str().unwrap();
        
        match operation {
            "read_file" => {
                let path = args["path"].as_str().unwrap();
                let revision = args["revision"].as_str();
                
                let content = self.repo.read_file(path, revision).await?;
                Ok(json!({ "content": content }))
            }
            "search_files" => {
                let pattern = args["pattern"].as_str().unwrap();
                
                let results = self.repo.search_files(pattern).await?;
                Ok(serde_json::to_value(results)?)
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", operation))
        }
    }
}
```

### Memory Tool for Agent State
```rust
#[derive(Clone)]
struct MemoryTool {
    state: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl Tool for MemoryTool {
    const NAME: &'static str = "memory";
    type Error = anyhow::Error;
    type Args = serde_json::Value;
    type Output = serde_json::Value;
    
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Store and retrieve information during analysis".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["store", "retrieve", "list_keys"],
                        "description": "Memory operation"
                    },
                    "key": {
                        "type": "string",
                        "description": "Key for storage/retrieval"
                    },
                    "value": {
                        "type": "any",
                        "description": "Value to store"
                    }
                },
                "required": ["operation"]
            }),
        }
    }
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let operation = args["operation"].as_str().unwrap();
        let mut state = self.state.lock().await;
        
        match operation {
            "store" => {
                let key = args["key"].as_str().unwrap();
                let value = args["value"].clone();
                state.insert(key.to_string(), value);
                Ok(json!({ "status": "stored" }))
            }
            "retrieve" => {
                let key = args["key"].as_str().unwrap();
                Ok(state.get(key).cloned().unwrap_or(json!(null)))
            }
            "list_keys" => {
                let keys: Vec<String> = state.keys().cloned().collect();
                Ok(json!({ "keys": keys }))
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", operation))
        }
    }
}
```

## Agent Implementations

### Commit Message Agent
```rust
pub struct CommitAgent {
    base_agent: GitIrisAgent,
}

impl CommitAgent {
    pub fn new(llm_client: Box<dyn rig::completion::CompletionClient>, repo: Arc<GitRepository>) -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(GitTool { repo: repo.clone() }),
            Box::new(FileAnalysisTool { repo: repo.clone() }),
            Box::new(MemoryTool { state: Arc::new(Mutex::new(HashMap::new())) }),
        ];
        
        Self {
            base_agent: GitIrisAgent {
                name: "CommitAgent".to_string(),
                llm_client,
                tools,
            }
        }
    }
    
    pub async fn generate_commit_message(&self, options: CommitOptions) -> Result<String> {
        let task = json!({
            "task": "generate_commit_message",
            "options": options,
            "instructions": "Analyze the staged changes and generate an appropriate commit message. Use the git_operations tool to get the diff, file_analysis tool to understand context, and memory tool to track your findings."
        });
        
        self.base_agent.execute(task).await
    }
    
    fn get_system_prompt(&self) -> String {
        r#"You are a Git commit message generator. Your task is to analyze code changes and create clear, concise commit messages following best practices.
        
        Use the available tools to:
        1. Get the current git status and staged changes
        2. Analyze the diff to understand what changed
        3. Read relevant files for context if needed
        4. Store important findings in memory
        5. Generate a commit message that accurately describes the changes
        
        Follow conventional commit format when applicable."#.to_string()
    }
}
```

### Release Notes Agent
```rust
pub struct ReleaseNotesAgent {
    base_agent: GitIrisAgent,
}

impl ReleaseNotesAgent {
    pub fn new(llm_client: Box<dyn rig::completion::CompletionClient>, repo: Arc<GitRepository>) -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(GitTool { repo: repo.clone() }),
            Box::new(FileAnalysisTool { repo: repo.clone() }),
            Box::new(MemoryTool { state: Arc::new(Mutex::new(HashMap::new())) }),
        ];
        
        Self {
            base_agent: GitIrisAgent {
                name: "ReleaseNotesAgent".to_string(),
                llm_client,
                tools,
            }
        }
    }
    
    pub async fn generate_release_notes(&self, from_version: &str, to_version: &str) -> Result<String> {
        let task = json!({
            "task": "generate_release_notes",
            "from_version": from_version,
            "to_version": to_version,
            "instructions": "Generate comprehensive release notes by analyzing all commits between versions. Group changes by type (features, fixes, etc.) and highlight breaking changes."
        });
        
        self.base_agent.execute(task).await
    }
}
```

## MCP Server Integration

### Enhanced MCP Tools with Agent Support

```rust
use rmcp::{Tool as McpTool, ToolDefinition as McpToolDefinition};

pub struct AgentCommitTool {
    agent: Arc<CommitAgent>,
}

#[async_trait]
impl McpTool for AgentCommitTool {
    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "git_iris_agent_commit".to_string(),
            description: Some("Generate commit messages using agent-based analysis".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "repository": {
                        "type": "string",
                        "description": "Path to git repository or remote URL"
                    },
                    "gitmoji": {
                        "type": "boolean",
                        "description": "Include gitmoji in commit message"
                    },
                    "instructions": {
                        "type": "string",
                        "description": "Additional instructions for commit generation"
                    }
                },
                "required": ["repository"]
            }),
        }
    }
    
    async fn call(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let options = CommitOptions {
            repository: input["repository"].as_str().unwrap().to_string(),
            gitmoji: input["gitmoji"].as_bool().unwrap_or(false),
            instructions: input["instructions"].as_str().map(String::from),
        };
        
        let message = self.agent.generate_commit_message(options).await?;
        
        Ok(json!({
            "commit_message": message,
            "agent_used": true,
            "tools_called": self.agent.get_tool_call_history()
        }))
    }
}
```

### MCP Server Configuration

```rust
pub async fn create_mcp_handler_with_agents() -> Result<GitIrisHandler> {
    let config = Config::load()?;
    let llm_client = create_llm_client(&config)?;
    
    let mut handler = GitIrisHandler::new();
    
    // Add traditional tools
    handler.add_tool(Box::new(CommitTool::new()));
    handler.add_tool(Box::new(ReviewTool::new()));
    
    // Add agent-based tools
    handler.add_tool(Box::new(AgentCommitTool {
        agent: Arc::new(CommitAgent::new(llm_client.clone(), repo.clone())),
    }));
    
    handler.add_tool(Box::new(AgentReleaseNotesTool {
        agent: Arc::new(ReleaseNotesAgent::new(llm_client.clone(), repo.clone())),
    }));
    
    Ok(handler)
}
```

## Configuration

### Agent-Specific Configuration
```toml
[agents]
# Maximum iterations for agent loops
max_iterations = 20

# Chunk size for processing large diffs
chunk_size = 2000

# Enable agent mode by default
use_agents = true

[agents.commit]
# Commit agent specific settings
analyze_related_files = true
max_context_files = 10

[agents.release_notes]
# Release notes agent settings
group_by_type = true
include_stats = true
analyze_breaking_changes = true

[agents.review]
# Code review agent settings
depth = "comprehensive"
include_suggestions = true
security_analysis = true
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)
1. Set up Rig dependencies and basic agent structure
2. Implement base `GitIrisAgent` class
3. Create tool registry and base tool implementations
4. Set up agent orchestrator

### Phase 2: Tool Implementation (Week 2-3)
1. Implement Git operations tool
2. Implement file analysis tool
3. Implement memory/state management tool
4. Create search and context tools

### Phase 3: Agent Implementation (Week 3-4)
1. Implement commit message agent
2. Implement code review agent
3. Implement release notes agent
4. Implement changelog agent

### Phase 4: MCP Integration (Week 4-5)
1. Create MCP tool wrappers for agents
2. Update MCP server to support agent tools
3. Add configuration for agent mode
4. Ensure backward compatibility

### Phase 5: Testing and Optimization (Week 5-6)
1. Comprehensive testing with large repositories
2. Performance optimization
3. Token usage optimization
4. Documentation and examples

## Benefits

1. **Scalability**: Handle arbitrarily large codebases and diffs through iterative exploration
2. **Intelligence**: Agents can reason about changes and explore context as needed
3. **Flexibility**: Easy to add new tools and capabilities
4. **MCP Integration**: Seamless integration with existing MCP server
5. **Backward Compatibility**: Traditional single-call mode still available

## Future Enhancements

1. **Parallel Agent Execution**: Run multiple agents concurrently for faster processing
2. **Agent Communication**: Allow agents to share findings and collaborate
3. **Custom Tool Plugins**: Allow users to add custom tools via plugins
4. **Agent Personas**: Different agent personalities for different coding styles
5. **Learning**: Agents that learn from repository history and user preferences