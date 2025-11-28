# Quick Start

Generate your first AI-powered commit message in under 60 seconds.

## Prerequisites

- Git-Iris installed ([Installation Guide](./installation.md))
- LLM provider API key ([Configuration Guide](./configuration.md))

## 1. Configure Your API Key

Pick your provider and set the API key:

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

Your configuration is saved to `~/.config/git-iris/config.toml` and works across all repositories.

## 2. Make Some Changes

Navigate to any Git repository and stage changes:

```bash
cd your-project
# Make changes to files
git add .
```

Git-Iris needs staged changes to generate a commit message.

## 3. Generate a Commit Message

Run the magic command:

```bash
git-iris gen
```

Iris will:

1. Analyze your staged changes
2. Explore relevant files using her tool arsenal
3. Generate a contextual commit message
4. Open Studio in Commit mode

**What You'll See:**

- Your staged files in the left panel
- A live diff view showing changes
- An AI-generated commit message ready for review
- Real-time token streaming as Iris thinks

## 4. Review and Commit

In Studio:

| Action         | Key |
| -------------- | --- |
| Edit message   | `e` |
| Chat with Iris | `/` |
| Commit changes | `c` |
| Change emoji   | `m` |
| Switch preset  | `p` |

Press `c` to commit, or `e` to edit the message manually.

## Alternative: Print to Stdout

For automation or scripting:

```bash
git-iris gen --print
```

This outputs the commit message without starting Studio.

**Auto-commit (use with caution):**

```bash
git-iris gen --auto-commit
```

## What Just Happened?

Iris didn't just pattern-match your diff. She:

- Used `git_diff()` to understand what changed
- Called `file_analyzer()` to examine file metadata
- Ran `code_search()` to find related patterns
- Analyzed `git_log()` to match your commit style
- Generated a message that captures **why** the change matters

This is agent-first intelligence. Iris gathers precisely what she needs via tool calls.

## Explore Studio

Launch Studio directly to access all modes:

```bash
git-iris studio
```

Try these modes:

- **Explore Mode** (`Shift+E`) — Navigate code with semantic blame
- **Review Mode** (`Shift+R`) — Get AI code reviews
- **PR Mode** (`Shift+P`) — Generate pull request descriptions
- **Changelog Mode** (`Shift+L`) — Create changelogs between refs

Press `/` in any mode to chat with Iris. Ask her to refine content, explain changes, or answer questions.

## Customize Your Experience

### Use a Different Style Preset

```bash
git-iris gen --preset conventional
git-iris gen --preset detailed
git-iris gen --preset cosmic  # For the mystically inclined
```

List all presets:

```bash
git-iris list-presets
```

### Disable Gitmoji

```bash
git-iris gen --no-gitmoji
```

Or disable globally:

```bash
git-iris config --gitmoji false
```

### Add Custom Instructions

Per-commit:

```bash
git-iris gen -i "Mention the ticket number and performance impact"
```

Globally:

```bash
git-iris config --instructions "Always include JIRA ticket in commit messages"
```

## Next Steps

You've generated your first AI commit. Now explore:

- **[Configuration Guide](./configuration.md)** — Deep dive into settings and providers
- **[Iris Studio](/studio/)** — Learn all six Studio modes
- **[User Guide](/user-guide/)** — Master commits, reviews, changelogs, and more

Press `?` in Studio to see all keyboard shortcuts.
