# 🔮 Git-Iris: Your Agentic Git Companion

<div align="center">

[![CI/CD](https://img.shields.io/github/actions/workflow/status/hyperb1iss/git-iris/cicd.yml?style=for-the-badge&logo=github-actions&logoColor=white&color=4C566A)](https://github.com/hyperb1iss/git-iris/actions)
[![Docker](https://img.shields.io/docker/pulls/hyperb1iss/git-iris?style=for-the-badge&logo=docker&logoColor=white&color=2496ED)](https://hub.docker.com/r/hyperb1iss/git-iris)
[![License](https://img.shields.io/badge/License-Apache%202.0-5E81AC?style=for-the-badge&logo=apache&logoColor=white&color=3B6EA8)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Release](https://img.shields.io/github/release/hyperb1iss/git-iris.svg?style=for-the-badge&logo=github&logoColor=white&color=9D6DB3)][releases]
[![Crates.io](https://img.shields.io/crates/v/git-iris.svg?style=for-the-badge&logo=rust&logoColor=white&color=D35D47)][crates]
[![GitHub Action](https://img.shields.io/badge/GitHub_Action-Release_Notes-5E81AC?style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/marketplace/actions/git-iris-release-notes)
[![Rust](https://img.shields.io/badge/rust-stable-EBCB8B?style=for-the-badge&logo=rust&logoColor=white&color=EFBB4D)](https://www.rust-lang.org/)
[![ko-fi](https://img.shields.io/badge/Ko--fi-Support%20Me-A3BE8C?style=for-the-badge&logo=ko-fi&logoColor=white&color=82B062)](https://ko-fi.com/hyperb1iss)

✨ _An intelligent agent that understands your code and crafts perfect Git artifacts_ ✨

📖 [Documentation](docs/) • [Installation](#-installation) • [Studio TUI](#-iris-studio) • [Commands](#-usage) • [Configuration](#%EF%B8%8F-configuration) • [Contributing](#-contributing) • [License](#-license)

</div>

<div align="center">
  <img src="https://raw.githubusercontent.com/hyperb1iss/git-iris/main/docs/images/git-iris-screenshot-1.png" alt="Git-Iris Screenshot 1" width="33%">
  <img src="https://raw.githubusercontent.com/hyperb1iss/git-iris/main/docs/images/git-iris-screenshot-2.png" alt="Git-Iris Screenshot 2" width="33%">
  <img src="https://raw.githubusercontent.com/hyperb1iss/git-iris/main/docs/images/git-iris-screenshot-3.png" alt="Git-Iris Screenshot 3" width="33%">
</div>

_Iris exploring your codebase and generating contextual commit messages_

## 💜 Overview

Git-Iris is powered by **Iris**, an intelligent agent that actively explores your codebase to understand what you're building. Rather than dumping context and hoping for the best, Iris uses tools to gather precisely the information she needs—analyzing diffs, exploring file relationships, and building understanding iteratively.

This agent-first architecture means Iris adapts to your project. She crafts meaningful commit messages, generates insightful changelogs, creates detailed release notes, and provides thorough code reviews—all informed by genuine understanding of your code.

## ⚡ Features

### 🌌 Iris Studio — The Unified TUI

**Studio** is a stunning, context-aware terminal interface that brings all of Git-Iris's capabilities together in one beautiful experience. Built with the **SilkCircuit Neon** design language, it adapts to your workflow with six specialized modes:

| Mode                 | Description                                                                 | Shortcut  |
| -------------------- | --------------------------------------------------------------------------- | --------- |
| 🔭 **Explore**       | Navigate your codebase with AI-powered semantic insights and blame analysis | `Shift+E` |
| 💫 **Commit**        | Generate and refine commit messages for your staged changes                 | `Shift+C` |
| 🔬 **Review**        | Get comprehensive AI-powered code reviews with severity ratings             | `Shift+R` |
| 📜 **PR**            | Generate complete pull request descriptions                                 | `Shift+P` |
| 🗂️ **Changelog**     | Create structured changelogs in Keep a Changelog format                     | `Shift+L` |
| 🎊 **Release Notes** | Generate detailed release documentation                                     | `Shift+N` |

**💬 Chat with Iris** — Press `/` in any mode to open an interactive conversation with Iris. Ask her to refine your commit message, explain changes, or improve the generated content. She can update content directly through intelligent tool calls!

### 🧿 Agentic Intelligence

- **Context-Aware Understanding**: Iris explores your codebase using tools, not templates—she understands what changed and why it matters
- **Adaptive Analysis**: For large changesets, Iris spawns subagents to analyze different parts concurrently, then synthesizes findings
- **Intelligent Focus**: Relevance scoring helps Iris prioritize what matters, ignoring noise in large diffs
- **Iterative Exploration**: Iris can dig deeper into specific files, search for patterns, and examine commit history when needed
- **Real-Time Streaming**: Watch Iris think in real-time with buttery-smooth token streaming as she analyzes your code
- **Token Usage Tracking**: See exactly how many tokens each operation consumes

### 🪄 Core Capabilities

- **✍️ Commit Messages**: Contextual messages that capture the essence of your changes
- **🔬 Code Reviews**: Thorough reviews that identify security issues, performance problems, and architectural concerns
- **🗂️ Changelogs**: Structured changelogs following Keep a Changelog format
- **🎊 Release Notes**: Comprehensive release notes with highlights and categorized changes
- **📜 PR Descriptions**: Complete pull request descriptions for single commits or branch comparisons

### 💎 Developer Experience

- **🤖 Multi-Provider Support**: Works with OpenAI, Anthropic, and Google
- **🌐 Remote Repository Support**: Work with remote repositories without manual cloning
- **🎨 Customizable Presets**: Style presets from Conventional Commits to Cosmic Oracle
- **✨ Gitmoji Integration**: Expressive emojis for your Git documentation
- **🐳 Docker Support**: Run in CI/CD pipelines without installation
- **🔮 Debug Mode**: Gorgeous color-coded output showing agent execution details
- **🖱️ Full Mouse Support**: Click, scroll, and navigate with your mouse in Studio
- **📋 Clipboard Integration**: Copy any generated content with `y` and get visual feedback
- **🔍 File Search Modal**: Quickly find and jump to any file in your repository

## 📦 Installation

### Prerequisites

- Rust and Cargo (latest stable version)
- Git 2.23.0 or newer

### Via Cargo (Recommended)

```bash
cargo install git-iris
```

### Via Docker

Git-Iris is available as a Docker image:

```bash
docker pull hyperb1iss/git-iris:latest
```

Run it:

```bash
docker run --rm -v "$(pwd):/git-repo" hyperb1iss/git-iris gen
```

For detailed instructions, examples, and CI/CD integration, see our [Docker Usage Guide](docker/README.md).

### Manual Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/hyperb1iss/git-iris.git
   cd git-iris
   ```

2. Build and install:
   ```bash
   cargo build --release
   cargo install --path .
   ```

## 🐳 Docker

Git-Iris provides official Docker images for easy integration into your CI/CD pipelines and workflows without installation:

```bash
# Generate a commit message (mount current directory)
docker run --rm --user $(id -u):$(id -g) -v "$(pwd):/git-repo" hyperb1iss/git-iris gen

# Configure with environment variables
docker run --rm --user $(id -u):$(id -g) -v "$(pwd):/git-repo" \
  -e GITIRIS_PROVIDER="openai" \
  -e GITIRIS_API_KEY="your-api-key" \
  hyperb1iss/git-iris gen
```

For persistent configuration, mount a volume to store your settings:

```bash
docker run --rm --user $(id -u):$(id -g) -v "$(pwd):/git-repo" \
  -v git-iris-config:/root/.config/git-iris \
  hyperb1iss/git-iris config --provider openai --api-key your-api-key
```

The Docker image is particularly useful in CI/CD workflows:

```yaml
# GitHub Actions example
- name: Generate Release Notes
  env:
    GITIRIS_PROVIDER: openai
    GITIRIS_API_KEY: ${{ secrets.OPENAI_API_KEY }}
  run: |
    docker run --rm -v "$(pwd):/git-repo" \
      -e GITIRIS_PROVIDER -e GITIRIS_API_KEY \
      hyperb1iss/git-iris release-notes \
      --from $(git describe --tags --abbrev=0 $(git rev-list --tags --skip=1 --max-count=1)) \
      --to $(git describe --tags --abbrev=0) \
      --print > RELEASE_NOTES.md
```

To build and test the Docker image locally:

```bash
# Build the image with a custom tag
./docker/build.sh mytag

# Test the image
./docker/test-image.sh mytag
```

For detailed instructions, examples, and CI/CD integration, see our [Docker Usage Guide](docker/README.md).

## 🤖 GitHub Action

Git-Iris is available as a GitHub Action for automated release note generation in your workflows:

```yaml
- name: Generate release notes
  uses: hyperb1iss/git-iris@v1
  id: release_notes
  with:
    from: v1.0.0
    to: v1.1.0
    provider: openai
    api-key: ${{ secrets.OPENAI_API_KEY }}
    output-file: RELEASE_NOTES.md

- name: Create Release
  uses: softprops/action-gh-release@v2
  with:
    body: ${{ steps.release_notes.outputs.release-notes }}
```

### Action Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `from` | Starting Git reference (tag, commit, branch) | Yes | - |
| `to` | Ending Git reference | No | `HEAD` |
| `provider` | LLM provider (`openai`, `anthropic`, `google`) | No | `openai` |
| `model` | Model to use (provider-specific) | No | Provider default |
| `api-key` | API key for the LLM provider | Yes | - |
| `output-file` | File path to write release notes | No | - |
| `version-name` | Explicit version name for release notes | No | - |
| `custom-instructions` | Custom instructions for generation | No | - |
| `version` | Git-Iris version to use | No | `latest` |

### Action Outputs

| Output | Description |
|--------|-------------|
| `release-notes` | Generated release notes content |
| `release-notes-file` | Path to output file (if specified) |

### Supported Platforms

The action downloads prebuilt binaries for fast execution:
- Linux x64 (`ubuntu-latest`)
- Linux ARM64 (`ubuntu-24.04-arm`)
- macOS ARM64 (`macos-latest`, runs on x64 via Rosetta)
- Windows x64 (`windows-latest`)

## ⚙️ Configuration

Git-Iris offers both global configuration and project-specific configuration options.

### Global Configuration

Global settings are stored in `~/.config/git-iris/config.toml` and apply across all repositories:

```bash
# For OpenAI
git-iris config --provider openai --api-key YOUR_OPENAI_API_KEY

# For Anthropic
git-iris config --provider anthropic --api-key YOUR_ANTHROPIC_API_KEY

# For Google
git-iris config --provider google --api-key YOUR_GOOGLE_API_KEY
```

### Project-Specific Configuration

Project settings are stored in `.irisconfig` in your repository root and can be shared with your team without sharing sensitive credentials:

```bash
# Set project-specific LLM provider
git-iris project-config --provider anthropic

# Configure project-specific preset
git-iris project-config --preset conventional

# Set model for the project
git-iris project-config --model claude-sonnet-4-5-20250929

# View current project configuration
git-iris project-config --print
```

Project configuration files do not store API keys for security reasons but can store other settings like models, presets, and custom instructions.

### Supported LLM Providers

Git-Iris supports the following LLM providers:

| Provider  | Default Model              | Context Window | API Key Required |
| --------- | -------------------------- | -------------- | ---------------- |
| openai    | gpt-5.1                    | 128,000        | Yes              |
| anthropic | claude-sonnet-4-5-20250929 | 200,000        | Yes              |
| google    | gemini-3-pro-preview       | 1,000,000      | Yes              |

Each provider also has a fast model configured for simple tasks (status updates, parsing). You can override models via configuration.

Additional configuration options:

```bash
# Enable/Disable Gitmoji
git-iris config --gitmoji true

# Set custom instructions
git-iris config --instructions "Always mention the ticket number in the commit message"

# Set default instruction preset
git-iris config --preset conventional

# Set token limit for a provider
git-iris config --provider openai --token-limit 4000

# Set model for a provider
git-iris config --provider openai --model gpt-4o

# Set additional parameters for a provider
git-iris config --provider openai --param temperature=0.7 --param max_tokens=150
```

For more detailed configuration information, please refer to our [Configuration Guide](CONFIG.md).

## 🌌 Iris Studio

**Iris Studio** is the crown jewel of Git-Iris—a unified terminal interface that brings all capabilities together in one beautiful, intuitive experience.

### ⚡ Launching Studio

```bash
git-iris              # Launch Studio (default command)
git-iris studio       # Explicit studio command
git-iris studio --mode commit    # Start in a specific mode
```

### ⌨️ Keyboard Navigation

**Global Controls:**

| Key                 | Action                            |
| ------------------- | --------------------------------- |
| `Tab` / `Shift+Tab` | Navigate between panels           |
| `/`                 | Open chat with Iris               |
| `?`                 | Show help overlay                 |
| `Shift+S`           | Open settings modal               |
| `Ctrl+F`            | Open file search modal            |
| `y`                 | Copy current content to clipboard |
| `Esc`               | Close modal / Cancel              |
| `q`                 | Quit Studio                       |

**Mode Switching:**

| Key       | Mode                  |
| --------- | --------------------- |
| `Shift+E` | 🔭 Explore Mode       |
| `Shift+C` | 💫 Commit Mode        |
| `Shift+R` | 🔬 Review Mode        |
| `Shift+P` | 📜 PR Mode            |
| `Shift+L` | 🗂️ Changelog Mode     |
| `Shift+N` | 🎊 Release Notes Mode |

**Explore Mode Controls:**

| Key                    | Action                                                    |
| ---------------------- | --------------------------------------------------------- |
| `Enter`                | Open file / Expand directory                              |
| `w`                    | **Semantic Blame** — Ask Iris "Why does this code exist?" |
| `j` / `k` or `↓` / `↑` | Navigate files                                            |
| `h` / `l` or `←` / `→` | Collapse / Expand directories                             |

**Commit Mode Controls:**

| Key                    | Action                           |
| ---------------------- | -------------------------------- |
| `g`                    | Generate commit message          |
| `c`                    | Commit changes                   |
| `s`                    | Toggle stage/unstage file        |
| `p`                    | Open preset selector             |
| `m`                    | Open emoji selector              |
| `j` / `k` or `↓` / `↑` | Navigate files                   |
| `[` / `]`              | Cycle through generated messages |
| `e`                    | Edit current message             |

**Reference Selection (Review, PR, Changelog, Release Notes):**

| Key | Action                  |
| --- | ----------------------- |
| `f` | Select "from" reference |
| `t` | Select "to" reference   |
| `g` | Generate content        |

### 🔭 Explore Mode & Semantic Blame

Explore mode lets you navigate your codebase with AI-powered insights. The standout feature is **Semantic Blame**:

1. Navigate to any file and position your cursor on a line of code
2. Press `w` to ask Iris "Why does this code exist?"
3. Iris analyzes the git blame history and explains the code's purpose—not just _what_ changed, but _why_

This transforms git blame from a dry history lookup into genuine understanding of your codebase's evolution.

### 💬 Chatting with Iris

Press `/` in any mode to open the chat panel. Iris can:

- **Refine content**: "Make the commit title shorter" or "Add more detail to the description"
- **Explain changes**: "What does this refactor accomplish?"
- **Update directly**: Iris can modify the generated content through tool calls
- **Answer questions**: Ask about the codebase, the changes, or Git in general

**Real-Time Streaming**: Watch Iris's responses appear token-by-token with live tool activity tracking. See which tools she's using as she analyzes your code.

The chat supports full markdown rendering with syntax highlighting!

### 🖱️ Mouse Support

Studio has full mouse support:

- **Click** on panels to focus them
- **Click** on files in the file tree to select them
- **Scroll** within any panel to navigate content
- **Click** buttons and interactive elements

### 🔧 In-App Modals

Studio provides several powerful modals for quick access:

| Modal                  | Shortcut          | Description                                                            |
| ---------------------- | ----------------- | ---------------------------------------------------------------------- |
| **Settings**           | `Shift+S`         | Configure provider, model, gitmoji, and presets without leaving Studio |
| **File Search**        | `Ctrl+F`          | Fuzzy-find any file in your repository and jump directly to it         |
| **Preset Selector**    | `p` (Commit mode) | Choose from style presets like Conventional, Detailed, or Cosmic       |
| **Emoji Selector**     | `m` (Commit mode) | Browse and select gitmoji for your commit message                      |
| **Reference Selector** | `f` / `t`         | Pick git references (branches, tags, commits) for comparisons          |

### 🎨 SilkCircuit Neon Design

Studio follows the **SilkCircuit Neon** color palette for a cohesive, electric aesthetic:

- **Electric Purple** `#e135ff` — Active modes, markers, emphasis
- **Neon Cyan** `#80ffea` — Paths, interactions, focus states
- **Coral** `#ff6ac1` — Commit hashes, numbers, constants
- **Electric Yellow** `#f1fa8c` — Warnings, timestamps
- **Success Green** `#50fa7b` — Confirmations, success states
- **Error Red** `#ff6363` — Errors, danger indicators

## 🎯 Usage

### Global Options

These options apply to all commands:

- `-l`, `--log`: Log debug messages to a file
- `--log-file`: Specify a custom log file path
- `-q`, `--quiet`: Suppress non-essential output (spinners, waiting messages, etc.)
- `-v`, `--version`: Display version information
- `-r`, `--repo`: Use a remote repository URL instead of local repository
- `--debug`: Enable debug mode with gorgeous color-coded output showing agent execution details

### 💫 Generate Commit Messages

Generate a commit message with Iris:

```bash
git-iris gen
```

Options:

- `-a`, `--auto-commit`: Automatically commit with the generated message
- `-i`, `--instructions`: Provide custom instructions for this commit
- `--provider`: Specify an LLM provider
- `--preset`: Use a specific instruction preset
- `--no-gitmoji`: Disable Gitmoji for this commit
- `-p`, `--print`: Print the generated message to stdout and exit
- `--no-verify`: Skip verification steps (pre/post commit hooks)

Example:

```bash
git-iris gen -a -i "Focus on performance improvements" --provider anthropic --preset detailed
```

To generate a commit message and print it to stdout without starting the interactive process:

```bash
git-iris gen --print
```

### 🔬 Code Reviews

Have Iris review your staged changes:

```bash
git-iris review
```

Options:

- `-i`, `--instructions`: Provide custom instructions for this review
- `--provider`: Specify an LLM provider
- `--preset`: Use a specific instruction preset
- `-p`, `--print`: Print the generated review to stdout and exit
- `--raw`: Output raw markdown without console formatting (for piping to files)
- `--include-unstaged`: Include unstaged changes in the review
- `--commit`: Review a specific commit by ID (hash, branch, or reference)
- `--from`: Starting branch for comparison (defaults to 'main')
- `--to`: Target branch for comparison

Example:

```bash
git-iris review -i "Focus on security" --preset security --include-unstaged
```

For more comprehensive reviews, you can also:

- **Review unstaged changes**: Include unstaged changes in the review

  ```bash
  git-iris review --include-unstaged
  ```

- **Review a specific commit**: Analyze a particular commit

  ```bash
  git-iris review --commit abc123
  ```

- **Review branch differences**: Compare entire branches (perfect for PR reviews)
  ```bash
  git-iris review --from main --to feature-branch
  ```

The branch comparison feature is particularly powerful for reviewing pull requests, as it analyzes all changes between two branches, giving you a comprehensive view of the entire feature or fix.

**Smart Branch Comparison**: Git-Iris uses merge-base comparison to ensure you only see changes relevant to the feature branch, not unrelated changes that happened in the base branch after the feature branch was created. This provides accurate PR reviews even when the base branch has moved forward.

Example usage:

```bash
# Compare feature branch to main (explicit)
git-iris review --from main --to feature-branch

# Compare feature branch to main (using default)
git-iris review --to feature-branch  # --from defaults to 'main'
```

Iris analyzes your code across multiple quality dimensions, focusing on what matters most for each specific changeset:

- **🛡️ Security**: Vulnerabilities, auth issues, insecure patterns
- **⚡ Performance**: Inefficient algorithms, resource leaks, blocking operations
- **🚨 Error Handling**: Missing try-catch, swallowed exceptions, unclear errors
- **🌀 Complexity**: Overly complex logic, deep nesting, god functions
- **🏛️ Abstraction**: Poor design patterns, leaky abstractions
- **🔄 Duplication**: Copy-pasted code, repeated logic
- **🧪 Testing**: Coverage gaps, brittle tests
- **🎨 Style**: Inconsistencies, naming, formatting
- **💡 Best Practices**: Anti-patterns, deprecated APIs

Each issue includes severity level (CRITICAL, HIGH, MEDIUM, LOW), exact file:line location, explanation, and a concrete fix recommendation.

### 📜 Generate Pull Request Descriptions

Create comprehensive PR descriptions for changesets spanning multiple commits or single commits:

```bash
git-iris pr --from <from-ref> --to <to-ref>
```

Options:

- `--from`: Starting Git reference (commit hash, tag, or branch name)
- `--to`: Ending Git reference (defaults to HEAD if not specified)
- `-i`, `--instructions`: Custom instructions for PR description generation
- `--preset`: Select an instruction preset for PR description generation
- `-p`, `--print`: Print the generated PR description to stdout and exit
- `--raw`: Output raw markdown without console formatting

**Examples:**

**Single commit analysis:**

```bash
# Analyze a single commit (compares against its parent)
git-iris pr --from abc1234
git-iris pr --to abc1234

# Analyze a specific commitish (e.g., 2 commits ago)
git-iris pr --to HEAD~2

# Same commit for both from and to
git-iris pr --from abc1234 --to abc1234
```

**Multiple commit analysis:**

```bash
# Review the last 3 commits together
git-iris pr --from HEAD~3

# Review commits from a specific point to now
git-iris pr --from @~5
```

**Commit range analysis:**

```bash
git-iris pr --from v1.0.0 --to HEAD --preset detailed
```

**Branch comparison:**

```bash
git-iris pr --from main --to feature-auth --preset conventional
```

**Comparison to main:**

```bash
# Compare feature branch to main
git-iris pr --to feature-branch
```

The PR description generator analyzes the entire changeset as an atomic unit rather than individual commits, providing:

- A comprehensive title and summary
- Detailed description of what was changed and why
- List of commits included in the PR
- Breaking changes identification
- Testing notes and deployment considerations
- Technical implementation details

This is particularly useful for:

- **Single commits**: Get detailed analysis of what changed in a specific commit or commitish (e.g., `HEAD~2`)
- **Multiple commits**: Review a range of commits together (e.g., `--from HEAD~3` reviews the last 3 commits)
- **Feature branches**: Get a complete overview of all changes in a feature
- **Release preparations**: Understand what's included in a release candidate
- **Code reviews**: Provide reviewers with comprehensive context
- **Documentation**: Create detailed records of what changed between versions

**Supported commitish syntax**: `HEAD~2`, `HEAD^`, `@~3`, `main~1`, `origin/main^`, and other Git commitish references.

### 🗂️ Generate Changelogs

Create a detailed changelog between Git references:

```bash
git-iris changelog --from <from-ref> --to <to-ref>
```

Options:

- `--from`: Starting Git reference (commit hash, tag, or branch name)
- `--to`: Ending Git reference (defaults to HEAD if not specified)
- `-i`, `--instructions`: Custom instructions for changelog generation
- `--preset`: Select an instruction preset for changelog generation
- `--detail-level`: Set the detail level (minimal, standard, detailed)
- `--gitmoji`: Enable or disable Gitmoji in the changelog
- `--raw`: Output raw markdown without console formatting
- `--update`: Update the changelog file with the new changes
- `--file`: Path to the changelog file (defaults to CHANGELOG.md)
- `--version-name`: Explicit version name to use in the changelog instead of getting it from Git

Example:

```bash
git-iris changelog --from v1.0.0 --to v1.1.0 --detail-level detailed --update
```

Example using explicit version name (useful in release workflows):

```bash
git-iris changelog --from v1.0.0 --to HEAD --update --version-name v1.1.0
```

### 🎊 Generate Release Notes

Create comprehensive release notes:

```bash
git-iris release-notes --from <from-ref> --to <to-ref>
```

Options:

- `--from`: Starting Git reference (commit hash, tag, or branch name)
- `--to`: Ending Git reference (defaults to HEAD if not specified)
- `-i`, `--instructions`: Custom instructions for release notes generation
- `--preset`: Select an instruction preset for release notes generation
- `--detail-level`: Set the detail level (minimal, standard, detailed)
- `--gitmoji`: Enable or disable Gitmoji in the release notes
- `--raw`: Output raw markdown without console formatting
- `--version-name`: Explicit version name to use in the release notes instead of getting it from Git

Example:

```bash
git-iris release-notes --from v1.0.0 --to v1.1.0 --preset conventional
```

Example using explicit version name:

```bash
git-iris release-notes --from v1.0.0 --to HEAD --version-name v1.1.0
```

### 🔧 Project Configuration

Create or update project-specific settings:

```bash
git-iris project-config
```

Options:

- `--provider`: Set default LLM provider for this project
- `--model`: Set model for the specified provider
- `--token-limit`: Set token limit for the specified provider
- `--param`: Set additional parameters for the specified provider
- `-p`, `--print`: Print the current project configuration
- `--gitmoji`: Enable or disable Gitmoji for this project
- `-i`, `--instructions`: Set instructions for message generation
- `--preset`: Set default instruction preset for this project

Example:

```bash
git-iris project-config --provider anthropic --preset security --model claude-sonnet-4-5-20250929
```

### 🌍 Working with Remote Repositories

Git-Iris supports working with remote repositories directly without having to clone them manually:

```bash
# Generate a changelog between two tags on a remote repository
git-iris changelog --repo https://github.com/example/repo.git --from v1.0.0 --to v2.0.0

# Generate release notes for a remote repository
git-iris release-notes --repo https://github.com/example/repo.git --from v1.0.0 --to v2.0.0

# Review code in a remote repository (read-only)
git-iris review --repo https://github.com/example/repo.git

# Generate a commit message for a remote repository (read-only)
git-iris gen --repo https://github.com/example/repo.git --print
```

Note: When working with remote repositories, Git-Iris operates in read-only mode. You can't commit changes directly to remote repositories.

## 🎭 Custom Instructions and Presets

Git-Iris offers two powerful ways to guide the AI in generating commit messages: custom instructions and presets.

### Instruction Presets

Presets are predefined sets of instructions that provide a quick way to adjust the commit message style. Git-Iris comes with several built-in presets to suit different commit styles and project needs.

To list available presets:

```bash
git-iris list-presets
```

This will display a list of all available presets with a brief description of each, categorized by type (general, review-specific, or commit-specific).

Some key presets include:

- `default`: Standard professional style (for both commits and reviews)
- `conventional`: Follows the Conventional Commits specification
- `detailed`: Provides more context and explanation
- `concise`: Short and to-the-point responses
- `cosmic`: Mystical, space-themed language ✨

For code reviews specifically, Git-Iris includes specialized presets:

- `security`: Focus on security vulnerabilities and best practices
- `performance`: Analyze code for performance optimizations
- `architecture`: Evaluate architectural patterns and design decisions
- `testing`: Focus on test coverage and testing strategies
- `maintainability`: Evaluate code for long-term maintenance
- `conventions`: Check adherence to language and project coding standards

To use a preset for a single commit:

```bash
git-iris gen --preset conventional
```

To use a preset for a code review:

```bash
git-iris review --preset security
```

To set a default preset:

```bash
git-iris config --preset conventional
```

Presets work seamlessly with other Git-Iris features. For example, if you have Gitmoji enabled, the preset instructions will be applied in addition to adding the appropriate Gitmoji.

### Custom Instructions

Custom instructions allow you to provide specific guidance for commit message generation. These can be set globally or per-commit.

Setting global custom instructions:

```bash
git-iris config --instructions "Always include the ticket number and mention performance impacts"
```

Providing per-commit instructions:

```bash
git-iris gen -i "Emphasize security implications of this change"
```

### Combining Presets and Custom Instructions

When using both a preset and custom instructions, Git-Iris combines them, with custom instructions taking precedence. This allows you to use a preset as a base and fine-tune it with specific instructions.

```bash
git-iris gen --preset conventional -i "Mention the JIRA ticket number"
```

In this case, the commit message will follow the Conventional Commits format and include the JIRA ticket number.

If you've set a default preset in your configuration, you can still override it for individual commits:

```bash
git-iris gen --preset detailed -i "Focus on performance improvements"
```

This will use the 'detailed' preset instead of your default, along with the custom instruction.

### Examples of Custom Instructions

1. **Ticket Number Integration**

   ```
   Always start the commit message with the JIRA ticket number in square brackets
   ```

2. **Language-Specific Conventions**

   ```
   For Rust files, mention any changes to public APIs or use of unsafe code
   ```

3. **Team-Specific Guidelines**

   ```
   Follow the Angular commit message format: <type>(<scope>): <subject>
   ```

4. **Project-Specific Context**

   ```
   For the authentication module, always mention if there are changes to the user model or permissions
   ```

5. **Performance Considerations**
   ```
   Highlight any changes that might affect application performance, including database queries
   ```

Custom instructions and presets allow you to tailor Git-Iris to your specific project needs, team conventions, or personal preferences. They provide a powerful way to ensure consistency and capture important context in your commit messages.

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to get started, our code of conduct, and the process for submitting pull requests.

## ⚖️ License

Distributed under the Apache 2.0 License. See `LICENSE` for more information.

---

<div align="center">

📚 [Documentation](docs/) • 🐛 [Report Bug](https://github.com/hyperb1iss/git-iris/issues) • 💡 [Request Feature](https://github.com/hyperb1iss/git-iris/issues)

</div>

<div align="center">

Created by [Stefanie Jane 🌠](https://github.com/hyperb1iss)

If you find Git-Iris useful, [buy me a Monster Ultra Violet](https://ko-fi.com/hyperb1iss)! ⚡️

</div>

[crates-shield]: https://img.shields.io/crates/v/git-iris.svg?style=for-the-badge&logo=rust&logoColor=white&color=D35D47
[crates]: https://crates.io/crates/git-iris
[releases-shield]: https://img.shields.io/github/release/hyperb1iss/git-iris.svg?style=for-the-badge&logo=github&logoColor=white&color=9D6DB3
[releases]: https://github.com/hyperb1iss/git-iris/releases
[license-shield]: https://img.shields.io/github/license/hyperb1iss/git-iris.svg?style=for-the-badge&logo=apache&logoColor=white&color=3B6EA8
