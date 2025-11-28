# LLM Providers

Git-Iris supports three LLM providers: OpenAI, Anthropic, and Google.

## Provider Overview

| Provider      | Default Model                | Fast Model                  | Context Window | API Key Env         |
| ------------- | ---------------------------- | --------------------------- | -------------- | ------------------- |
| **OpenAI**    | `gpt-5.1`                    | `gpt-5.1-mini`              | 128K           | `OPENAI_API_KEY`    |
| **Anthropic** | `claude-sonnet-4-5-20250929` | `claude-haiku-4-5-20251001` | 200K           | `ANTHROPIC_API_KEY` |
| **Google**    | `gemini-3-pro-preview`       | `gemini-2.5-flash`          | 1M             | `GOOGLE_API_KEY`    |

## Configuration Format

Each provider has its own section under `[providers]`:

```toml
[providers.PROVIDER_NAME]
api_key = "YOUR_API_KEY"
model = "model-name"           # Optional: primary model
fast_model = "fast-model-name" # Optional: for status updates
token_limit = 8000             # Optional: custom limit
```

## OpenAI Configuration

```toml
[providers.openai]
api_key = "sk-..."
model = "gpt-5.1"
fast_model = "gpt-5.1-mini"
token_limit = 128000
```

### CLI Setup

```bash
git-iris config --provider openai --api-key YOUR_API_KEY
git-iris config --provider openai --model gpt-5.1
```

### Environment Variable

```bash
export OPENAI_API_KEY="sk-..."
```

## Anthropic Configuration

```toml
[providers.anthropic]
api_key = "sk-ant-..."
model = "claude-sonnet-4-5-20250929"
fast_model = "claude-haiku-4-5-20251001"
token_limit = 200000
```

### CLI Setup

```bash
git-iris config --provider anthropic --api-key YOUR_API_KEY
git-iris config --provider anthropic --model claude-sonnet-4-5-20250929
```

### Environment Variable

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Legacy Alias

The provider name `claude` is still supported as an alias for `anthropic`.

## Google Configuration

```toml
[providers.google]
api_key = "your-google-api-key"
model = "gemini-3-pro-preview"
fast_model = "gemini-2.5-flash"
token_limit = 1000000
```

### CLI Setup

```bash
git-iris config --provider google --api-key YOUR_API_KEY
git-iris config --provider google --model gemini-3-pro-preview
```

### Environment Variable

```bash
export GOOGLE_API_KEY="..."
```

## Switching Providers

### Set Default Provider

```bash
git-iris config --provider anthropic
```

### Override Per-Command

```bash
git-iris gen --provider openai
git-iris review --provider google
```

## Additional Parameters

Provider-specific parameters can be set using `--param`:

```bash
git-iris config --provider openai --param temperature=0.7
git-iris config --provider openai --param max_tokens=4096
```

In TOML:

```toml
[providers.openai]
api_key = "sk-..."

  [providers.openai.additional_params]
  temperature = "0.7"
  max_tokens = "4096"
```

## Token Limits

Each provider has a default context window. You can override this:

```bash
git-iris config --provider anthropic --token-limit 100000
```

This is useful for:

- Cost control (smaller limit = fewer tokens)
- Faster responses
- Testing with limited context

## API Key Priority

API keys are loaded in this order:

1. **Config file** (`~/.config/git-iris/config.toml`)
2. **Environment variable** (`OPENAI_API_KEY`, etc.)

Project configs (`.irisconfig`) **never** contain API keys for security.

## Verification

Check your provider configuration:

```bash
# View current configuration
cat ~/.config/git-iris/config.toml

# Test with a command
git-iris gen --print
```

If authentication fails, Git-Iris will tell you which environment variable to set.

## Security Best Practices

- **Never commit** `config.toml` with API keys
- Use environment variables in CI/CD
- Restrict file permissions:
  ```bash
  chmod 600 ~/.config/git-iris/config.toml
  ```
- Rotate API keys periodically
- Use separate keys for different projects if needed
