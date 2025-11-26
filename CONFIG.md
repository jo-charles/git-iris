# Git-Iris Configuration Guide

Git-Iris uses a TOML configuration file located at `~/.config/git-iris/config.toml`. This document outlines all available configuration options and their usage.

## Configuration Structure

The configuration file is organized into these main sections:

1. Global settings
2. Default provider
3. Provider-specific configurations

## Configuration Options

### Global Settings

- `use_gitmoji`: Boolean (optional)
  - Description: Enables Gitmoji in commit messages.
  - Default: `false`
  - Example: `use_gitmoji = true`

- `custom_instructions`: String (optional)
  - Description: Custom instructions included in all LLM prompts for commit messages, code reviews, changelogs, and release notes.
  - Default: `""`
  - Example: `custom_instructions = "Always mention the ticket number and focus on the impact of changes."`

- `instruction_preset`: String (optional)
  - Description: Default preset for AI instructions.
  - Default: `"default"`
  - Example: `instruction_preset = "conventional"`

### Default Provider

- `default_provider`: String (required)
  - Description: The default LLM provider.
  - Default: `"openai"`
  - Example: `default_provider = "claude"`

### Provider-Specific Configurations

Each provider has its own subtable under `[providers]` with these fields:

- `api_key`: String (required)
  - Description: The provider's API key.
  - Example: `api_key = "sk-1234567890abcdef"`

- `model`: String (optional)
  - Description: The specific model to use.
  - Default: Provider-dependent
  - Example: `model = "gpt-4o"`

- `additional_params`: Table (optional)
  - Description: Additional provider or model-specific parameters.
  - Example: `additional_params = { temperature = "0.7", max_tokens = "150" }`

- `custom_token_limit`: Integer (optional)
  - Description: Custom token limit for this provider.
  - Default: Provider-dependent
  - Example: `custom_token_limit = 8000`

## Supported Providers and Default Models

| Provider  | Default Model              | Notes                      |
| --------- | -------------------------- | -------------------------- |
| openai    | gpt-4o                     | Requires OPENAI_API_KEY    |
| anthropic | claude-3-7-sonnet-20250219 | Requires ANTHROPIC_API_KEY |
| google    | gemini-2.0-flash           | Requires GOOGLE_API_KEY    |
| groq      | llama-3.1-70b-versatile    | Requires GROQ_API_KEY      |
| ollama    | llama3                     | Local, no API key needed   |
| xai       | grok-2-beta                | Requires XAI_API_KEY       |
| deepseek  | deepseek-chat              | Requires DEEPSEEK_API_KEY  |
| phind     | phind-v2                   | No API key needed          |

## Example Configuration File

```toml
use_gitmoji = true
custom_instructions = """
Always mention the ticket number if applicable.
Focus on the impact of changes rather than implementation details.
"""
default_provider = "openai"
instruction_preset = "conventional"

[providers.openai]
api_key = "sk-1234567890abcdef"
model = "gpt-4"
additional_params = { temperature = "0.7", max_tokens = "150" }
custom_token_limit = 8000

[providers.anthropic]
api_key = "sk-abcdef1234567890"
model = "claude-3-7-sonnet-20250219"
additional_params = { temperature = "0.8" }
custom_token_limit = 200000

[providers.gemini]
api_key = "your-gemini-api-key"
model = "gemini-2.0-flash"
additional_params = { temperature = "0.7" }
custom_token_limit = 1048576
```

## Changing Configuration

Use the `git-iris config` command to modify settings:

```bash
git-iris config --provider openai --api-key YOUR_API_KEY
git-iris config --provider openai --model gpt-4
git-iris config --provider openai --param temperature=0.7 --param max_tokens=150
git-iris config --gitmoji true
git-iris config --custom-instructions "Your custom instructions here"
git-iris config --token-limit 8000
git-iris config --preset conventional
```

You can also edit the `~/.config/git-iris/config.toml` file directly with a text editor.

## Adding a New Provider

To add a new provider, create a new section under `[providers]`:

```toml
[providers.new_provider]
api_key = "your-api-key-here"
model = "model-name"
additional_params = { param1 = "value1", param2 = "value2" }
custom_token_limit = 10000
```

Set it as the default provider if desired:

```toml
default_provider = "new_provider"
```

Note: The application code must support the new provider's API for it to function.

## Token Optimization

Git-Iris automatically optimizes token usage to maximize context while staying within provider limits. You can set a custom token limit for each provider using the `custom_token_limit` option.

## Security Notes

- Keep your API keys secret and never share your configuration file containing API keys.
- Git-Iris stores API keys in the configuration file. Ensure the file has appropriate permissions (readable only by you).
- Consider using environment variables for API keys in shared environments.

## Troubleshooting

If you encounter issues:

1. Verify your API keys are correct and have the necessary permissions.
2. Check that you're using supported models for each provider.
3. Ensure your custom instructions don't exceed token limits.
4. Review the Git-Iris logs for any error messages.
5. For code review or changelog generation issues, try using a higher token limit.

For further assistance, please refer to the [Git-Iris documentation](https://github.com/hyperb1iss/git-iris/wiki) or [open an issue](https://github.com/hyperb1iss/git-iris/issues) on the GitHub repository.
