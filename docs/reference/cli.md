# CLI Command Reference

Complete reference for all Git-Iris commands and flags.

## Global Flags

Available on all commands:

| Flag                | Short | Description                                          |
| ------------------- | ----- | ---------------------------------------------------- |
| `--log`             | `-l`  | Log debug messages to file                           |
| `--log-file <PATH>` |       | Custom log file path (default: `git-iris-debug.log`) |
| `--quiet`           | `-q`  | Suppress non-essential output                        |
| `--version`         | `-v`  | Display version information                          |
| `--repo <URL>`      | `-r`  | Use remote repository instead of local               |
| `--debug`           |       | Enable debug mode with color-coded agent execution   |
| `--theme <NAME>`    |       | Override theme for this session                      |
| `--help`            | `-h`  | Show help information                                |

## Commands

### `gen` - Generate Commit Messages

```bash
git-iris gen [OPTIONS]
```

Generate AI-powered commit messages for staged changes.

**Options:**

| Flag                    | Short | Description                                 |
| ----------------------- | ----- | ------------------------------------------- |
| `--auto-commit`         | `-a`  | Automatically commit with generated message |
| `--no-gitmoji`          |       | Disable gitmoji for this commit             |
| `--print`               | `-p`  | Print message to stdout and exit            |
| `--no-verify`           |       | Skip pre/post commit hooks                  |
| `--provider <NAME>`     |       | Override default provider                   |
| `--instructions <TEXT>` | `-i`  | Custom instructions                         |
| `--preset <NAME>`       |       | Instruction preset name                     |
| `--gitmoji <BOOL>`      |       | Enable/disable gitmoji                      |

**Examples:**

```bash
# Interactive mode (launches Studio)
git-iris gen

# Print only
git-iris gen --print

# Auto-commit
git-iris gen --auto-commit

# Use specific provider
git-iris gen --provider anthropic --print

# Custom instructions
git-iris gen -i "Focus on security implications" --print
```

---

### `studio` - Launch Iris Studio

```bash
git-iris studio [OPTIONS]
```

Launch unified TUI for all operations.

**Options:**

| Flag            | Description                                                    |
| --------------- | -------------------------------------------------------------- |
| `--mode <MODE>` | Initial mode: `explore`, `commit`, `review`, `pr`, `changelog` |
| `--from <REF>`  | Starting ref for comparison                                    |
| `--to <REF>`    | Ending ref for comparison                                      |

**Examples:**

```bash
# Auto-detect mode
git-iris studio

# Start in commit mode
git-iris studio --mode commit

# Start in PR mode with refs
git-iris studio --mode pr --from main --to feature-branch
```

---

### `review` - Code Review

```bash
git-iris review [OPTIONS]
```

Generate comprehensive code reviews with AI.

**Options:**

| Flag                 | Description |
| -------------------- | ----------- | -------------------------------------- |
| `--print`            | `-p`        | Print review to stdout                 |
| `--raw`              |             | Output raw markdown without formatting |
| `--include-unstaged` |             | Include unstaged changes               |
| `--commit <HASH>`    |             | Review specific commit                 |
| `--from <REF>`       |             | Starting branch for comparison         |
| `--to <REF>`         |             | Target branch for comparison           |

**Examples:**

```bash
# Review staged changes
git-iris review

# Review specific commit
git-iris review --commit abc1234

# Review branch comparison
git-iris review --from main --to feature-branch

# Include unstaged changes
git-iris review --include-unstaged --print
```

---

### `pr` - Pull Request Descriptions

```bash
git-iris pr [OPTIONS]
```

Generate pull request descriptions.

**Options:**

| Flag           | Description |
| -------------- | ----------- | ------------------------------ |
| `--print`      | `-p`        | Print to stdout                |
| `--raw`        |             | Output raw markdown            |
| `--from <REF>` |             | Starting ref (default: `main`) |
| `--to <REF>`   |             | Target ref (default: `HEAD`)   |

**Examples:**

```bash
# PR from main to current branch
git-iris pr

# PR from specific branch
git-iris pr --from develop --to feature-branch

# Single commit PR
git-iris pr --from abc1234

# Print only
git-iris pr --print
```

---

### `changelog` - Generate Changelog

```bash
git-iris changelog --from <REF> [OPTIONS]
```

Generate changelog between Git references.

**Options:**

| Flag                    | Required | Description                                   |
| ----------------------- | -------- | --------------------------------------------- |
| `--from <REF>`          | Yes      | Starting Git reference                        |
| `--to <REF>`            | No       | Ending reference (default: `HEAD`)            |
| `--raw`                 | No       | Output raw markdown                           |
| `--update`              | No       | Update CHANGELOG.md file                      |
| `--file <PATH>`         | No       | Changelog file path (default: `CHANGELOG.md`) |
| `--version-name <NAME>` | No       | Explicit version name                         |

**Examples:**

```bash
# Changelog from tag to HEAD
git-iris changelog --from v1.0.0

# Changelog between tags
git-iris changelog --from v1.0.0 --to v2.0.0

# Update CHANGELOG.md
git-iris changelog --from v1.0.0 --update

# Custom version name
git-iris changelog --from v1.0.0 --version-name "v2.0.0"
```

---

### `release-notes` - Generate Release Notes

