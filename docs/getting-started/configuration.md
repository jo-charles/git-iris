# Configuration

Git-Iris uses a two-tier configuration system: global settings for your personal preferences, and project-specific settings that teams can share without exposing secrets.

## Global Configuration

Global settings live in `~/.config/git-iris/config.toml` and apply to all repositories.

### Set Up Your Provider

Pick your LLM provider and configure the API key:

**OpenAI:**

```bash
git-iris config --provider openai --api-key sk-...
```

**Anthropic:**

```bash
git-iris config --provider anthropic --api-key sk-ant-...
```

**Google:**

```bash
git-iris config --provider google --api-key AIza...
```

The API key is stored securely in your global config file.

### Environment Variables

Prefer environment variables? Git-Iris checks these:

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export GOOGLE_API_KEY=AIza...
```

Environment variables take precedence over config file values.

### View Current Configuration

```bash
git-iris config
```

This displays your active provider, model, presets, and all configured providers.

## Provider Details

Git-Iris supports three LLM providers with different strengths:

| Provider      | Default Model              | Fast Model                | Context Window | Best For                          |
| ------------- | -------------------------- | ------------------------- | -------------- | --------------------------------- |
| **openai**    | gpt-5.1                    | gpt-5.1-mini              | 128K           | General purpose, fast             |
| **anthropic** | claude-sonnet-4-5-20250929 | claude-haiku-4-5-20251001 | 200K           | Deep analysis, code understanding |
| **google**    | gemini-3-pro-preview       | gemini-2.5-flash          | 1M             | Massive context windows           |

**Fast models** are used for simple tasks like status updates and parsing. The primary model handles complex analysis.

### Override Models

Set a custom model for your provider:

```bash
git-iris config --provider anthropic --model claude-opus-4-20250514
```

Set a custom fast model:

```bash
git-iris config --provider openai --fast-model gpt-5.1-mini
```

### Token Limits

Override the default token limit:

```bash
git-iris config --provider openai --token-limit 4000
```

## Customization Options

### Gitmoji

Enable or disable emoji prefixes in commits:

```bash
git-iris config --gitmoji true   # Enable
git-iris config --gitmoji false  # Disable
```

### Instruction Presets

Set a default style preset:

```bash
git-iris config --preset conventional
git-iris config --preset detailed
git-iris config --preset cosmic
```

List available presets:

```bash
git-iris list-presets
```

Presets are categorized:

- **General** (both commits and reviews): `default`, `conventional`, `detailed`, `concise`, `cosmic`
- **Review-specific**: `security`, `performance`, `architecture`, `testing`, `maintainability`

### Custom Instructions

Add global instructions that apply to all operations:

```bash
git-iris config --instructions "Always include JIRA ticket numbers in brackets"
```

These combine with presets—your custom instructions are applied **in addition to** the preset style.

### Additional Parameters

Set provider-specific parameters:

```bash
git-iris config --provider openai --param temperature=0.7 --param max_tokens=150
```

## Project Configuration

Project settings live in `.irisconfig` in your repository root. Teams can commit this file to share settings **without exposing API keys**.

### Create Project Config

Set project-specific settings:

```bash
git-iris project-config --provider anthropic --preset conventional
```

Set a model for the project:

```bash
git-iris project-config --model claude-sonnet-4-5-20250929
```

Set project instructions:

```bash
git-iris project-config --instructions "Follow Angular commit format"
```

### View Project Config

```bash
git-iris project-config --print
```

### Security Note

**API keys are NEVER stored in `.irisconfig` files.** Only non-sensitive settings like models, presets, and custom instructions are saved. This prevents accidentally committing secrets to version control.

API keys must be in your global config (`~/.config/git-iris/config.toml`) or environment variables.

## Configuration Precedence

Settings are layered with this priority:

1. **CLI flags** (highest priority)
2. **Environment variables**
3. **Project config** (`.irisconfig`)
4. **Global config** (`~/.config/git-iris/config.toml`)
5. **Defaults** (lowest priority)

Example:

```bash
# Project config sets anthropic, but CLI overrides for this run
git-iris gen --provider openai
```

## Themes

Git-Iris supports custom themes for Studio. Set your preferred theme:

```bash
git-iris config --theme silkcircuit-neon  # Default
git-iris config --theme silkcircuit-dusk
git-iris config --theme silkcircuit-coral
```

List available themes:

```bash
git-iris themes
```

Override theme for a single session:

```bash
git-iris studio --theme silkcircuit-violet
```

## Advanced Options

### Debug Mode

Enable color-coded agent execution output:

```bash
git-iris gen --debug
```

This shows Iris's tool calls, reasoning, and token usage in real-time with gorgeous formatting.

### Logging

Log debug messages to a file:

```bash
git-iris gen --log
```

Custom log file path:

```bash
git-iris gen --log --log-file /tmp/iris-debug.log
```

### Quiet Mode

Suppress spinners and progress messages:

```bash
git-iris gen --quiet
```

Useful for scripting and CI/CD where you only want final output.

## Example Workflows

### Team Setup

Commit shared project settings:

```bash
git-iris project-config --provider anthropic --preset conventional
git add .irisconfig
git commit -m "Add Git-Iris project configuration"
```

Team members clone the repo and only need to set their personal API key:

```bash
git-iris config --provider anthropic --api-key sk-ant-...
```

### Multiple Providers

Switch providers easily:

```bash
git-iris config --provider openai --api-key sk-...
git-iris config --provider anthropic --api-key sk-ant-...

# Use OpenAI for this commit
git-iris gen --provider openai

# Use Anthropic for reviews
git-iris review --provider anthropic
```

### Per-Repository Presets

Different projects can use different styles:

```bash
cd backend-api
git-iris project-config --preset conventional

cd frontend-app
git-iris project-config --preset detailed
```

## Configuration File Format

The global config file (`~/.config/git-iris/config.toml`) looks like this:

```toml
default_provider = "anthropic"
use_gitmoji = true
instruction_preset = "conventional"
instructions = "Always include ticket numbers"
theme = "silkcircuit-neon"

[providers.anthropic]
api_key = "sk-ant-..."
model = "claude-sonnet-4-5-20250929"
fast_model = "claude-haiku-4-5-20251001"

[providers.openai]
api_key = "sk-..."
model = "gpt-5.1"
fast_model = "gpt-5.1-mini"
```

You can edit this manually if you prefer, but the `git-iris config` command is safer.

## What's Next?

With configuration complete, explore Git-Iris features:

- **[User Guide: Commits](/user-guide/commits.md)** — Master AI-generated commit messages
- **[User Guide: Reviews](/user-guide/reviews.md)** — Get comprehensive code reviews
- **[Iris Studio](/studio/)** — Learn all six Studio modes

Press `Shift+S` in Studio to adjust settings without leaving the interface.
