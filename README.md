# 🔮 Git-Iris: AI-Powered Git Workflow Assistant

<div align="center">

[![CI/CD](https://img.shields.io/github/actions/workflow/status/hyperb1iss/git-iris/cicd.yml?style=for-the-badge&logo=github-actions&logoColor=white&color=4C566A)](https://github.com/hyperb1iss/git-iris/actions)
[![License](https://img.shields.io/badge/License-Apache%202.0-5E81AC?style=for-the-badge&logo=apache&logoColor=white&color=3B6EA8)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Release](https://img.shields.io/github/release/hyperb1iss/git-iris.svg?style=for-the-badge&logo=github&logoColor=white&color=9D6DB3)][releases]
[![Crates.io](https://img.shields.io/crates/v/git-iris.svg?style=for-the-badge&logo=rust&logoColor=white&color=D35D47)][crates]
[![Rust](https://img.shields.io/badge/rust-stable-EBCB8B?style=for-the-badge&logo=rust&logoColor=white&color=EFBB4D)](https://www.rust-lang.org/)
[![ko-fi](https://img.shields.io/badge/Ko--fi-Support%20Me-A3BE8C?style=for-the-badge&logo=ko-fi&logoColor=white&color=82B062)](https://ko-fi.com/hyperb1iss)

_Elevate your Git workflow with the power of AI_ 🚀

[Installation](#installation) • [Configuration](#configuration) • [Usage](#usage) • [Contributing](#contributing) • [License](#license)

</div>

<div align="center">
  <img src="https://raw.githubusercontent.com/hyperb1iss/git-iris/main/docs/images/git-iris-screenshot-1.png" alt="Git-Iris Screenshot 1" width="33%">
  <img src="https://raw.githubusercontent.com/hyperb1iss/git-iris/main/docs/images/git-iris-screenshot-2.png" alt="Git-Iris Screenshot 2" width="33%">
  <img src="https://raw.githubusercontent.com/hyperb1iss/git-iris/main/docs/images/git-iris-screenshot-3.png" alt="Git-Iris Screenshot 3" width="33%">
</div>

_Git-Iris in action: AI-powered Git workflow assistance_

## 🌟 Overview

Git-Iris is a comprehensive AI-powered Git workflow assistant that enhances your development process from start to finish. It offers intelligent support for crafting meaningful commit messages, generating insightful changelogs, creating detailed release notes, and providing code reviews. By leveraging advanced AI models, Git-Iris boosts your productivity and improves the quality of your project documentation.

## ✨ Features

Git-Iris offers a suite of AI-powered tools to enhance your Git workflow:

- 🤖 **Intelligent Commit Messages**: Generate context-aware, meaningful commit messages
- 🔍 **AI-Powered Code Reviews**: Get detailed feedback on your changes before committing
- 📜 **Dynamic Changelog Generation**: Create structured, detailed changelogs between any two Git references
- 📋 **Comprehensive Release Notes**: Automatically generate release notes with summaries and key changes
- 🔄 **Multi-Provider AI Support**: Leverage OpenAI GPT-4o, Anthropic Claude, Gemini, or Ollama for AI capabilities
- 🎨 **Gitmoji Integration**: Add expressive emojis to your commits, changelogs, and release notes
- 🖥️ **Interactive CLI**: Refine AI-generated content through an intuitive command-line interface
- 🔧 **Customizable Workflows**: Tailor AI behavior with custom instructions and presets
- 📚 **Flexible Instruction Presets**: Quickly switch between different documentation styles
- 🧠 **Smart Context Extraction**: Analyze repository changes for more accurate AI-generated content
- 📊 **Intelligent Code Analysis**: Provide context-aware suggestions based on your codebase
- 🔍 **Relevance Scoring**: Prioritize important changes in generated content
- 📝 **Multi-Language Support**: Analyze changes in Rust, JavaScript, Python, Java, and more
- 🚀 **Performance Optimized**: Efficient token management for responsive AI interactions

## 🛠️ Installation

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

## ⚙️ Configuration

Git-Iris uses a configuration file located at `~/.config/git-iris/config.toml`. Set up your preferred AI provider:

```bash
# For OpenAI
git-iris config --provider openai --api-key YOUR_OPENAI_API_KEY

# For Anthropic Claude
git-iris config --provider claude --api-key YOUR_CLAUDE_API_KEY

# For Google Gemini
git-iris config --provider gemini --api-key YOUR_GEMINI_API_KEY

# For Ollama (no API key required)
git-iris config --provider ollama
```

Additional configuration options:

```bash
# Set default provider
git-iris config --default-provider openai

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

## 📖 Usage

Generate an AI-powered commit message:

```bash
git-iris gen
```

Options:

- `-a`, `--auto-commit`: Automatically commit with the generated message
- `-i`, `--instructions`: Provide custom instructions for this commit
- `--provider`: Specify an LLM provider (openai, claude, ollama)
- `--preset`: Use a specific instruction preset
- `--no-gitmoji`: Disable Gitmoji for this commit
- `-l`, `--log`: Enable logging to file
- `-p`, `--print`: Print the generated message to stdout and exit
- `--no-verify`: Skip verification steps (pre/post commit hooks)

Example:

```bash
git-iris gen -a -i "Focus on performance improvements" --provider claude --preset detailed
```

To generate a commit message and print it to stdout without starting the interactive process:

```bash
git-iris gen --print
```

### Interactive Commit Process

The interactive CLI allows you to refine and perfect your commit messages:

- Use arrow keys to navigate through suggestions
- Press 'e' to edit the current message
- Press 'i' to modify AI instructions
- Press 'g' to change the emoji
- Press 'p' to change the preset
- Press 'u' to edit user info
- Press 'r' to regenerate the message
- Press Enter to commit
- Press Esc to cancel

### Getting an AI Code Review

Git-Iris can analyze your staged changes and provide a detailed code review:

```bash
git-iris review
```

The review includes:

- A summary of the changes
- Code quality assessment
- Suggestions for improvement
- Identified issues
- Positive aspects of your code

Options:

- `-i`, `--instructions`: Provide custom instructions for this review
- `--provider`: Specify an LLM provider (openai, claude, ollama)
- `--preset`: Use a specific instruction preset
- `-p`, `--print`: Print the generated review to stdout and exit

Example:

```bash
git-iris review -i "Pay special attention to error handling" --provider claude
```

This allows you to get valuable feedback on your code before committing it, improving code quality and catching potential issues early.

### Generating a Changelog

Git-Iris can generate changelogs between two Git references:

```bash
git-iris changelog --from <from-ref> --to <to-ref>
```

Options:

- `--from`: Starting Git reference (commit hash, tag, or branch name)
- `--to`: Ending Git reference (defaults to HEAD if not specified)
- `--instructions`: Custom instructions for changelog generation
- `--preset`: Select an instruction preset for changelog generation
- `--detail-level`: Set the detail level (minimal, standard, detailed)
- `--gitmoji`: Enable or disable Gitmoji in the changelog

Example:

```bash
git-iris changelog --from v1.0.0 --to v1.1.0 --detail-level detailed --gitmoji true
```

This command generates a detailed changelog of changes between versions 1.0.0 and 1.1.0, including Gitmoji.

### Generating Release Notes

Git-Iris can also generate comprehensive release notes:

```bash
git-iris release-notes --from <from-ref> --to <to-ref>
```

Options:

- `--from`: Starting Git reference (commit hash, tag, or branch name)
- `--to`: Ending Git reference (defaults to HEAD if not specified)
- `--instructions`: Custom instructions for release notes generation
- `--preset`: Select an instruction preset for release notes generation
- `--detail-level`: Set the detail level (minimal, standard, detailed)
- `--gitmoji`: Enable or disable Gitmoji in the release notes

Example:

```bash
git-iris release-notes --from v1.0.0 --to v1.1.0 --preset conventional --detail-level standard
```

This command generates standard-level release notes between versions 1.0.0 and 1.1.0 using the conventional commits preset.

## 🎛️ Custom Instructions and Presets

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
- `cosmic`: Mystical, space-themed language

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

## 📄 License

Distributed under the Apache 2.0 License. See `LICENSE` for more information.

---

<div align="center">

📚 [Documentation](https://github.com/hyperb1iss/git-iris/wiki) • 🐛 [Report Bug](https://github.com/hyperb1iss/git-iris/issues) • 💡 [Request Feature](https://github.com/hyperb1iss/git-iris/issues)

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
