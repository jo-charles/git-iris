# Git-Iris Usage Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Basic Usage](#basic-usage)
   - [Generating Commit Messages](#generating-commit-messages)
   - [Code Reviews](#code-reviews)
   - [Changelogs](#changelogs)
   - [Release Notes](#release-notes)
5. [Advanced Features](#advanced-features)
   - [Custom Instructions](#custom-instructions)
   - [Instruction Presets](#instruction-presets)
   - [TUI Navigation](#tui-navigation)
   - [Multiple LLM Providers](#multiple-llm-providers)
   - [Token Optimization](#token-optimization)
   - [Project Configuration](#project-configuration)
6. [MCP Integration](#mcp-integration)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)
9. [FAQ](#faq)

## 1. Introduction <a name="introduction"></a>

Git-Iris is an AI-powered tool designed to enhance your Git workflow. It generates meaningful commit messages, provides code reviews, creates changelogs, and produces release notes by analyzing your code changes and project context.

## 2. Installation <a name="installation"></a>

### Prerequisites

- Rust and Cargo (latest stable version)
- Git 2.23.0 or newer

### Via Cargo (Recommended)

```bash
cargo install git-iris
```

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

## 3. Configuration <a name="configuration"></a>

Git-Iris uses a configuration file located at `~/.config/git-iris/config.toml`. You can set it up using the following commands:

```bash
# Set up OpenAI as the provider
git-iris config --provider openai --api-key YOUR_OPENAI_API_KEY

# Set up Anthropic as the provider
git-iris config --provider anthropic --api-key YOUR_ANTHROPIC_API_KEY
# Note: "claude" is still supported for backward compatibility

# Set up Google as the provider
git-iris config --provider google --api-key YOUR_GOOGLE_API_KEY

# Set up Ollama as the provider (no API key required)
git-iris config --provider ollama

# For other supported providers (Groq, XAI, DeepSeek, Phind)
git-iris config --provider <provider> --api-key YOUR_API_KEY

# Enable Gitmoji
git-iris config --gitmoji true

# Set custom instructions
git-iris config --instructions "Always mention the ticket number in the commit message"
```

### Supported LLM Providers

Git-Iris uses the [llm crate](https://crates.io/crates/llm) to support multiple LLM providers:

| Provider  | Default Model              | Token Limit | API Key Required |
| --------- | -------------------------- | ----------- | ---------------- |
| anthropic | claude-3-7-sonnet-20250219 | 200,000     | Yes              |
| deepseek  | deepseek-chat              | 64,000      | Yes              |
| google    | gemini-2.0-flash           | 1,000,000   | Yes              |
| groq      | llama-3.1-70b-versatile    | 128,000     | Yes              |
| ollama    | llama3                     | 128,000     | No               |
| openai    | gpt-4o                     | 128,000     | Yes              |
| phind     | phind-v2                   | 32,000      | No               |
| xai       | grok-2-beta                | 128,000     | Yes              |

### Project-Specific Configuration

Git-Iris supports project-specific configuration files that override global settings:

```bash
# Create or update a project-specific configuration
git-iris project-config --instructions "Include JIRA ticket numbers from branch names"

# Set a different model for the project
git-iris project-config --model "gpt-4o-mini"

# Print current project configuration
git-iris project-config --print
```

The project configuration is stored in `.irisconfig` in the repository root and is automatically used when running Git-Iris in that repository.

## 4. Basic Usage <a name="basic-usage"></a>

### Generating a Commit Message <a name="generating-commit-messages"></a>

1. Stage your changes using `git add`
2. Run the following command:
   ```bash
   git-iris gen
   ```
3. Review the generated message in the interactive TUI
4. Accept, edit, or regenerate the message as needed
5. Confirm to create the commit

#### Command-line Options for Commit Generation

- `--auto-commit`: Automatically commit with the generated message
- `--no-gitmoji`: Disable Gitmoji for this commit
- `--print`: Print the message to stdout without committing
- `--no-verify`: Skip pre-commit hooks
- `--provider`: Specify an LLM provider
- `--preset`: Use a specific instruction preset
- `--instructions`: Provide custom instructions

Example:

```bash
git-iris gen --preset conventional --provider anthropic
```

### Getting an AI Code Review <a name="code-reviews"></a>

1. Stage your changes using `git add`
2. Run the following command:
   ```bash
   git-iris review
   ```
3. Wait for the AI to analyze your changes
4. Review the feedback, which includes:
   - A summary of your changes
   - Code quality assessment
   - Suggestions for improvement
   - Identified issues across 11 quality dimensions
   - Positive aspects of your code

#### Command-line Options for Code Review

- `--instructions`: Provide custom instructions for the review
- `--provider`: Specify an LLM provider
- `--preset`: Use a specific instruction preset
- `--print`: Print the review to stdout
- `--include-unstaged`: Include unstaged changes in the review
- `--commit`: Review a specific commit by ID (hash, branch, or reference)

Example:

```bash
git-iris review --preset security --include-unstaged
```

### Generating a Changelog <a name="changelogs"></a>

Generate a structured changelog between two Git references:

```bash
git-iris changelog --from v1.0.0 --to v1.1.0
```

#### Command-line Options for Changelog Generation

- `--from`: Starting Git reference (required)
- `--to`: Ending Git reference (defaults to HEAD)
- `--provider`: Specify an LLM provider
- `--instructions`: Custom instructions for changelog generation
- `--preset`: Use a specific instruction preset
- `--detail-level`: Set detail level (minimal, standard, detailed)
- `--update`: Update the changelog file with the new changes
- `--file`: Path to the changelog file (default: CHANGELOG.md)

Example:

```bash
git-iris changelog --from v1.0.0 --detail-level detailed --update
```

### Creating Release Notes <a name="release-notes"></a>

Generate comprehensive release notes between two Git references:

```bash
git-iris release-notes --from v1.0.0 --to v1.1.0
```

#### Command-line Options for Release Notes

- `--from`: Starting Git reference (required)
- `--to`: Ending Git reference (defaults to HEAD)
- `--provider`: Specify an LLM provider
- `--instructions`: Custom instructions for release notes generation
- `--preset`: Use a specific instruction preset
- `--detail-level`: Set detail level (minimal, standard, detailed)

Example:

```bash
git-iris release-notes --from v1.0.0 --preset marketing
```

## 5. Advanced Features <a name="advanced-features"></a>

### Custom Instructions <a name="custom-instructions"></a>

You can provide custom instructions to guide the AI in generating content:

```bash
git-iris gen --instructions "Focus on performance improvements and API changes"
git-iris review --instructions "Look for potential security vulnerabilities"
git-iris changelog --from v1.0.0 --instructions "Focus on user-facing changes"
```

You can also set default instructions in your configuration:

```bash
git-iris config --instructions "Always include the scope of changes"
```

### Instruction Presets <a name="instruction-presets"></a>

Git-Iris includes built-in instruction presets for different use cases:

```bash
# List available presets
git-iris list-presets

# Use a preset for commit message generation
git-iris gen --preset conventional

# Use a preset for code review
git-iris review --preset security

# Use a preset for changelog generation
git-iris changelog --from v1.0.0 --preset detailed
```

### TUI Navigation <a name="tui-navigation"></a>

The Text User Interface (TUI) for commit message generation supports the following keyboard shortcuts:

- Use `↑`/`↓` arrows to navigate through suggestions
- Press `e` to edit the current message
- Press `i` to modify AI instructions
- Press `u` to edit user information
- Press `p` to select an instruction preset
- Press `g` to select an emoji (when Gitmoji is enabled)
- Press `r` to regenerate the message
- Press `Enter` to commit
- Press `Esc` to cancel

### Multiple LLM Providers <a name="multiple-llm-providers"></a>

Git-Iris supports multiple LLM providers through the [llm crate](https://crates.io/crates/llm). You can switch between them for different tasks:

```bash
git-iris gen --provider anthropic
git-iris review --provider openai
git-iris changelog --from v1.0.0 --provider ollama
```

### Token Optimization <a name="token-optimization"></a>

Git-Iris automatically optimizes token usage to stay within provider limits while maximizing context. You can set a custom token limit for a provider:

```bash
git-iris config --provider openai --token-limit 8000
```

### Project Configuration <a name="project-configuration"></a>

You can create project-specific configuration that overrides global settings:

```bash
# Create/update project config
git-iris project-config --instructions "Include JIRA ticket from branch name"
git-iris project-config --provider anthropic --model claude-3-5-sonnet

# View current project configuration
git-iris project-config --print
```

## 6. MCP Integration <a name="mcp-integration"></a>

Git-Iris can be used directly from AI assistants and editors through the Model Context Protocol (MCP).

### Starting the MCP Server

```bash
# Start with stdio transport (default)
git-iris serve

# Start with development mode for more verbose logging
git-iris serve --dev

# Start with SSE transport on specific port
git-iris serve --transport sse --port 3077
```

### Available Tools

The MCP integration exposes the following tools:

- `git_iris_commit`: Generate commit messages and perform Git commits
- `git_iris_code_review`: Generate comprehensive code reviews
- `git_iris_changelog`: Generate detailed changelogs
- `git_iris_release_notes`: Generate comprehensive release notes

### Using with Claude Desktop

1. Start Git-Iris as an MCP server: `git-iris serve`
2. Open Claude Desktop
3. Add Git-Iris as an MCP server in Claude's settings
4. Access Git-Iris functionality directly through Claude

### Using with Cursor

When using Cursor with Claude, Git-Iris tools are automatically available as long as the server is running.

### Using with VS Code

1. Start Git-Iris as an MCP server: `git-iris serve --transport sse --port 3077`
2. Install an MCP-compatible extension in VS Code
3. Configure the extension to connect to Git-Iris at `http://localhost:3077`

## 7. Best Practices <a name="best-practices"></a>

1. **Stage Changes Carefully**: Only stage the changes you want to include in the commit or review before running Git-Iris.

2. **Review Generated Content**: Always review AI-generated content to ensure accuracy and completeness.

3. **Use Custom Instructions**: Tailor the AI output to your project's needs by setting appropriate custom instructions.

4. **Leverage Gitmoji**: Enable Gitmoji for more expressive and categorized commit messages.

5. **Combine with Conventional Commits**: Use the conventional commits preset to generate standardized commit messages.

6. **Get Reviews Early**: Use the code review feature before committing to catch issues early in the development process.

7. **Use Detail Levels Appropriately**: Select the appropriate detail level (minimal, standard, or detailed) based on your needs when generating changelogs or release notes.

8. **Create Project-Specific Configuration**: Use project-specific configuration to tailor Git-Iris to each repository's needs.

9. **Use MCP for Integration**: Leverage the MCP integration to seamlessly incorporate Git-Iris into your AI-assisted workflows.

## 8. Troubleshooting <a name="troubleshooting"></a>

### Issue: Git-Iris fails to generate a message

- Ensure your API key is correctly set in the configuration
- Check your internet connection
- Verify that you have staged changes in your repository
- Try using a different provider with `--provider`

### Issue: Generated messages are not relevant

- Try providing more specific custom instructions
- Ensure you're using the latest version of Git-Iris
- Consider switching to a different LLM provider
- Try using a different instruction preset

### Issue: Token limit errors

- Increase the token limit in your configuration
- For very large changes, consider breaking them into smaller, logical commits
- Try a provider with larger token limits (e.g., Anthropic or Google)

### Issue: MCP server connection problems

- Check that the server is running with `ps aux | grep git-iris`
- Verify the transport type and port settings
- Ensure your firewall allows connections on the specified port

## 9. FAQ <a name="faq"></a>

**Q: Can I use Git-Iris with GitHub Actions or other CI/CD pipelines?**
A: While Git-Iris is primarily designed for local use, it can be integrated into CI/CD pipelines with some additional setup. Refer to our advanced documentation for details.

**Q: How does Git-Iris handle sensitive information?**
A: Git-Iris is designed to avoid sending sensitive information to LLM providers. However, always review generated messages to ensure no sensitive data is included. For highly sensitive repositories, consider using Ollama with a locally hosted model.

**Q: Can I use Git-Iris for projects in languages it doesn't explicitly support?**
A: Yes, Git-Iris can generate commit messages for any text-based files. Language-specific analysis is available for supported languages, but basic analysis works for all text files.

**Q: How can I contribute to Git-Iris?**
A: We welcome contributions! Please refer to our [CONTRIBUTING.md](../CONTRIBUTING.md) file for guidelines on how to contribute to the project.

**Q: Can I use different AI providers for different features?**
A: Yes, you can specify the AI provider for each command using the `--provider` flag.

**Q: How detailed are the code reviews?**
A: Code reviews analyze your staged changes and provide feedback across 11 quality dimensions including complexity, security, performance, and more. The depth depends on the AI model used and the complexity of your changes.

**Q: Can I use Git-Iris in offline environments?**
A: You can use Git-Iris with locally hosted models via Ollama when internet access is restricted.

**Q: Does Git-Iris support multiple languages?**
A: Yes, Git-Iris includes specialized file analyzers for 20+ languages and file types including Rust, JavaScript/TypeScript, Python, Java, C/C++, and many more.
