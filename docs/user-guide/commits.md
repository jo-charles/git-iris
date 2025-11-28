# Commit Message Generation

Iris analyzes your staged changes and generates conventional commit messages that follow best practices, optionally enhanced with gitmoji for visual clarity.

## Quick Example

```bash
# Stage your changes
git add src/auth.rs

# Generate commit message (interactive)
git-iris gen

# Or auto-commit with generated message
git-iris gen --auto-commit

# Just print the message without committing
git-iris gen --print
```

## Command Reference

```bash
git-iris gen [FLAGS] [OPTIONS]
```

### Key Flags

| Flag                        | Description                                          |
| --------------------------- | ---------------------------------------------------- |
| `-a, --auto-commit`         | Automatically commit with generated message          |
| `-p, --print`               | Print message to stdout and exit (for scripting)     |
| `--no-gitmoji`              | Disable emoji prefixes for this commit               |
| `--no-verify`               | Skip pre/post-commit hooks                           |
| `-i, --instructions "text"` | Custom instructions for message style                |
| `--preset <name>`           | Use instruction preset (e.g., `concise`, `detailed`) |

### Global Options

| Option              | Description                           |
| ------------------- | ------------------------------------- |
| `--provider <name>` | Override LLM provider                 |
| `--debug`           | Show agent execution details          |
| `--quiet`           | Suppress spinners and progress output |

## Workflow Modes

### Interactive Mode (Default)

Launches Studio in Commit mode with a full TUI:

```bash
git-iris gen
```

- Edit generated message before committing
- View diff alongside message
- Chat with Iris to refine the message
- Press `c` to commit or `q` to cancel

### Auto-Commit Mode

Generate and commit in one step:

```bash
git-iris gen --auto-commit
```

**Note:** Not available for remote repositories. Use `--print` instead.

### Print Mode

Output message for scripting or manual use:

```bash
# Save to file
git-iris gen --print > commit-msg.txt

# Pipe to git commit
git-iris gen --print | git commit -F -

# Use in scripts
MSG=$(git-iris gen --print)
git commit -m "$MSG"
```

## Message Format

Iris generates messages following this structure:

```
<emoji> <type>: <subject>

<body>
```

**Example:**

```
✨ feat: Add JWT authentication middleware

Implements secure token-based authentication using RS256 signing.
Includes refresh token rotation and automatic expiry handling.
```

## Customizing Style

### Using Presets

```bash
# Concise messages
git-iris gen --preset concise

# Detailed explanations
git-iris gen --preset detailed

# Technical focus
git-iris gen --preset technical
```

See [Presets](./presets.md) for all available options.

### Custom Instructions

```bash
git-iris gen --instructions "Focus on performance impacts"

git-iris gen --instructions "Include migration notes for breaking changes"
```

## Gitmoji Control

### Disable Globally

Edit `~/.config/git-iris/config.toml`:

```toml
use_gitmoji = false
```

### Disable Per-Commit

```bash
git-iris gen --no-gitmoji
```

### Enable/Disable via Config

```bash
# Enable
git-iris config --gitmoji true

# Disable
git-iris config --gitmoji false
```

## Hook Integration

### Skip Verification

If you have pre-commit hooks that modify files or fail unexpectedly:

```bash
git-iris gen --auto-commit --no-verify
```

### Hook Workflow

Iris respects your Git hooks:

1. Runs **pre-commit** hook before generating message
2. If hook modifies staged files, regenerates message
3. Runs **commit-msg** hook on generated message
4. Runs **post-commit** hook after successful commit

## Tips

**For Large Changesets:**

- Iris uses parallel subagent analysis for 20+ files
- Consider using `--preset concise` to keep messages focused
- Split large changes into multiple commits when possible

**For Scripting:**

- Use `--print` to integrate with CI/CD
- Combine with `--quiet` to suppress all UI output
- Use `--no-verify` only when hooks are problematic

**For Consistency:**

- Set a default preset in your project config
- Use `--instructions` for project-specific conventions
- Configure gitmoji preference globally

## Examples

```bash
# Standard workflow
git add .
git-iris gen
# Edit in Studio, then press 'c' to commit

# Quick auto-commit
git add src/auth.rs
git-iris gen --auto-commit

# Concise style without emoji
git-iris gen --preset concise --no-gitmoji --print

# Detailed with custom focus
git-iris gen --preset detailed --instructions "Emphasize security changes"

# Debug agent execution
git-iris gen --debug --print
```

## Error Handling

**No Staged Changes:**

```
⚠ No staged changes. Please stage your changes first.
→ Use 'git add <file>' or 'git add .'
```

**Remote Repository:**

```
✗ Cannot automatically commit to a remote repository.
→ Use --print instead
```

**Pre-Commit Hook Failure:**
Iris will show the hook error and exit. Fix the issue and try again.
