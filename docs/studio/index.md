# Iris Studio

**Iris Studio** is Git-Iris's unified TUI (Terminal User Interface) for all AI-powered git workflows. It provides a beautiful, electric-themed interface where you can explore code, generate commit messages, create reviews, and more—all without leaving your terminal.

## Visual Overview

Studio uses a **3-panel layout** across all modes:

```
┌─────────────────────────────────────────────────────────────────┐
│  [Explore] Commit Review PR Changelog Release      ?Help /Chat  │
├───────────────┬──────────────────────┬─────────────────────────┤
│               │                      │                         │
│     LEFT      │       CENTER         │         RIGHT           │
│               │                      │                         │
│  File Tree    │   Code/Content       │    Context/Details      │
│  or Lists     │   (Main Focus)       │    (Supporting Info)    │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
│               │                      │                         │
├───────────────┴──────────────────────┴─────────────────────────┤
│  main | 3 staged, 2 modified | ⠋ Iris thinking...             │
└─────────────────────────────────────────────────────────────────┘
```

- **Top Bar**: Mode tabs, status indicators, and global shortcuts
- **Left Panel**: Navigation (files, commits, changelogs)
- **Center Panel**: Main content (code, messages, reviews, diffs)
- **Right Panel**: Context (diffs, metadata, analysis)
- **Bottom Bar**: Git status, Iris status, notifications

## The Electric SilkCircuit Theme

Studio uses the **SilkCircuit Neon** color palette for a consistent, cyberpunk aesthetic:

- **Electric Purple** `#e135ff` — Active mode, selections
- **Neon Cyan** `#80ffea` — File paths, function names
- **Coral** `#ff6ac1` — Commit hashes, numbers
- **Electric Yellow** `#f1fa8c` — Warnings, attention
- **Success Green** `#50fa7b` — Success states, additions
- **Error Red** `#ff6363` — Errors, deletions

Background is dark (`#121218`) with code panels in `#1e1e28`.

## Global Keybindings

These work in **all modes**:

| Key                  | Action                                   |
| -------------------- | ---------------------------------------- |
| <kbd>?</kbd>         | Open help modal                          |
| <kbd>/</kbd>         | Chat with Iris (universal across modes)  |
| <kbd>Tab</kbd>       | Focus next panel (Left → Center → Right) |
| <kbd>Shift+Tab</kbd> | Focus previous panel                     |
| <kbd>Shift+E</kbd>   | Switch to Explore mode                   |
| <kbd>Shift+C</kbd>   | Switch to Commit mode                    |
| <kbd>Shift+R</kbd>   | Switch to Review mode                    |
| <kbd>Shift+P</kbd>   | Switch to PR mode                        |
| <kbd>Shift+L</kbd>   | Switch to Changelog mode                 |
| <kbd>Shift+N</kbd>   | Switch to Release Notes mode             |
| <kbd>,</kbd>         | Open Settings                            |
| <kbd>q</kbd>         | Quit Studio                              |
| <kbd>Esc</kbd>       | Close modal / Clear selection            |

## Available Modes

Studio provides six specialized modes for different git workflows:

### [Explore Mode](modes/explore.md)

Ever wondered _why_ a line of code exists? Explore Mode is your detective tool. Navigate files, ask Iris "why was this added?", and get historical context backed by commit analysis and code patterns.

- **Panel Layout**: File Tree | Code View | Analysis
- **Key Feature**: Semantic blame ("why does this code exist?")
- **When to use**: Understanding unfamiliar code, investigating bugs, learning codebase

### [Commit Mode](modes/commit.md)

Say goodbye to "fix stuff" commits. Iris analyzes your changes, understands the _intent_ behind them, and crafts messages that your future self will thank you for.

- **Panel Layout**: Changed Files | Message Editor | Diff View
- **Key Feature**: Smart emoji selection + custom instructions
- **When to use**: Creating meaningful commits with context

### [Review Mode](modes/review.md)

Get a code review before your PR does. Iris examines your changes for security issues, performance concerns, and best practices—catching problems before they reach your teammates.

- **Panel Layout**: Changed Files | Review Output | Diff View
- **Key Feature**: Multi-dimensional analysis (security, perf, best practices)
- **When to use**: Pre-commit review, PR preparation, code quality checks

### [PR Mode](modes/pr.md)

Stop staring at an empty PR description. Iris synthesizes your commits into a coherent narrative that tells reviewers exactly what changed, why, and how to test it.

