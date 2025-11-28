# Tool System

Tools are functions that Iris calls to gather information. Built on the [Rig framework](https://docs.rs/rig-core), they provide structured, type-safe interfaces for code analysis and Git operations.

**Location:** `src/agents/tools/`

## Design Philosophy

### Tools Provide Data, Not Decisions

A critical principle of Git-Iris:

- **Tools return structured information** (diffs, file contents, commit history)
- **Iris makes decisions** (what's important, how to describe changes)
- **No hardcoded heuristics** in tools for determining commit messages or review priorities

This ensures the LLM drives intelligence while tools stay focused on data access.

### Type-Safe Interfaces

All tools use Rig's `Tool` trait:

```rust
#[async_trait::async_trait]
pub trait Tool {
    const NAME: &'static str;
    type Error;
    type Args: JsonSchema + DeserializeOwned;
    type Output: Serialize;

    async fn definition(&self, _: String) -> ToolDefinition;
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}
```

**Benefits:**

- Automatic JSON schema generation for arguments
- Type-safe argument parsing
- Structured error handling
- Self-documenting tool definitions

## Core Tools

### Git Operations

#### `git_status`

**Purpose:** Get repository status

**Arguments:**

```rust
pub struct GitStatusArgs {
    pub include_unstaged: bool,  // Default: false
}
```

**Returns:** String with branch name and file list

**Example:**

```
Branch: main
Files changed: 3
  src/agents/iris.rs: Modified
  src/types/commit.rs: Added
  README.md: Modified
```

#### `git_diff`

**Purpose:** Get staged changes with relevance scoring and semantic analysis

**Arguments:**

```rust
pub struct GitDiffArgs {
    pub detail: DetailLevel,        // summary | minimal | full
    pub from_ref: Option<String>,   // For PR/review (e.g., "main")
    pub to_ref: Option<String>,     // Default: HEAD
}

pub enum DetailLevel {
    Summary,  // File list + stats, no diffs
    Minimal,  // High-relevance files only
    Full,     // All files with complete diffs
}
```

**Returns:** Formatted diff with metadata

**Key features:**

- **Relevance scoring** (0.0-1.0) for each file
- **Semantic change detection** (function additions, type changes, refactors)
- **Size guidance** for context strategy
- **Sorted by relevance** (most important first)

**Example output:**

```
=== DIFF SUMMARY ===
Size: Medium (8 files, 347 lines changed)
Guidance: Focus on files with >60% relevance (top 5 shown)

=== CHANGES (sorted by relevance) ===

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📄 src/agents/iris.rs [MODIFIED] ★★★★★ 95% relevance
   Reasons: source code, core source, substantive changes, adds function
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

@@ -310,6 +310,15 @@
 pub struct IrisAgent {
+    /// Fast model for subagents
+    fast_model: Option<String>,
}
...
```

**Relevance scoring** considers:

- Change type (added > modified > deleted)
- File type (source > config > docs)
- Path patterns (src/ > test/)
- Diff size (substantive preferred over trivial or massive)
- Semantic patterns (function defs, imports, types)

See [Context Strategy](./context.md) for the algorithm.

#### `git_log`

**Purpose:** Fetch recent commits for style reference

**Arguments:**

```rust
pub struct GitLogArgs {
    pub count: usize,  // Number of commits (default: 5)
}
```

**Returns:** Recent commit messages

**Example:**

```
✨ Add parallel analysis for large changesets
♻️ Refactor agent builder for Send safety
📝 Update documentation with architecture diagrams
```

Iris uses this to match project style.

#### `git_changed_files`

**Purpose:** Get list of changed files without diffs

**Arguments:** None

**Returns:** Simple file list

**Example:**

```
src/agents/iris.rs (Modified)
src/types/commit.rs (Added)
README.md (Modified)
```

Useful for quick changeset overview.

### File Operations

#### `file_read`

**Purpose:** Read file contents directly

**Arguments:**

```rust
pub struct FileReadArgs {
    pub path: String,              // File path
    pub start_line: Option<usize>, // Optional: line to start from
    pub num_lines: Option<usize>,  // Optional: number of lines
}
```

**Returns:** File contents with optional range

**Example:**

```rust
// Read entire file
file_read({ "path": "src/agents/iris.rs" })

// Read specific range
file_read({
    "path": "src/agents/iris.rs",
    "start_line": 100,
    "num_lines": 50
})
```

**When to use:**

- Reading project configuration (Cargo.toml, package.json)
- Examining specific functions or modules
- Understanding context not visible in diffs

#### `code_search`

**Purpose:** Search for patterns, symbols, or text across files

**Arguments:**

```rust
pub struct CodeSearchArgs {
    pub pattern: String,           // Regex pattern
    pub file_pattern: Option<String>, // Optional: file glob (e.g., "*.rs")
    pub context_lines: usize,      // Lines of context (default: 2)
}
```

**Returns:** Matches with context

**Example:**

```rust
// Find all function definitions
code_search({
    "pattern": "pub fn ",
    "file_pattern": "src/**/*.rs"
})

// Find specific API usage
code_search({
    "pattern": "execute_with_agent",
    "context_lines": 5
})
```

**Best practices:**

- Use sparingly — `file_read` is better for known files
- Prefer specific patterns over broad searches
- Limit file patterns to relevant areas

### Project Documentation

#### `project_docs`

**Purpose:** Read project documentation and conventions

**Arguments:**

```rust
pub struct ProjectDocsArgs {
    pub doc_type: String,  // "readme", "agents", "claude", "context"
}
```

**Returns:** Document contents

**Doc types:**

- `readme` — Project README
- `agents` — AGENTS.md (agent-specific conventions)
- `claude` — CLAUDE.md (project-specific LLM instructions)
- `context` — All of the above concatenated

**Example:**

```rust
// Get full project context
project_docs({ "doc_type": "context" })
```

**Why this matters:**

Every project has conventions (commit style, terminology, architecture patterns). Iris reads these first to align her output with project standards.

### Repository Metadata

#### `git_repo_info`

**Purpose:** Get repository metadata

**Arguments:** None

**Returns:** JSON with repo details

**Example:**

```json
{
  "path": "/Users/user/git-iris",
  "branch": "main",
  "remote": "https://github.com/user/git-iris",
  "commit_count": 342
}
```

Useful for including repo URLs in PR descriptions or release notes.

### Agent Delegation

#### `workspace`

**Purpose:** Iris's persistent notes and task tracking

**Arguments:**

```rust
pub struct WorkspaceArgs {
    pub action: String,  // "add", "list", "clear"
    pub note: Option<String>,
}
```

**Returns:** Current workspace state

**Example:**

```rust
// Add a note
workspace({ "action": "add", "note": "Auth changes affect 3 modules" })

// List notes
workspace({ "action": "list" })

// Clear all notes
workspace({ "action": "clear" })
```

**Use case:** Iris tracks findings across multiple tool calls, building up context before generating final output.

#### `parallel_analyze`

**Purpose:** Spawn concurrent subagents for large tasks

**Arguments:**

```rust
pub struct ParallelAnalyzeArgs {
    pub tasks: Vec<String>,  // List of focused prompts
}
```

**Returns:** Aggregated results

**Example:**

```rust
parallel_analyze({
    "tasks": [
        "Analyze authentication changes in src/auth/",
        "Review API endpoint changes in src/api/",
        "Check database migration in migrations/"
    ]
})
```

**Returns:**

```json
{
  "results": [
    {
      "task": "Analyze authentication changes...",
      "result": "The auth module adds OAuth2 support...",
      "success": true
    },
    {
      "task": "Review API endpoint changes...",
      "result": "Three new endpoints added for user management...",
      "success": true
    }
  ],
  "successful": 2,
  "failed": 0,
  "execution_time_ms": 3421
}
```

**How it works:**

1. Spawns N independent subagents (using fast model)
2. Each subagent has core tools (`git_diff`, `file_read`, etc.)
3. Runs concurrently with separate context windows
4. Main agent synthesizes results

**When to use:**

- Changesets >20 files or >1000 lines
- Batch commit analysis
- Multi-module refactors

See [Context Strategy](./context.md) for decision criteria.

#### `analyze_subagent`

**Purpose:** Delegate a single focused task to a sub-agent

**Arguments:** Free-form prompt string

**Returns:** Sub-agent's analysis

**Example:**

```rust
analyze_subagent("Analyze the security implications of changes in src/auth/oauth.rs")
```

**Difference from `parallel_analyze`:**

- Single task vs. multiple concurrent tasks
- Simpler interface
- Use for deep dives on specific files/modules

### Content Update Tools (Studio Only)

These tools are only available in Studio chat mode:

#### `update_commit`

Update the current commit message in Studio.

#### `update_pr`

Update the current PR description in Studio.

#### `update_review`

Update the current review content in Studio.

**Example Studio interaction:**

```
User: "Make the commit message more concise"
Iris: [Calls update_commit with revised message]
```

## Tool Registry

To ensure consistency between main agents and subagents, Git-Iris uses a **tool registry macro**:

**Source:** `src/agents/tools/registry.rs`

```rust
#[macro_export]
macro_rules! attach_core_tools {
    ($builder:expr) => {{
        $builder
            .tool(DebugTool::new(GitStatus))
            .tool(DebugTool::new(GitDiff))
            .tool(DebugTool::new(GitLog))
            .tool(DebugTool::new(GitChangedFiles))
            .tool(DebugTool::new(FileRead))
            .tool(DebugTool::new(CodeSearch))
            .tool(DebugTool::new(ProjectDocs))
    }};
}
```

**Usage:**

```rust
// Main agent
let agent = attach_core_tools!(builder)
    .tool(GitRepoInfo)       // Main agent only
    .tool(Workspace::new())  // Main agent only
    .tool(ParallelAnalyze::new(...)) // Main agent only
    .build();

// Subagent
let sub_agent = attach_core_tools!(sub_builder)
    .build();  // No delegation tools (prevents recursion)
```

**Benefits:**

- Subagents always have the same analysis capabilities
- Changes to tool set apply everywhere
- No drift between agent implementations

## Creating a Custom Tool

### Step 1: Define Arguments and Output

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolArgs {
    pub input: String,
    #[serde(default)]
    pub optional_flag: bool,
}

#[derive(Debug, Serialize)]
pub struct MyToolOutput {
    pub result: String,
    pub metadata: HashMap<String, String>,
}
```

### Step 2: Implement the Tool

```rust
use rig::tool::Tool;
use rig::completion::ToolDefinition;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyTool;

impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    type Error = anyhow::Error;
    type Args = MyToolArgs;
    type Output = MyToolOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "my_tool".to_string(),
            description: "What this tool does and when to use it".to_string(),
            parameters: crate::agents::tools::parameters_schema::<MyToolArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        // Implement tool logic
        let result = format!("Processed: {}", args.input);

        Ok(MyToolOutput {
            result,
            metadata: HashMap::new(),
        })
    }
}
```

### Step 3: Add to Agent

```rust
let agent = client.agent(model)
    .tool(DebugTool::new(MyTool))
    .build();
