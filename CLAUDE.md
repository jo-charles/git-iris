# Git-Iris Developer Guide

## Architecture Overview

Git-Iris uses an agent-first architecture powered by **Iris**, an LLM-driven agent built on the [Rig framework](https://docs.rs/rig-core). Iris dynamically explores codebases using tool calls rather than dumping all context upfront.

### Core Principle
The LLM makes all intelligent decisions. We avoid deterministic heuristics—Iris decides which tools to use, manages her own context, and adapts her approach based on what she learns.

## Project Structure

```
src/
├── agents/                 # Agent framework
│   ├── iris.rs            # Main agent implementation
│   ├── core.rs            # Backend abstraction (OpenAI/Anthropic)
│   ├── setup.rs           # Agent setup service
│   ├── status.rs          # Real-time status tracking
│   ├── capabilities/      # Task-specific prompts (TOML)
│   │   ├── commit.toml
│   │   ├── review.toml
│   │   ├── pr.toml
│   │   ├── changelog.toml
│   │   └── release_notes.toml
│   └── tools/             # Tools Iris can use
│       ├── git.rs         # Git operations (diff, log, status)
│       ├── file_analyzer.rs # File content analysis
│       ├── code_search.rs # Pattern searching
│       └── workspace.rs   # Notes and task tracking
├── cli.rs                 # CLI entry point
├── commit/                # Commit message generation
├── changes/               # Changelog/release notes
├── config.rs              # Configuration management
├── git.rs                 # Git2 wrapper
└── llm.rs                 # LLM provider abstraction
```

## Agent Mode (Default)

Agent mode is enabled by default. Use `--legacy` for the old non-agent implementation.

```bash
git-iris gen                    # Agent mode (default)
git-iris gen --legacy           # Legacy mode
git-iris gen --debug            # Agent mode with debug output
```

## Capabilities

Each capability is defined in `src/agents/capabilities/*.toml`:

- **commit** - Generate commit messages from staged changes
- **review** - Comprehensive code reviews with severity ratings
- **pr** - Pull request descriptions
- **changelog** - Structured changelogs (Keep a Changelog format)
- **release_notes** - Release notes with highlights and sections

Capabilities define:
- `name` - Capability identifier
- `output_type` - Expected JSON schema (e.g., `GeneratedMessage`, `GeneratedReview`)
- `task_prompt` - Instructions for Iris

## Tools Available to Iris

| Tool | Purpose |
|------|---------|
| `git_diff` | Get staged/commit changes with relevance scores |
| `git_log` | Recent commit history |
| `git_status` | Repository status |
| `git_changed_files` | List of changed files |
| `file_analyzer` | Deep file analysis (content, metadata) |
| `code_search` | Search for patterns, functions, classes |
| `workspace` | Iris's notes and task tracking |
| `project_docs` | Read project documentation |

## Output Types

Iris produces structured JSON matching these schemas (in `src/commit/types.rs` and `src/changes/models.rs`):

- `GeneratedMessage` - Commit message (emoji, title, body)
- `GeneratedPullRequest` - PR description with sections
- `GeneratedReview` - Code review with dimension analysis
- `ChangelogResponse` - Changelog sections (Added, Changed, Fixed, etc.)
- `ReleaseNotesResponse` - Release notes with highlights

## Adding a New Capability

1. Create `src/agents/capabilities/new_capability.toml`:
```toml
name = "my_capability"
description = "What it does"
output_type = "MyOutputType"

task_prompt = """
Instructions for Iris...
"""
```

2. Add output type to `src/agents/iris.rs` `StructuredResponse` enum
3. Add match arm in `execute_task()` for the new output type

## Key Design Decisions

1. **LLM-First**: No hardcoded heuristics—Iris makes decisions
2. **Tool-Based Context**: Gather only what's needed via tool calls
3. **Structured Output**: JSON schemas ensure parseable responses
4. **Output Validation**: Recovery logic handles malformed JSON
5. **Graceful Fallback**: `--legacy` mode if agent fails

## Development Commands

```bash
cargo build                     # Build
cargo test                      # Run tests
cargo clippy                    # Lint
cargo run -- gen --debug        # Test commit generation with debug
RUST_LOG=debug cargo run -- gen # Verbose logging
```

## SilkCircuit Design Language

Git-Iris CLI output follows the **SilkCircuit Neon** color palette for a cohesive, electric aesthetic.

### Color Palette

| Color | Hex | Usage |
|-------|-----|-------|
| Electric Purple | `#e135ff` | Keywords, markers, emphasis |
| Neon Cyan | `#80ffea` | Functions, paths, interactions |
| Coral | `#ff6ac1` | Hashes, numbers, constants |
| Electric Yellow | `#f1fa8c` | Warnings, timestamps |
| Success Green | `#50fa7b` | Success states, confirmations |
| Error Red | `#ff6363` | Errors, danger, removals |

### Semantic Usage

- **Branch names** → Neon Cyan (bold)
- **Commit hashes** → Coral
- **Timestamps** → Electric Yellow
- **Current item markers** → Electric Purple
- **Warnings** → Electric Yellow
- **Success messages** → Success Green
- **Errors** → Error Red

### Implementation

Using the `colored` crate with true color support:

```rust
use colored::Colorize;

// Success message
println!("{}", "✨ Commit created successfully".truecolor(80, 250, 123));

// Error message
println!("{}", "Error: No staged changes".truecolor(255, 99, 99));

// Commit hash
println!("Commit: {}", hash.truecolor(255, 106, 193));
```

### Typography

- Monospace fonts: JetBrains Mono, Fira Code, SF Mono
- Unicode box-drawing for separators: `─`, `━`
- Braille spinners for progress: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`

## Provider Configuration

Set via environment or config:
```bash
export IRIS_PROVIDER=anthropic
export IRIS_MODEL=claude-sonnet-4-5-20250929
export ANTHROPIC_API_KEY=sk-...
```

Or use `git-iris config --provider anthropic --api-key YOUR_KEY`