- **Panel Layout**: Commit List | PR Description | Diff View
- **Key Feature**: Structured markdown output ready to paste
- **When to use**: Creating PRs, documenting feature branches

### [Changelog Mode](modes/changelog.md)

Release day shouldn't mean hours of commit archaeology. Iris categorizes your changes into a proper Keep a Changelog format—Added, Changed, Fixed, Removed—automatically.

- **Panel Layout**: Commit List | Changelog Output | Diff View
- **Key Feature**: Categorized changes (Added, Changed, Fixed, etc.)
- **When to use**: Release preparation, version documentation

### [Release Notes Mode](modes/release-notes.md)

Turn technical commits into user-friendly release notes. Iris highlights what users care about, explains breaking changes clearly, and formats everything for your audience.

- **Panel Layout**: Commit List | Release Notes | Diff View
- **Key Feature**: User-focused narrative with breaking changes
- **When to use**: Public releases, customer communication

## Chat with Iris

Press <kbd>/</kbd> in any mode to open the chat modal. Chat is **universal**—it persists across modes and can access all generated content.

### What You Can Do in Chat

- **Ask questions** about generated content: "Why did you choose this emoji?"
- **Request changes**: "Make the commit message more concise"
- **Get explanations**: "Explain the security issues you found"
- **Update content directly**: Iris has tools to modify commit messages, reviews, and PRs

Chat state persists throughout your session, so you can have ongoing conversations while switching between modes.

See [Chat Documentation](chat.md) for details.

## Navigation Patterns

Studio follows vim-like navigation for consistency:

### Movement

- <kbd>j</kbd>/<kbd>↓</kbd> — Move down
- <kbd>k</kbd>/<kbd>↑</kbd> — Move up
- <kbd>h</kbd>/<kbd>←</kbd> — Collapse / Previous (context-dependent)
- <kbd>l</kbd>/<kbd>→</kbd> — Expand / Next (context-dependent)
- <kbd>g</kbd>/<kbd>Home</kbd> — Go to first
- <kbd>G</kbd>/<kbd>End</kbd> — Go to last
- <kbd>Ctrl+d</kbd>/<kbd>PgDn</kbd> — Page down
- <kbd>Ctrl+u</kbd>/<kbd>PgUp</kbd> — Page up

### Selection

- <kbd>Enter</kbd> — Select item / Toggle expand
- <kbd>v</kbd> — Enter visual selection mode (Explore only)
- <kbd>y</kbd> — Copy to clipboard

### Common Actions

- <kbd>r</kbd> — Regenerate / Refresh (context-dependent)
- <kbd>e</kbd> — Edit (Commit mode)
- <kbd>w</kbd> — "Why?" / Semantic blame (Explore mode)

See mode-specific pages for detailed keybindings.

## Status Indicators

### Git Status (Bottom Left)

```
main | 3 staged, 2 modified, 1 untracked
```

Shows current branch and change counts.

### Iris Status (Bottom Center)

- `⠋ Generating commit message...` — Iris is thinking
- `✓ Done` — Task complete
- `✗ Error: ...` — Something went wrong
- Idle when no task is running

### Notifications (Bottom Right)

Temporary messages appear for 5 seconds:

- **Green** — Success
- **Yellow** — Warning
- **Red** — Error
- **Cyan** — Info

## Tips & Tricks

### Focus Flow

Most modes default to center panel focus (where the action is). Use <kbd>Tab</kbd> to move to side panels when you need more context.

### Quick Mode Switching

Hold <kbd>Shift</kbd> + first letter of mode name:

- <kbd>Shift+E</kbd> = Explore
- <kbd>Shift+C</kbd> = Commit
- <kbd>Shift+R</kbd> = Review

### Chat Everywhere

<kbd>/</kbd> opens chat from anywhere. Ask Iris to:

- Explain generated content
- Make changes to messages/reviews
- Answer questions about the codebase

### Clipboard Integration

Press <kbd>y</kbd> in most contexts to copy:

- Commit messages
- Code lines/selections
- Review content
- PR descriptions
- Changelog entries

## Next Steps

- Read [Navigation Patterns](navigation.md) for detailed movement
- Jump to a [mode-specific guide](modes/explore.md) (Explore, Commit, Review, PR, Changelog, Release Notes)
- Learn about [Chat with Iris](chat.md)