```bash
git-iris release-notes --from <REF> [OPTIONS]
```

Generate comprehensive release notes.

**Options:**

| Flag                    | Required | Description                        |
| ----------------------- | -------- | ---------------------------------- |
| `--from <REF>`          | Yes      | Starting Git reference             |
| `--to <REF>`            | No       | Ending reference (default: `HEAD`) |
| `--raw`                 | No       | Output raw markdown                |
| `--version-name <NAME>` | No       | Explicit version name              |

**Examples:**

```bash
# Release notes from tag
git-iris release-notes --from v1.0.0

# Between tags
git-iris release-notes --from v1.0.0 --to v2.0.0

# Custom version
git-iris release-notes --from v1.0.0 --version-name "2.0.0-beta"
```

---

### `config` - Configuration Management

```bash
git-iris config [OPTIONS]
```

Configure global Git-Iris settings.

**Options:**

| Flag                  | Description               |
| --------------------- | ------------------------- |
| `--provider <NAME>`   | Set default provider      |
| `--api-key <KEY>`     | Set API key               |
| `--model <NAME>`      | Set primary model         |
| `--fast-model <NAME>` | Set fast model            |
| `--token-limit <NUM>` | Set token limit           |
| `--param <KEY=VALUE>` | Set additional parameters |

**Examples:**

```bash
# Set provider and API key
git-iris config --provider anthropic --api-key sk-ant-...

# Configure models
git-iris config --provider anthropic \
  --model claude-sonnet-4-5-20250929 \
  --fast-model claude-haiku-4-5-20251001

# Set token limit
git-iris config --provider openai --token-limit 8000

# Additional parameters
git-iris config --provider openai \
  --param temperature=0.7 \
  --param max_tokens=4096
```

---

### `project-config` - Project Configuration

```bash
git-iris project-config [OPTIONS]
```

Manage project-specific `.irisconfig` file.

**Options:**

| Flag                  | Description             |
| --------------------- | ----------------------- | ---------------------------- |
| `--provider <NAME>`   | Set project provider    |
| `--model <NAME>`      | Set project model       |
| `--fast-model <NAME>` | Set project fast model  |
| `--token-limit <NUM>` | Set project token limit |
| `--param <KEY=VALUE>` | Set project parameters  |
| `--print`             | `-p`                    | Print current project config |

**Examples:**

```bash
# Create project config
git-iris project-config --provider anthropic

# Set project model
git-iris project-config --model claude-sonnet-4-5-20250929

# View project config
git-iris project-config --print
```

---

### `list-presets` - List Instruction Presets

```bash
git-iris list-presets
```

Display all available instruction presets.

**No options.**

---

### `themes` - List Themes

```bash
git-iris themes
```

Display all available themes.

**No options.**

## Common Workflows

### First-Time Setup

```bash
# Install
brew install hyperb1iss/tap/git-iris

# Configure
git-iris config --provider anthropic --api-key YOUR_KEY
git-iris config --model claude-sonnet-4-5-20250929
```

### Daily Usage

```bash
# Stage changes
git add .

# Generate commit (interactive)
git-iris gen

# Or auto-commit
git-iris gen --auto-commit
```

### Code Review Workflow

```bash
# Review staged changes
git add .
git-iris review

# Or review a PR branch
git-iris review --from main --to feature-branch --print
```

### Release Workflow

```bash
# Generate changelog
git-iris changelog --from v1.0.0 --update

# Generate release notes
git-iris release-notes --from v1.0.0 > RELEASE_NOTES.md

# Create PR description
git-iris pr --from main > pr_description.md
```

## Debug and Troubleshooting

### Enable Debug Logging

```bash
# Basic logging
git-iris gen --log

# Custom log file
git-iris gen --log --log-file my-debug.log

# Color-coded agent debug
git-iris gen --debug
```

### Test Configuration

```bash
# Test with print (no commit)
git-iris gen --print

# Test specific provider
git-iris gen --provider openai --print

# Verify API key works
git-iris review --print
```

### Remote Repository Testing

```bash
# Test against remote repo
git-iris gen --repo https://github.com/user/repo --print
```

## Exit Codes

| Code | Meaning                                    |
| ---- | ------------------------------------------ |
| `0`  | Success                                    |
| `1`  | General error                              |
| `2`  | Configuration error                        |
| `3`  | Git error (not in repo, no staged changes) |
| `4`  | API error (authentication, rate limit)     |

## Environment Variables

See [Environment Variables](../configuration/environment.md) for details.

| Variable            | Purpose                   |
| ------------------- | ------------------------- |
| `OPENAI_API_KEY`    | OpenAI authentication     |
| `ANTHROPIC_API_KEY` | Anthropic authentication  |
| `GOOGLE_API_KEY`    | Google authentication     |
| `GITIRIS_PROVIDER`  | Default provider (Docker) |
| `GITIRIS_API_KEY`   | Generic API key (Docker)  |

## Shell Aliases

Recommended aliases for common operations:

```bash
# ~/.bashrc or ~/.zshrc

# Quick commit
alias gic='git-iris gen --auto-commit'

# Print commit message
alias gim='git-iris gen --print'

# Code review
alias gir='git-iris review --print'

# Launch studio
alias gis='git-iris studio'
```
