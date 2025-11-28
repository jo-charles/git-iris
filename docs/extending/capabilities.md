# Adding Capabilities

Capabilities define what tasks Iris can perform. Each capability is a TOML file containing a prompt and output schema. This guide shows you how to add a new capability to Git-Iris.

## What is a Capability?

A capability combines three elements:

1. **Task Prompt** — Instructions for Iris on how to approach the task
2. **Output Type** — Structured format for the response (JSON schema)
3. **Tool Access** — Which tools Iris can use for this task

## Capability Structure

### Basic TOML Format

```toml
name = "my_capability"
description = "Brief description of what this does"
output_type = "MyOutputType"

task_prompt = """
Instructions for Iris go here.

## Tools Available
- `git_diff()` - Get code changes
- `file_analyzer()` - Analyze specific files

## Output Requirements
Your requirements for the output format.

## JSON Output
Return a `MyOutputType` with: field1, field2, etc.
"""
```

### Real Example: Commit Message Generation

From `src/agents/capabilities/commit.toml`:

```toml
name = "commit"
description = "Generate commit messages from staged changes"
output_type = "GeneratedMessage"

task_prompt = """
Generate a commit message for the staged changes.

## MANDATORY FIRST STEP
**ALWAYS call `project_docs(doc_type="context")` FIRST** before any other tool.
This fetches README + AGENTS.md/CLAUDE.md containing project conventions you MUST follow.
Do not skip this step.

## Tools Available
- `project_docs(doc_type="context")` - **CALL FIRST** - Get README + context
- `git_diff()` - Get staged changes with relevance scores
- `git_log(count=5)` - Recent commits for style reference

## Workflow
1. **FIRST**: Call `project_docs(doc_type="context")`
2. Call `git_diff()` to see what changed
3. Generate the commit message following project conventions

## Output Requirements
- **Subject line**: Imperative mood, max 72 chars, no period
- **Body**: Explain WHY, not what. Wrap at 72 chars.
- **Plain text only**: No markdown, no code fences

## JSON Output
Return a `GeneratedMessage` with: `emoji` (string or null), `title` (subject), `message` (body)
"""
```

## Step-by-Step: Adding a New Capability

::: tip Teaching Example
This section walks through creating a hypothetical "Feature Summary" capability. **This capability does not exist in the current codebase** — it's an example to illustrate the pattern. Follow along to learn how capabilities work.
:::

### Step 1: Create the TOML File

Create `src/agents/capabilities/feature_summary.toml`:

```toml
name = "feature_summary"
description = "Generate a high-level summary of a feature branch"
output_type = "FeatureSummary"

task_prompt = """
You are Iris, an AI assistant analyzing a feature branch to create a high-level summary.

## MANDATORY FIRST STEP
**ALWAYS call `project_docs(doc_type="context")` FIRST** to understand the project.

## Tools Available
- `project_docs(doc_type="context")` - Get project context (README, conventions)
- `git_diff(from, to)` - Get changes between branches
- `git_log(count=N)` - Get commit history
- `file_analyzer(paths)` - Analyze specific files in detail

## Workflow
1. Call `project_docs(doc_type="context")` to understand the project
2. Get the diff between main and the feature branch
3. Identify key files and patterns
4. Summarize the feature's purpose, implementation approach, and impact

## Output Requirements
- **Purpose**: 1-2 sentences on what this feature does
- **Approach**: Brief technical overview
- **Files Changed**: Count and categorization
- **Impact**: User-facing changes, API changes, internal refactors

## JSON Output
Return a `FeatureSummary` with:
- `purpose` (string)
- `approach` (string)
- `files_changed` (number)
- `impact` (string)
"""
```

### Step 2: Define the Output Type

Create or update `src/types/feature_summary.rs`:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Feature summary response
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct FeatureSummary {
    /// What this feature does (1-2 sentences)
    pub purpose: String,
    /// Technical approach overview
    pub approach: String,
    /// Number of files changed
    pub files_changed: usize,
    /// User-facing and technical impact
    pub impact: String,
}

impl FeatureSummary {
    /// Format as markdown for display
    pub fn format(&self) -> String {
        format!(
            "# Feature Summary\n\n\
            ## Purpose\n{}\n\n\
            ## Approach\n{}\n\n\
            ## Impact\n{}\n\n\
            Files changed: {}\n",
            self.purpose,
            self.approach,
            self.impact,
            self.files_changed
        )
    }
}
```

Add to `src/types/mod.rs`:

```rust
pub mod feature_summary;
pub use feature_summary::FeatureSummary;
```

### Step 3: Register in StructuredResponse

Edit `src/agents/iris.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredResponse {
    CommitMessage(crate::types::GeneratedMessage),
    PullRequest(crate::types::MarkdownPullRequest),
    Changelog(crate::types::MarkdownChangelog),
    ReleaseNotes(crate::types::MarkdownReleaseNotes),
    MarkdownReview(crate::types::MarkdownReview),
    SemanticBlame(String),
    PlainText(String),
    // Add your new type:
    FeatureSummary(crate::types::FeatureSummary),
}

