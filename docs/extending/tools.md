# Adding Tools

Tools give Iris the ability to inspect your codebase, gather context, and perform operations. This guide shows you how to implement a new tool using the Rig framework.

## What is a Tool?

A tool is a Rust struct that implements the `rig::tool::Tool` trait. When Iris needs information, she can invoke tools by name with specific arguments. The tool executes and returns structured data.

### Tool Lifecycle

```
1. Iris decides she needs information
2. Iris calls tool by name: git_diff(from="main", to="HEAD")
3. Tool executes and returns structured output
4. Iris incorporates the result into her reasoning
5. Iris may call more tools or produce final output
```

## Tool Trait Requirements

Every tool must implement:

```rust
use rig::tool::Tool;
use rig::completion::ToolDefinition;

impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    type Error = MyToolError;
    type Args = MyToolArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        // Return tool metadata for LLM
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Execute tool logic
    }
}
```

## Step-by-Step: Creating a Tool

### Example: Dependency Analyzer

Let's build a tool that analyzes project dependencies.

### Step 1: Create the Tool File

Create `src/agents/tools/dependency_analyzer.rs`:

```rust
//! Dependency analyzer tool for Iris
//!
//! Analyzes project dependencies from package manifests.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::common::parameters_schema;

// Define error type using the standard macro
crate::define_tool_error!(DependencyAnalyzerError);

/// Dependency analyzer tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalyzer;

/// Arguments for dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DependencyAnalyzerArgs {
    /// Type of manifest to analyze (cargo, npm, pip, etc.)
    #[serde(default)]
    pub manifest_type: Option<String>,
    /// Whether to include dev dependencies
    #[serde(default)]
    pub include_dev: bool,
}

impl Tool for DependencyAnalyzer {
    const NAME: &'static str = "dependency_analyzer";
    type Error = DependencyAnalyzerError;
    type Args = DependencyAnalyzerArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "dependency_analyzer".to_string(),
            description: "Analyze project dependencies from package manifests (Cargo.toml, package.json, requirements.txt)".to_string(),
            parameters: parameters_schema::<DependencyAnalyzerArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Get current working directory
        let repo_path = std::env::current_dir()
            .map_err(|e| DependencyAnalyzerError(format!("Failed to get CWD: {}", e)))?;

        // Detect manifest type if not specified
        let manifest_type = match args.manifest_type.as_deref() {
            Some(t) => t.to_string(),
            None => detect_manifest_type(&repo_path)?,
        };

        // Read and parse manifest
        let dependencies = match manifest_type.as_str() {
            "cargo" => parse_cargo_toml(&repo_path, args.include_dev)?,
            "npm" => parse_package_json(&repo_path, args.include_dev)?,
            "pip" => parse_requirements_txt(&repo_path)?,
            _ => return Err(DependencyAnalyzerError(format!(
                "Unsupported manifest type: {}",
                manifest_type
            ))),
        };

        Ok(dependencies)
    }
}

/// Detect manifest type from files present
fn detect_manifest_type(repo_path: &PathBuf) -> Result<String, DependencyAnalyzerError> {
    if repo_path.join("Cargo.toml").exists() {
        Ok("cargo".to_string())
    } else if repo_path.join("package.json").exists() {
        Ok("npm".to_string())
    } else if repo_path.join("requirements.txt").exists() {
        Ok("pip".to_string())
    } else {
        Err(DependencyAnalyzerError(
            "No recognized dependency manifest found".to_string(),
        ))
    }
}

/// Parse Cargo.toml
fn parse_cargo_toml(
    repo_path: &PathBuf,
    include_dev: bool,
) -> Result<String, DependencyAnalyzerError> {
    use std::fs;

    let cargo_path = repo_path.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|e| DependencyAnalyzerError(format!("Failed to read Cargo.toml: {}", e)))?;

    let cargo_toml: toml::Value = toml::from_str(&content)
        .map_err(|e| DependencyAnalyzerError(format!("Failed to parse Cargo.toml: {}", e)))?;

    let mut output = String::from("## Rust Dependencies (Cargo.toml)\n\n");

    // Regular dependencies
    if let Some(deps) = cargo_toml.get("dependencies").and_then(|v| v.as_table()) {
        output.push_str("### Dependencies\n");
        for (name, value) in deps {
            let version = extract_version(value);
            output.push_str(&format!("- {} = {}\n", name, version));
        }
        output.push('\n');
    }

    // Dev dependencies
    if include_dev {
        if let Some(dev_deps) = cargo_toml.get("dev-dependencies").and_then(|v| v.as_table()) {
            output.push_str("### Dev Dependencies\n");
            for (name, value) in dev_deps {
                let version = extract_version(value);
                output.push_str(&format!("- {} = {}\n", name, version));
            }
        }
    }

    Ok(output)
}

/// Parse package.json
fn parse_package_json(
    repo_path: &PathBuf,
    include_dev: bool,
) -> Result<String, DependencyAnalyzerError> {
    use std::fs;

    let package_path = repo_path.join("package.json");
    let content = fs::read_to_string(&package_path)
        .map_err(|e| DependencyAnalyzerError(format!("Failed to read package.json: {}", e)))?;

    let package: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| DependencyAnalyzerError(format!("Failed to parse package.json: {}", e)))?;

    let mut output = String::from("## JavaScript/TypeScript Dependencies (package.json)\n\n");

    // Regular dependencies
    if let Some(deps) = package.get("dependencies").and_then(|v| v.as_object()) {
        output.push_str("### Dependencies\n");
        for (name, value) in deps {
            let version = value.as_str().unwrap_or("*");
            output.push_str(&format!("- {} @ {}\n", name, version));
        }
        output.push('\n');
    }

    // Dev dependencies
    if include_dev {
        if let Some(dev_deps) = package.get("devDependencies").and_then(|v| v.as_object()) {
            output.push_str("### Dev Dependencies\n");
            for (name, value) in dev_deps {
                let version = value.as_str().unwrap_or("*");
                output.push_str(&format!("- {} @ {}\n", name, version));
            }
        }
    }

    Ok(output)
}

/// Parse requirements.txt
fn parse_requirements_txt(repo_path: &PathBuf) -> Result<String, DependencyAnalyzerError> {
    use std::fs;

    let req_path = repo_path.join("requirements.txt");
    let content = fs::read_to_string(&req_path).map_err(|e| {
        DependencyAnalyzerError(format!("Failed to read requirements.txt: {}", e))
    })?;

    let mut output = String::from("## Python Dependencies (requirements.txt)\n\n");

    for line in content.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            output.push_str(&format!("- {}\n", trimmed));
        }
    }

    Ok(output)
}

/// Extract version from TOML value (handles both string and table formats)
fn extract_version(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}
```

