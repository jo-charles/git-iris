# Capability System

Capabilities define what Iris can do. Each capability is a TOML file that specifies the task prompt and expected output format.

**Location:** `src/agents/capabilities/*.toml`

## Design Philosophy

### Separation of Concerns

- **TOML files** define the task and output type
- **Agent** handles execution and tool calling
- **Types** define the structured response schema

This separation allows:

- **Non-programmers** to modify task instructions
- **Easy experimentation** with different prompts
- **Version control** of prompt engineering
- **Compile-time embedding** for portability

### LLM-Driven Structure

Capabilities don't rigidly enforce structure — they **guide** the LLM. For example:

- **Commit messages:** JSON with specific fields (`emoji`, `title`, `message`)
- **Reviews:** Markdown with suggested sections, but Iris decides final structure
- **PRs:** Markdown with flexibility for project-specific conventions

The LLM adapts to project needs while following general guidelines.

## Capability Structure

A capability TOML has three fields:

```toml
name = "capability_name"
description = "Short description of what this capability does"
output_type = "OutputTypeName"

task_prompt = """
Multi-line prompt that instructs Iris...
"""
```

### Output Types

Output types map to Rust enums in `src/agents/iris.rs`:

```rust
pub enum StructuredResponse {
    CommitMessage(GeneratedMessage),       // JSON: { emoji, title, message }
    PullRequest(MarkdownPullRequest),      // Markdown wrapper
    Changelog(MarkdownChangelog),          // Markdown wrapper
    ReleaseNotes(MarkdownReleaseNotes),    // Markdown wrapper
    MarkdownReview(MarkdownReview),        // Markdown wrapper
    SemanticBlame(String),                 // Plain text
    PlainText(String),                     // Fallback
}
```

**JSON types** (like `GeneratedMessage`) have strict schemas. **Markdown types** wrap a single `content: String` field, giving Iris flexibility in formatting.

## Built-in Capabilities

### 1. Commit (`commit.toml`)

**Purpose:** Generate commit messages from staged changes

**Output:** `GeneratedMessage`

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GeneratedMessage {
    pub emoji: Option<String>,  // Single gitmoji or null
    pub title: String,          // Subject line (max 72 chars)
    pub message: String,        // Body (optional)
}
```

**Key instructions:**

- Always call `project_docs(doc_type="context")` first
- Use `git_diff()` for changes with relevance scores
- Follow project-specific conventions from README/AGENTS.md
- Adapt context strategy based on changeset size
- Use `parallel_analyze` for very large changes

**Style adaptation:**

- Gitmoji mode: Set `emoji` field from gitmoji list
- Conventional mode: Set `emoji` to null, use conventional prefixes
- Presets: Apply personality while maintaining structure

### 2. Review (`review.toml`)

**Purpose:** Analyze code changes and provide structured feedback

**Output:** `MarkdownReview`

**Suggested sections:**

- Overview
- Security Concerns
- Performance Impact
- Code Quality
- Testing Recommendations
- Documentation Needs

**Key instructions:**

- Focus on substantive issues, not nitpicks
- Highlight security and performance concerns
- Suggest concrete improvements
- Consider project context from README/AGENTS.md

### 3. Pull Request (`pr.toml`)

**Purpose:** Generate PR descriptions from branch changes

**Output:** `MarkdownPullRequest`

**Suggested sections:**

- Summary
- Changes
- Test Plan
- Breaking Changes (if any)
- Screenshots/Demos (if applicable)

**Key instructions:**

- Use `git_diff(from_ref="main")` for full branch context
- Analyze entire feature branch, not just latest commit
- Include migration/upgrade notes for breaking changes
- Suggest testing approach

### 4. Changelog (`changelog.toml`)

**Purpose:** Generate changelog entries in Keep a Changelog format

**Output:** `MarkdownChangelog`

**Structure:**

```markdown
## [Version] - YYYY-MM-DD

### Added

- New features

### Changed

- Enhancements to existing features

### Deprecated

- Features marked for removal

### Removed

- Deleted features

### Fixed

- Bug fixes

### Security

- Security patches
```

**Key instructions:**

- Group changes by category
- Be specific about what changed
- Include migration notes if needed
- Focus on user-facing impact

### 5. Release Notes (`release_notes.toml`)

**Purpose:** Generate user-facing release documentation

**Output:** `MarkdownReleaseNotes`

**Suggested sections:**

- Highlights
- Breaking Changes
- New Features
- Improvements
- Bug Fixes
- Performance
- Upgrade Instructions

**Key instructions:**

- Write for end users, not developers
- Highlight impact and benefits
- Include version numbers and dates
- Provide upgrade path for breaking changes

### 6. Chat (`chat.toml`)

**Purpose:** Interactive conversation with Iris in Studio

**Output:** Varies (text or tool calls)

**Special features:**

- Access to content update tools (`update_commit`, `update_pr`, `update_review`)
- Can read and modify current Studio content
- Freeform conversation for exploration

### 7. Semantic Blame (`semantic_blame.toml`)

**Purpose:** Explain the history and reasoning behind code

**Output:** `SemanticBlame` (plain text)

**Key instructions:**

- Read git log for the file/region
- Analyze commit messages and diffs
- Explain _why_ the code evolved this way
- Connect changes to broader project goals

## Creating a Custom Capability

### Step 1: Create the TOML File

Create `src/agents/capabilities/my_capability.toml`:

```toml
name = "my_capability"
description = "What my capability does"
output_type = "MyOutputType"

