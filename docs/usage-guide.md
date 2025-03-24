# Git-Iris Usage Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Basic Usage](#basic-usage)
5. [Advanced Features](#advanced-features)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)
8. [FAQ](#faq)

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

# Set up Claude as the provider
git-iris config --provider claude --api-key YOUR_CLAUDE_API_KEY

# Enable Gitmoji
git-iris config --gitmoji true

# Set custom instructions
git-iris config --custom-instructions "Always mention the ticket number in the commit message"
```

For more detailed configuration options, refer to the [Configuration Guide](CONFIG.md).

## 4. Basic Usage <a name="basic-usage"></a>

### Generating a Commit Message

1. Stage your changes using `git add`
2. Run the following command:
   ```bash
   git-iris gen
   ```
3. Review the generated message in the interactive interface
4. Accept, edit, or regenerate the message as needed
5. Confirm to create the commit

### Command-line Options for Commit Generation

- `--verbose`: Enable detailed output
- `--gitmoji`: Override the Gitmoji setting
- `--provider`: Specify an LLM provider
- `--auto-commit`: Automatically commit with the generated message

Example:

```bash
git-iris gen --verbose --gitmoji --provider openai
```

### Getting an AI Code Review

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
   - Identified issues
   - Positive aspects of your code

Command-line options for code review:

- `-i`, `--instructions`: Provide custom instructions for the review
- `--provider`: Specify an LLM provider
- `--preset`: Use a specific instruction preset
- `-p`, `--print`: Print the review to stdout

Example:

```bash
git-iris review -i "Focus on security best practices" --provider claude
```

### Generating a Changelog

Generate a structured changelog between two Git references:

```bash
git-iris changelog --from v1.0.0 --to v1.1.0
```

Command-line options:

- `--from`: Starting Git reference (required)
- `--to`: Ending Git reference (defaults to HEAD)
- `--provider`: Specify an LLM provider
- `--instructions`: Custom instructions for changelog generation
- `--preset`: Use a specific instruction preset
- `--detail-level`: Set detail level (minimal, standard, detailed)
- `--gitmoji`: Enable or disable Gitmoji

### Creating Release Notes

Generate comprehensive release notes between two Git references:

```bash
git-iris release-notes --from v1.0.0 --to v1.1.0
```

Command-line options:

- `--from`: Starting Git reference (required)
- `--to`: Ending Git reference (defaults to HEAD)
- `--provider`: Specify an LLM provider
- `--instructions`: Custom instructions for release notes generation
- `--preset`: Use a specific instruction preset
- `--detail-level`: Set detail level (minimal, standard, detailed)
- `--gitmoji`: Enable or disable Gitmoji

## 5. Advanced Features <a name="advanced-features"></a>

### Custom Instructions

You can provide custom instructions to guide the AI in generating commit messages, code reviews, changelogs, or release notes:

```bash
git-iris gen --custom-instructions "Focus on performance improvements and API changes"
git-iris review --custom-instructions "Look for potential security vulnerabilities"
git-iris changelog --from v1.0.0 --to v1.1.0 --custom-instructions "Focus on user-facing changes"
```

### Interactive CLI Navigation

- Use arrow keys to navigate through suggestions
- Press 'e' to edit the current message
- Press 'i' to modify AI instructions
- Press 'r' to regenerate the message
- Press Enter to commit
- Press Esc to cancel

### Token Optimization

Git-Iris automatically optimizes token usage to stay within provider limits while maximizing context. You can set a custom token limit:

```bash
git-iris config --token-limit 8000
```

### Multiple LLM Providers

Git-Iris supports multiple LLM providers. You can switch between them:

```bash
git-iris gen --provider claude
```

## 6. Best Practices <a name="best-practices"></a>

1. **Stage Changes Carefully**: Only stage the changes you want to include in the commit or review before running Git-Iris.

2. **Review Generated Content**: Always review AI-generated content to ensure accuracy and completeness.

3. **Use Custom Instructions**: Tailor the AI output to your project's needs by setting appropriate custom instructions.

4. **Leverage Gitmoji**: Enable Gitmoji for more expressive and categorized commit messages.

5. **Combine with Conventional Commits**: Use custom instructions to guide Git-Iris in following the Conventional Commits format if your project requires it.

6. **Get Reviews Early**: Use the code review feature before committing to catch issues early in the development process.

7. **Use Detail Levels Appropriately**: Select the appropriate detail level (minimal, standard, or detailed) based on your needs when generating changelogs or release notes.

## 7. Troubleshooting <a name="troubleshooting"></a>

### Issue: Git-Iris fails to generate a message

- Ensure your API key is correctly set in the configuration
- Check your internet connection
- Verify that you have staged changes in your repository

### Issue: Generated messages are not relevant

- Try providing more specific custom instructions
- Ensure you're using the latest version of Git-Iris
- Consider switching to a different LLM provider

### Issue: Token limit errors

- Increase the token limit in your configuration
- For very large changes, consider breaking them into smaller, logical commits

## 8. FAQ <a name="faq"></a>

**Q: Can I use Git-Iris with GitHub Actions or other CI/CD pipelines?**
A: While Git-Iris is primarily designed for local use, it can be integrated into CI/CD pipelines with some additional setup. Refer to our advanced documentation for details.

**Q: How does Git-Iris handle sensitive information?**
A: Git-Iris is designed to avoid sending sensitive information to LLM providers. However, always review generated messages to ensure no sensitive data is included.

**Q: Can I use Git-Iris for projects in languages it doesn't explicitly support?**
A: Yes, Git-Iris can generate commit messages for any text-based files. Language-specific analysis is available for supported languages, but basic analysis works for all text files.

**Q: How can I contribute to Git-Iris?**
A: We welcome contributions! Please refer to our [CONTRIBUTING.md](../CONTRIBUTING.md) file for guidelines on how to contribute to the project.

**Q: Can I use different AI providers for different features?**
A: Yes, you can specify the AI provider for each command using the `--provider` flag.

**Q: How detailed are the code reviews?**
A: Code reviews analyze your staged changes and provide feedback on code quality, potential issues, and suggestions for improvements. The depth depends on the AI model used and the complexity of your changes.

**Q: Can I use Git-Iris in offline environments?**
A: You can use Git-Iris with locally hosted models via Ollama when internet access is restricted.