### Step 2: Add to Module Exports

Edit `src/agents/tools/mod.rs`:

```rust
pub mod dependency_analyzer;
pub use dependency_analyzer::DependencyAnalyzer;
```

### Step 3: Register in Tool Registry

Edit `src/agents/tools/registry.rs`:

```rust
#[macro_export]
macro_rules! attach_core_tools {
    ($builder:expr) => {{
        use $crate::agents::debug_tool::DebugTool;
        use $crate::agents::tools::{
            CodeSearch, DependencyAnalyzer, FileRead, GitChangedFiles,
            GitDiff, GitLog, GitStatus, ProjectDocs,
        };

        $builder
            .tool(DebugTool::new(GitStatus))
            .tool(DebugTool::new(GitDiff))
            .tool(DebugTool::new(GitLog))
            .tool(DebugTool::new(GitChangedFiles))
            .tool(DebugTool::new(FileRead))
            .tool(DebugTool::new(CodeSearch))
            .tool(DebugTool::new(ProjectDocs))
            .tool(DebugTool::new(DependencyAnalyzer))  // Add here
    }};
}

pub const CORE_TOOLS: &[&str] = &[
    "git_status",
    "git_diff",
    "git_log",
    "git_changed_files",
    "file_read",
    "code_search",
    "project_docs",
    "dependency_analyzer",  // Add here
];
```

### Step 4: Test Your Tool

```bash
# Build
cargo build

# Test with debug mode to see tool calls
cargo run -- gen --debug

# You can also test tools directly in unit tests
cargo test dependency_analyzer
```

## Tool Design Patterns

### Pattern 1: Simple Query Tool

Returns information based on arguments:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleQueryTool;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SimpleQueryArgs {
    pub query: String,
}

impl Tool for SimpleQueryTool {
    const NAME: &'static str = "simple_query";
    type Error = SimpleQueryError;
    type Args = SimpleQueryArgs;
    type Output = String;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Process query and return results
        Ok(format!("Results for: {}", args.query))
    }
}
```

### Pattern 2: Stateful Tool

Maintains internal state across calls:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulTool {
    #[serde(skip)]
    state: Arc<Mutex<ToolState>>,
}

#[derive(Debug, Default)]
struct ToolState {
    cache: HashMap<String, String>,
}

impl StatefulTool {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ToolState::default())),
        }
    }
}

impl Tool for StatefulTool {
    // ... implementation uses self.state
}
```

**Example**: `Workspace` tool (see `src/agents/tools/workspace.rs`)

### Pattern 3: Repository-Aware Tool

Accesses Git repository data:

```rust
use crate::git::GitRepo;
use super::common::get_current_repo;

impl Tool for GitAwareTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let repo = get_current_repo().map_err(GitAwareError::from)?;

        // Use repo methods
        let branch = repo.get_current_branch()?;
        let files = repo.extract_files_info(false)?;

        // Process and return
        Ok(format!("Branch: {}, Files: {}", branch, files.staged_files.len()))
    }
}
```

**Example**: `GitDiff`, `GitLog`, `GitStatus` (see `src/agents/tools/git.rs`)

### Pattern 4: File System Tool

Reads files and analyzes content:

```rust
use std::fs;
use std::path::PathBuf;

impl Tool for FileSystemTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = PathBuf::from(&args.file_path);

        // Read file
        let content = fs::read_to_string(&path)
            .map_err(|e| FileSystemError(format!("Failed to read file: {}", e)))?;

        // Analyze
        let line_count = content.lines().count();

        Ok(format!("File has {} lines", line_count))
    }
}
```