```

### Step 4: Test

```bash
cargo test
```

## Tool Design Best Practices

### 1. Single Responsibility

Each tool should do **one thing well**:

✅ **Good:** `git_diff` — Returns diffs with metadata
❌ **Bad:** `analyze_and_generate_commit` — Mixes analysis and generation

### 2. Structured Output

Return structured data, not formatted text:

✅ **Good:**

```rust
pub struct DiffOutput {
    pub files: Vec<FileChange>,
    pub total_lines: usize,
    pub size_category: String,
}
```

❌ **Bad:**

```rust
pub struct DiffOutput {
    pub formatted_text: String,  // Unstructured
}
```

### 3. Clear Descriptions

Tool descriptions should explain:

- **What** the tool does
- **When** to use it
- **What** it returns

Example:

```rust
description: "Get staged changes with relevance scores. Use this to see what's \
              changed and prioritize files for analysis. Returns diffs sorted \
              by importance with semantic change detection."
```

### 4. Sensible Defaults

Make common use cases simple:

```rust
#[derive(JsonSchema)]
pub struct GitDiffArgs {
    #[serde(default = "DetailLevel::default")]
    pub detail: DetailLevel,  // Defaults to Full
}
```

### 5. Error Context

Provide helpful error messages:

```rust
Err(anyhow::anyhow!(
    "Failed to read file '{}': {}. Make sure the path is relative to repo root.",
    path,
    e
))
```

## Debug Wrapper

All tools are wrapped in `DebugTool` for instrumentation:

```rust
pub struct DebugTool<T> {
    inner: T,
}

