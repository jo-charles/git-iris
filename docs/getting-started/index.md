# Getting Started

_"Finally, commit messages that explain why, not just what."_

Git-Iris is powered by **Iris**, an intelligent agent who actively explores your codebase to understand what you're building. Instead of dumping context and hoping for the best, Iris uses tools to gather precisely what she needs—analyzing diffs, exploring relationships, building genuine understanding.

This agent-first architecture means Git-Iris adapts to _your_ project—learning your conventions, matching your style, and capturing the intent behind every change. Iris crafts meaningful commit messages, generates insightful changelogs, creates detailed release notes, and provides thorough code reviews—all informed by real understanding of your code.

## What You'll Learn

This guide gets you running with Git-Iris in minutes:

- **[Installation](./installation.md)** — Install via Cargo, Docker, or manual build
- **[Quick Start](./quick-start.md)** — Generate your first AI commit in 60 seconds
- **[Configuration](./configuration.md)** — Set up API keys and customize your workflow

## What Makes Git-Iris Different?

### Agent-First Intelligence

Iris doesn't use templates. She **explores** your codebase:

| Traditional Tools        | Iris                                            |
| ------------------------ | ----------------------------------------------- |
| Dump all context upfront | Gathers precisely what she needs via tool calls |
| Static analysis          | Iterative exploration and understanding         |
| One-size-fits-all output | Adapts to your project's context                |

### Studio: Your Unified Interface

**Iris Studio** brings all capabilities together in one beautiful TUI:

- 🔭 **Explore** — Navigate code with AI-powered semantic blame
- 💫 **Commit** — Generate and refine commit messages
- 🔬 **Review** — Get comprehensive code reviews
- 📜 **PR** — Create pull request descriptions
- 🗂️ **Changelog** — Generate structured changelogs
- 🎊 **Release Notes** — Document releases with style

Press `/` in any mode to chat with Iris. Ask her to refine content, explain changes, or answer questions about your code.

### Multi-Provider Support

Work with your preferred LLM:

| Provider      | Default Model              | Context Window |
| ------------- | -------------------------- | -------------- |
| **OpenAI**    | gpt-5.1                    | 128K tokens    |
| **Anthropic** | claude-sonnet-4-5-20250929 | 200K tokens    |
| **Google**    | gemini-3-pro-preview       | 1M tokens      |

Switch providers instantly—configuration is shared across your system.

## What's Next?

Jump into [Installation](./installation.md) to get Git-Iris running, or skip straight to the [Quick Start](./quick-start.md) if you're ready to generate your first AI commit.