**Example**: `FileRead` (see `src/agents/tools/file_read.rs`)

## Best Practices

### 1. Clear Tool Descriptions

The `description` field in `ToolDefinition` is what Iris sees. Make it actionable:

```rust
ToolDefinition {
    name: "dependency_analyzer".to_string(),
    description: "Analyze project dependencies from package manifests. Auto-detects Cargo.toml, package.json, or requirements.txt. Use include_dev=true for dev dependencies.".to_string(),
    parameters: parameters_schema::<DependencyAnalyzerArgs>(),
}
```

**Good**: "Analyze project dependencies from package manifests"
**Bad**: "A tool for dependencies"

### 2. Useful Default Arguments

Use `#[serde(default)]` for optional arguments:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MyToolArgs {
    /// Required query
    pub query: String,

    /// Optional limit (defaults to 10)
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Optional flag (defaults to false)
    #[serde(default)]
    pub include_extra: bool,
}

fn default_limit() -> usize {
    10
}
```

### 3. Structured Output

Return data in a format Iris can parse and reason about:

```rust
// Good - structured sections
Ok(format!(
    "## Summary\n{}\n\n## Details\n{}\n\n## Recommendations\n{}",
    summary, details, recommendations
))

// Bad - unstructured text
Ok(format!("{} {} {}", summary, details, recommendations))
```

### 4. Error Handling

Use descriptive errors:

```rust
// Good
Err(DependencyAnalyzerError(format!(
    "No package.json found in {}. Make sure you're in a Node.js project.",
    repo_path.display()
)))

// Bad
Err(DependencyAnalyzerError("File not found".to_string()))
```

### 5. Performance Considerations

**Cache expensive operations:**

```rust
#[derive(Debug, Clone)]
pub struct CachedTool {
    #[serde(skip)]
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl Tool for CachedTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(cached) = cache.get(&args.query) {
            return Ok(cached.clone());
        }

        let result = expensive_operation(&args.query)?;
        cache.insert(args.query.clone(), result.clone());
        Ok(result)
    }
}
```

**Limit output size:**

```rust
// Truncate large outputs
let mut output = generate_output();
if output.len() > 10_000 {
    output.truncate(10_000);
    output.push_str("\n\n... (output truncated)");
}
Ok(output)
```

### 6. Test Your Tool

Write unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dependency_analyzer() {
        let tool = DependencyAnalyzer;
        let args = DependencyAnalyzerArgs {
            manifest_type: Some("cargo".to_string()),
            include_dev: false,
        };

        let result = tool.call(args).await;
        assert!(result.is_ok());
    }
}
```

## Real-World Examples

### Git Diff Tool

From `src/agents/tools/git.rs`:

**Key features:**

- Multiple detail levels (`summary`, `standard`)
- Relevance scoring to prioritize files
- Size guidance for agents
- Flexible ref arguments

**Study this for**: Repository operations, scoring algorithms, output formatting

### File Analyzer Tool

From `src/agents/tools/file_analyzer.rs`:

**Key features:**

- Multi-file batch analysis
- Language detection
- Complexity metrics
- Dependency extraction

**Study this for**: File system operations, syntax analysis, structured output

### Code Search Tool

From `src/agents/tools/code_search.rs`:

**Key features:**

- Pattern matching across codebase
- Language-aware search
- Context around matches
- Result ranking

**Study this for**: Search implementations, regex patterns, result formatting

### Workspace Tool

From `src/agents/tools/workspace.rs`:

**Key features:**

- Stateful note-taking
- Task management
- Multiple action types
- Internal state synchronization

**Study this for**: Stateful tools, action-based interfaces, concurrent access

## Common Tool Helpers

Use the shared utilities in `src/agents/tools/common.rs`:

```rust
use super::common::{get_current_repo, parameters_schema};

// Get current Git repository
let repo = get_current_repo()?;

// Generate JSON schema for args
let params = parameters_schema::<MyToolArgs>();
```

## Error Type Macro

Use the standard error macro:

```rust
// At top of your tool file
crate::define_tool_error!(MyToolError);

// Now you can use MyToolError(String) in your tool
```

This creates a consistent error type that works with the `Tool` trait.

## Debugging Tools

Test tool execution with debug mode:

```bash
cargo run -- gen --debug
```

This shows:

- Which tools Iris calls
- Arguments passed to each tool
- Tool output
- Iris's reasoning about the results

## Integration with Capabilities

Reference your tool in capability TOML files:

```toml
task_prompt = """
## Tools Available
- `dependency_analyzer(manifest_type, include_dev)` - Analyze project dependencies
- `git_diff()` - Get code changes
- `file_analyzer()` - Analyze specific files

## Workflow
1. Use `dependency_analyzer()` to understand project tech stack
2. Then analyze relevant source files with `file_analyzer()`
"""
```

## Next Steps

- **Create capabilities** that use your tool → [Adding Capabilities](./capabilities.md)
- **Add Studio modes** to surface tool data → [Adding Studio Modes](./modes.md)
- **Contribute** your tool back → [Contributing](./contributing.md)