task_prompt = """
Instructions for Iris on how to complete this task.

## Tools Available
- `git_diff()` - Get changes
- `file_read()` - Read files
- `code_search()` - Search for patterns

## Output Requirements
Describe the expected structure...
"""
```

### Step 2: Define the Output Type

In `src/types/my_output.rs`:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyOutputType {
    pub summary: String,
    pub details: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
```

### Step 3: Add to `StructuredResponse` Enum

In `src/agents/iris.rs`:

```rust
pub enum StructuredResponse {
    // ... existing variants
    MyOutput(MyOutputType),
}

impl fmt::Display for StructuredResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing matches
            StructuredResponse::MyOutput(output) => {
                write!(f, "{}", output.summary)
            }
        }
    }
}
```

### Step 4: Embed the Capability

In `src/agents/iris.rs`, add the constant:

```rust
const CAPABILITY_MY_CAPABILITY: &str = include_str!("capabilities/my_capability.toml");
```

And update the loader:

```rust
fn load_capability_config(&self, capability: &str) -> Result<(String, String)> {
    let content = match capability {
        // ... existing capabilities
        "my_capability" => CAPABILITY_MY_CAPABILITY,
        _ => { /* fallback */ }
    };
    // ...
}
```

### Step 5: Handle Execution

In `execute_task()`, add a match arm:

```rust
match output_type.as_str() {
    // ... existing types
    "MyOutputType" => {
        let response = self.execute_with_agent::<MyOutputType>(
            &system_prompt,
            user_prompt,
        ).await?;
        Ok(StructuredResponse::MyOutput(response))
    }
    // ...
}
```

### Step 6: Test

```bash
cargo build
cargo run -- my-capability
```

## Prompt Engineering Best Practices

### 1. Mandatory First Steps

Always instruct Iris to gather context first:

```toml
task_prompt = """
## MANDATORY FIRST STEP
**ALWAYS call `project_docs(doc_type="context")` FIRST**
This fetches project conventions you MUST follow.
"""
```

### 2. Tool Guidance

List available tools with clear purposes:

```toml
## Tools Available
- `git_diff()` - Get staged changes with relevance scores
- `git_log(count=5)` - Recent commits for style reference
- `file_read(path, start_line, num_lines)` - Read file contents
```

### 3. Size-Based Strategy

Guide Iris on how to handle different changeset sizes:

```toml
## Context Strategy by Size
- **Small** (≤3 files): Consider all changes
- **Large** (>10 files): Focus on high-relevance files
- **Huge** (>20 files): Use `parallel_analyze`
```

### 4. Output Requirements

Be explicit about format:

```toml
## Output Requirements
- **Subject line**: Imperative mood, max 72 chars
- **Body**: Wrap at 72 chars, explain WHY not what
- **Plain text only**: No markdown, no code fences
```

### 5. Avoid Uncertainty

Instruct Iris to be definitive:

```toml
## Writing Guidelines
- **NEVER use speculative language**: Avoid "likely", "probably", "seems"
- If unsure, use tools to investigate
- State facts definitively
```

### 6. Style Flexibility

Allow preset injection:

```toml
## Style Adaptation
If STYLE INSTRUCTIONS are provided, prioritize that style.
A cosmic preset means cosmic language. Express the style!
```

This enables users to inject personality via presets.

## Advanced Patterns

### Conditional Tool Calls

Instruct Iris to adapt:

```toml
If the changeset is large (>20 files or >1000 lines):
  - Use `parallel_analyze` to distribute analysis
  - Example: parallel_analyze({ "tasks": ["Analyze auth/", "Review API/"] })
Otherwise:
  - Use `git_diff()` and `file_read()` directly
```

### Multi-Stage Analysis

Guide a workflow:

```toml
1. Call `git_diff()` to see what changed
2. Identify the primary affected subsystem
3. Call `code_search()` to find related patterns
4. Call `file_read()` for detailed context
5. Synthesize findings into a coherent summary
```

### Project-Specific Adaptation

Leverage project docs:

```toml
After reading `project_docs(doc_type="context")`:
- Follow any commit conventions from AGENTS.md
- Use terminology from README
- Respect project style guide
```

## Validation and Recovery

All JSON outputs go through schema validation:

1. **Schema generation** — `schemars::schema_for!` creates JSON schema from Rust type
2. **Prompt injection** — Schema is added to prompt as a constraint
3. **Response parsing** — `extract_json_from_response()` finds JSON in response
4. **Sanitization** — `sanitize_json_response()` fixes control characters
5. **Validation** — `validate_and_parse()` attempts recovery if parsing fails

See [Output Validation](./output.md) for details.

## Debugging Capabilities

Run with `--debug` to see:

- Which capability is loaded
- The full prompt sent to the LLM
- Tool calls made by Iris
- JSON extraction and validation steps
- Token usage statistics

```bash
git-iris gen --debug
```

Color-coded output shows:

- 🔵 Blue — Phase transitions
- 🟢 Green — Successful operations
- 🟡 Yellow — Warnings
- 🔴 Red — Errors

## Best Practices Summary

✅ **DO:**

- Start with `project_docs()` for context
- Provide clear tool descriptions
- Guide size-based strategies
- Allow style flexibility
- Be explicit about output format

❌ **DON'T:**

- Hardcode project-specific details
- Over-constrain markdown structure
- Assume file locations
- Use speculative language
- Ignore relevance scores

## Next Steps

- [Tools](./tools.md) — Building tools that capabilities can use
- [Output Validation](./output.md) — Schema validation and error recovery
- [Agent System](./agent.md) — How capabilities are executed