impl fmt::Display for StructuredResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing arms ...
            StructuredResponse::FeatureSummary(summary) => {
                write!(f, "{}", summary.format())
            }
        }
    }
}
```

### Step 4: Load the Capability

Add the embedded TOML constant in `src/agents/iris.rs`:

```rust
const CAPABILITY_COMMIT: &str = include_str!("capabilities/commit.toml");
const CAPABILITY_PR: &str = include_str!("capabilities/pr.toml");
// ... existing capabilities ...
const CAPABILITY_FEATURE_SUMMARY: &str = include_str!("capabilities/feature_summary.toml");
```

Register in the capability map (find the capability loading code):

```rust
capabilities.insert("feature_summary", CAPABILITY_FEATURE_SUMMARY);
```

### Step 5: Add Execution Logic

In `src/agents/iris.rs`, find the `execute_task` method and add a match arm:

```rust
match capability.output_type.as_str() {
    "GeneratedMessage" => {
        let response: GeneratedMessage = self.execute_with_schema(prompt).await?;
        Ok(StructuredResponse::CommitMessage(response))
    }
    // ... existing arms ...
    "FeatureSummary" => {
        let response: FeatureSummary = self.execute_with_schema(prompt).await?;
        Ok(StructuredResponse::FeatureSummary(response))
    }
    _ => Err(anyhow::anyhow!("Unknown output type: {}", capability.output_type)),
}
```

### Step 6: Test Your Capability

```bash
# Build
cargo build

# Test in CLI (you may need to add a CLI command for your capability)
cargo run -- feature-summary main..feature-branch

# Or test in Studio (if you add a mode for it)
cargo run -- studio
```

## Best Practices

### Prompt Engineering

**Be specific about workflow:**

```toml
## Workflow
1. Call `project_docs(doc_type="context")` first
2. Get the diff with `git_diff()`
3. For files over 500 lines, use `file_analyzer()` for targeted analysis
4. Synthesize findings into output format
```

**Provide clear output requirements:**

```toml
## Output Requirements
- **Title**: Max 100 chars, action-oriented
- **Summary**: 2-3 paragraphs, focus on impact
- **No uncertain language**: State facts definitively, not "likely" or "probably"
```

**Give examples:**

```toml
Example output:
{
  "title": "Add user authentication with JWT",
  "summary": "Implements JWT-based authentication...",
  "impact": "Breaking: All API endpoints now require auth headers"
}
```

### Context Strategy

Guide Iris on how to handle different changeset sizes:

```toml
## Context Strategy by Size
- **Small** (≤3 files, <100 lines): Consider all changes equally
- **Medium** (≤10 files, <500 lines): Focus on files with >60% relevance
- **Large** (>10 files or >500 lines): Use top 5-7 highest-relevance files
- **Very Large** (>20 files): Use `parallel_analyze` to distribute work
```

### Tool Selection

Only list tools relevant to the task:

```toml
## Tools Available
- `git_diff()` - Get changes (use detail="summary" first)
- `git_log(count=5)` - Recent commits for context
- `file_analyzer(paths)` - Deep analysis of specific files
# Don't list every possible tool—keep it focused
```

### Certainty Standards

Enforce definitive language:

```toml
## Writing Standards
- **NEVER use uncertain language**: Avoid "likely", "probably", "possibly",
  "might", "may", "seems", "appears to", "presumably", "could be"
- You have full code access—investigate until you can state facts definitively
- If unsure, use tools to gather more context
```

## Output Type Patterns

### Simple JSON Schema

For structured data:

```rust
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct SimpleOutput {
    pub field1: String,
    pub field2: Vec<String>,
    pub field3: Option<usize>,
}
```

### Markdown Wrapper

For LLM-driven formatting:

```rust
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct MarkdownOutput {
    /// Markdown content (LLM controls structure)
    pub content: String,
}

impl MarkdownOutput {
    pub fn raw_content(&self) -> &str {
        &self.content
    }
}
```

Use markdown wrappers when you want the LLM to control the exact structure while still having parseable output.

## Common Patterns

### Multi-Stage Analysis

```toml
## Workflow
1. Initial scan: `git_diff(detail="summary")` for overview
2. Identify key areas from relevance scores
3. Deep dive: `file_analyzer()` on top 5 files
4. Synthesize into structured output
```

### Parallel Processing

For large changesets:

```toml
## Very Large Changesets (>20 files)
Use `parallel_analyze` to distribute work:
parallel_analyze({
  "tasks": [
    "Analyze API changes in src/api/",
    "Review database schema changes",
    "Check frontend component updates"
  ]
})
Each subagent analyzes independently, then you synthesize.
```

### Style Adaptation

Allow preset-based customization:

```toml
## Style Adaptation
If STYLE INSTRUCTIONS are provided, prioritize that style in your output.
The structural requirements still apply, but adapt tone and word choice.
```

## Integration with Studio

If you add a Studio mode for your capability:

1. Add mode variant to `Mode` enum in `src/studio/state/mod.rs`
2. Create state struct in `src/studio/state/modes.rs`
3. Implement handler and renderer (see [Adding Studio Modes](./modes.md))

## Troubleshooting

### Iris doesn't use the right tools

Make the tool list more explicit and add workflow steps that require specific tools.

### Output parsing fails

Ensure your JSON schema matches exactly. Test with `--debug` to see raw responses.

### Responses are too vague

Add certainty standards and specific output requirements. Show examples.

### Context is incomplete

Guide Iris on when to gather more context. Add size-based strategies.

## Examples in Codebase

Study these real capabilities:

- **`commit.toml`** — Simple structured output, style adaptation
- **`review.toml`** — Markdown wrapper, parallel analysis, size strategies
- **`pr.toml`** — Multi-stage workflow, context gathering
- **`changelog.toml`** — Version comparison, structured formatting

## Next Steps

- **Add tools** to give Iris new context sources → [Adding Tools](./tools.md)
- **Create a mode** to surface your capability in Studio → [Adding Studio Modes](./modes.md)
- **Contribute** your capability back → [Contributing](./contributing.md)