impl<T: Tool> Tool for DebugTool<T> {
    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        debug::debug_tool_call(Self::NAME, &args);
        let timer = debug::DebugTimer::start(format!("Tool: {}", Self::NAME));

        let result = self.inner.call(args).await;

        timer.finish();
        if result.is_ok() {
            debug::debug_tool_success(Self::NAME);
        } else {
            debug::debug_tool_error(Self::NAME, &format!("{:?}", result));
        }

        result
    }
}
```

Enable with `--debug` for color-coded tool execution traces.

## Testing Tools

### Unit Tests

Test tool logic directly:

```rust
#[tokio::test]
async fn test_git_diff() {
    let tool = GitDiff;
    let args = GitDiffArgs {
        detail: DetailLevel::Summary,
        from_ref: None,
        to_ref: None,
    };

    let result = tool.call(args).await.unwrap();
    assert!(result.contains("DIFF SUMMARY"));
}
```

### Integration Tests

Test tools within agent context:

```rust
#[tokio::test]
async fn agent_uses_git_diff() {
    let agent = IrisAgent::new("openai", "gpt-4o").unwrap();
    let response = agent.execute_task("commit", "Generate message").await.unwrap();

    // Verify the agent called git_diff and produced output
    assert!(matches!(response, StructuredResponse::CommitMessage(_)));
}
```

## Common Patterns

### Pagination

For large results:

```rust
pub struct SearchArgs {
    pub pattern: String,
    pub max_results: usize,  // Default: 50
    pub offset: usize,       // Default: 0
}
```

### Context Windows

For reading large files:

```rust
pub struct FileReadArgs {
    pub path: String,
    pub start_line: Option<usize>,
    pub num_lines: Option<usize>,  // Default: entire file
}
```

### Progressive Detail

Offer multiple detail levels:

```rust
pub enum DetailLevel {
    Summary,  // Quick overview
    Minimal,  // Key items only
    Full,     // Everything
}
```

Iris can start with `Summary` and drill down if needed.

## Performance Considerations

### Lazy Evaluation

Compute expensive operations only when needed:

```rust
// ✅ Good: Compute relevance only if detail level requires it
if matches!(args.detail, DetailLevel::Full | DetailLevel::Minimal) {
    calculate_relevance_scores(&files);
}
```

### Caching

Tools can cache expensive results:

```rust
static REPO_INFO_CACHE: OnceCell<RepoInfo> = OnceCell::new();

async fn call(&self, _: Args) -> Result<Output> {
    let info = REPO_INFO_CACHE.get_or_try_init(|| {
        expensive_repo_scan()
    })?;
    Ok(info.clone())
}
```

### Parallel Execution

Tools can use concurrency internally:

```rust
let results = futures::future::join_all(
    files.iter().map(|f| analyze_file(f))
).await;
```

## Next Steps

- [Capabilities](./capabilities.md) — How tools are used in task prompts
- [Agent System](./agent.md) — How agents call tools
- [Context Strategy](./context.md) — Relevance scoring algorithm
