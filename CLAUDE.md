# Git-Iris Developer Guide

## Architecture Overview

Git-Iris uses an agent-first architecture powered by **Iris**, an LLM-driven agent built on the [Rig framework](https://docs.rs/rig-core). Iris dynamically explores codebases using tool calls rather than dumping all context upfront.

### Core Principles

- **LLM-First**: The LLM makes all intelligent decisions—we avoid deterministic heuristics
- **Tool-Based Context**: Iris gathers precisely what she needs via tool calls
- **Unified Interface**: Studio provides a single TUI for all capabilities
- **Event-Driven State**: Pure reducer pattern for predictable, testable state management

## Project Structure

```
src/
├── agents/                 # Agent framework (core of Git-Iris)
│   ├── iris.rs            # Main agent implementation
│   ├── core.rs            # Backend abstraction (OpenAI/Anthropic/Google)
│   ├── setup.rs           # IrisAgentService entry point
│   ├── status.rs          # Real-time status tracking
│   ├── debug.rs           # Debug mode output formatting
│   ├── capabilities/      # Task-specific prompts (TOML)
│   │   ├── commit.toml    # Commit message generation
│   │   ├── review.toml    # Code review analysis
│   │   ├── pr.toml        # PR description generation
│   │   ├── changelog.toml # Changelog generation
│   │   ├── release_notes.toml # Release notes
│   │   └── chat.toml      # Interactive chat capability
│   └── tools/             # Tools Iris can use
│       ├── git.rs         # Git operations (diff, log, status)
│       ├── file_analyzer.rs # File content analysis
│       ├── code_search.rs # Pattern searching
│       ├── workspace.rs   # Notes and task tracking
│       ├── project_docs.rs # Project documentation
│       ├── parallel_analyze.rs # Concurrent subagent processing
│       └── content_update.rs # Chat content update tools
│
├── studio/                 # Iris Studio TUI (Ratatui-based)
│   ├── app.rs             # Main event loop and app coordination
│   ├── state.rs           # Centralized state for all modes
│   ├── reducer.rs         # Pure state transitions and side effects
│   ├── events.rs          # Comprehensive event type definitions
│   ├── history.rs         # Complete audit trail and session persistence
│   ├── theme.rs           # SilkCircuit Neon color definitions
│   ├── components/        # Reusable UI components
│   │   ├── code_view.rs   # Syntax-highlighted source display
│   │   ├── diff_view.rs   # Unified diff rendering with hunks
│   │   ├── file_tree.rs   # Directory navigation with git status
│   │   └── message_editor.rs # Text editing with cursor management
│   ├── render/            # Mode-specific rendering
│   │   ├── commit.rs      # Commit mode panels
│   │   ├── explore.rs     # Explore mode panels
│   │   ├── review.rs      # Review mode panels
│   │   ├── pr.rs          # PR mode panels
│   │   ├── changelog.rs   # Changelog mode panels
│   │   ├── release_notes.rs # Release notes panels
│   │   ├── chat.rs        # Chat panel with markdown
│   │   └── modals.rs      # Settings, search, selectors
│   └── handlers/          # Input handling
│       ├── global.rs      # Cross-mode keybindings
│       ├── commit.rs      # Commit mode handlers
│       ├── explore.rs     # Explore mode handlers
│       └── ...            # Other mode handlers
│
├── types/                  # Response type definitions
│   ├── commit.rs          # GeneratedMessage
│   ├── pr.rs              # MarkdownPullRequest
│   ├── review.rs          # MarkdownReview
│   ├── changelog.rs       # MarkdownChangelog
│   └── release_notes.rs   # MarkdownReleaseNotes
│
├── services/               # Pure operations (no LLM)
│   └── git_commit.rs      # GitCommitService for git operations
│
├── cli.rs                 # CLI entry point
├── commands.rs            # Command handlers
├── providers.rs           # LLM provider configuration
├── config.rs              # Configuration management
├── git/                   # Git2 wrapper module
├── gitmoji.rs             # Emoji processing
└── output.rs              # Git output formatting
```

## Iris Studio Architecture

Studio is built on a **pure reducer pattern** for predictable state management:

```
┌─────────────────────────────────────────────────────────────┐
│                        Studio App                           │
├─────────────────────────────────────────────────────────────┤
│  Input Events (keyboard, mouse)                             │
│         │                                                   │
│         ▼                                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐ │
│  │   Handler   │ -> │   Reducer   │ -> │  Side Effects   │ │
│  │ (map input  │    │ (pure func) │    │ (spawn agent,   │ │
│  │  to event)  │    │             │    │  load data)     │ │
│  └─────────────┘    └─────────────┘    └─────────────────┘ │
│                            │                                │
│                            ▼                                │
│                     ┌─────────────┐                         │
│                     │    State    │                         │
│                     │  (updated)  │                         │
│                     └─────────────┘                         │
│                            │                                │
│                            ▼                                │
│                     ┌─────────────┐                         │
│                     │   Render    │                         │
│                     │ (to frame)  │                         │
│                     └─────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

**Events (`events.rs`):**

- `StudioEvent` enum captures all possible state transitions
- Events are dispatched from handlers and async tasks
- Clear, traceable data flow

**Reducer (`reducer.rs`):**

- Pure function: `(state, event) → (state, effects)`
- No I/O inside reducer—side effects returned as data
- Enables testing, replay, debugging

**Side Effects:**

- `SpawnAgent { task }` — Start async agent execution
- `LoadData { data_type, from_ref, to_ref }` — Load git data
- `UpdateContent { content, update_type }` — Update displayed content
- `StreamUpdate { tool_name, message }` — Show progress updates

### Studio Modes

| Mode              | Description                        | State Struct       |
| ----------------- | ---------------------------------- | ------------------ |
| **Explore**       | Navigate codebase with AI insights | `ExploreMode`      |
| **Commit**        | Generate/edit commit messages      | `CommitMode`       |
| **Review**        | AI-powered code reviews            | `ReviewMode`       |
| **PR**            | Pull request descriptions          | `PRMode`           |
| **Changelog**     | Structured changelog generation    | `ChangelogMode`    |
| **Release Notes** | Release documentation              | `ReleaseNotesMode` |

### Chat Integration

Press `/` in any mode to open chat with Iris:

```rust
// Chat state tracks conversation
pub struct ChatState {
    pub is_open: bool,
    pub input: String,
    pub messages: Vec<ChatMessage>,
    pub is_typing: bool,
    pub scroll_offset: u16,
}
```

Iris can update content directly through tools:

- `update_commit` — Modify commit message
- `update_pr` — Modify PR description
- `update_review` — Modify review content

## Agent Architecture

### Capabilities

Each capability is defined in `src/agents/capabilities/*.toml`:

| Capability      | Output Type            | Description                           |
| --------------- | ---------------------- | ------------------------------------- |
| `commit`        | `GeneratedMessage`     | Commit messages with emoji/title/body |
| `review`        | `MarkdownReview`       | Multi-dimensional code analysis       |
| `pr`            | `MarkdownPullRequest`  | Pull request descriptions             |
| `changelog`     | `MarkdownChangelog`    | Keep a Changelog format               |
| `release_notes` | `MarkdownReleaseNotes` | Release documentation                 |
| `chat`          | Varies                 | Interactive conversation              |

### Tools Available to Iris

| Tool                         | Purpose                                   |
| ---------------------------- | ----------------------------------------- |
| `git_diff(detail, from, to)` | Get changes with relevance scores         |
| `git_log(count)`             | Recent commit history for style reference |
| `git_status()`               | Repository status                         |
| `git_changed_files()`        | List of changed files                     |
| `file_analyzer()`            | Deep file analysis (content, metadata)    |
| `code_search()`              | Search for patterns, functions, classes   |
| `workspace()`                | Iris's notes and task tracking            |
| `project_docs(doc_type)`     | Read README, AGENTS.md, CLAUDE.md         |
| `parallel_analyze()`         | Concurrent subagent processing            |
| `update_commit()`            | Chat: update commit message               |
| `update_pr()`                | Chat: update PR description               |
| `update_review()`            | Chat: update review                       |

### Context Strategy

Iris adapts her approach based on changeset size:

| Scenario                           | Strategy                        |
| ---------------------------------- | ------------------------------- |
| Small (<10 files, <100 lines each) | Full context for all files      |
| Medium (10-20 files)               | Relevance scoring to prioritize |
| Large (20+ files)                  | Parallel subagent analysis      |

### Adding a New Capability

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
4. (Optional) Add Studio mode in `src/studio/state.rs`

## Output Types

Iris produces structured responses (all in `src/types/`):

| Type                   | Format   | Description                 |
| ---------------------- | -------- | --------------------------- |
| `GeneratedMessage`     | JSON     | `{ emoji, title, message }` |
| `MarkdownPullRequest`  | Markdown | `{ content: String }`       |
| `MarkdownReview`       | Markdown | `{ content: String }`       |
| `MarkdownChangelog`    | Markdown | `{ content: String }`       |
| `MarkdownReleaseNotes` | Markdown | `{ content: String }`       |

The `Markdown*` types use a simple wrapper, letting the LLM drive format while capability TOMLs provide guidelines.

## SilkCircuit Design Language

Git-Iris follows the **SilkCircuit Neon** color palette for a cohesive, electric aesthetic.

### Color Palette

| Color           | Hex       | RGB               | Usage                           |
| --------------- | --------- | ----------------- | ------------------------------- |
| Electric Purple | `#e135ff` | `(225, 53, 255)`  | Active modes, markers, emphasis |
| Neon Cyan       | `#80ffea` | `(128, 255, 234)` | Paths, interactions, focus      |
| Coral           | `#ff6ac1` | `(255, 106, 193)` | Hashes, numbers, constants      |
| Electric Yellow | `#f1fa8c` | `(241, 250, 140)` | Warnings, timestamps            |
| Success Green   | `#50fa7b` | `(80, 250, 123)`  | Success, confirmations          |
| Error Red       | `#ff6363` | `(255, 99, 99)`   | Errors, danger                  |

### Backgrounds

| Surface   | Hex       | Usage             |
| --------- | --------- | ----------------- |
| Base      | `#121218` | Main background   |
| Panel     | `#181820` | Individual panels |
| Highlight | `#2d283c` | Selections        |
| Code      | `#1e1e28` | Code blocks       |

### Implementation

```rust
use colored::Colorize;

// Success message
println!("{}", "✨ Commit created".truecolor(80, 250, 123));

// Error message
println!("{}", "Error: No staged changes".truecolor(255, 99, 99));

// Commit hash
println!("Commit: {}", hash.truecolor(255, 106, 193));
```

### Typography

- Monospace fonts: JetBrains Mono, Fira Code, SF Mono
- Unicode box-drawing: `─`, `━`, `│`, `┌`, `┐`, `└`, `┘`
- Braille spinners: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`

## Development Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test -- --nocapture    # With output

# Lint
cargo clippy
cargo clippy -- -W clippy::pedantic

# Run with debug
cargo run -- gen --debug     # Color-coded agent execution
cargo run -- studio          # Launch Studio TUI
RUST_LOG=debug cargo run -- gen  # Verbose logging

# Format
cargo fmt
```

## Provider Configuration

Set via environment or config:

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export GOOGLE_API_KEY=...
```

Or use CLI:

```bash
git-iris config --provider anthropic --api-key YOUR_KEY
git-iris config --provider anthropic --model claude-sonnet-4-5-20250929
```

### Provider Details

| Provider  | Default Model              | Fast Model                | Context |
| --------- | -------------------------- | ------------------------- | ------- |
| openai    | gpt-5.1                    | gpt-5.1-mini              | 128K    |
| anthropic | claude-sonnet-4-5-20250929 | claude-haiku-4-5-20251001 | 200K    |
| google    | gemini-3-pro-preview       | gemini-2.5-flash          | 1M      |

## Key Design Decisions

1. **LLM-First**: No hardcoded heuristics—Iris makes decisions
2. **Tool-Based Context**: Gather only what's needed via tool calls
3. **Pure Reducer**: State changes are predictable and testable
4. **Structured Output**: JSON schemas ensure parseable responses
5. **Output Validation**: Recovery logic handles malformed JSON
6. **Unified Interface**: Studio provides one TUI for all operations
7. **Event-Driven**: Clear data flow from input to state to render
