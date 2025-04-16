# Git-Iris MCP Integration

Git-Iris supports the Model Context Protocol (MCP), allowing it to be used directly from compatible editors and AI assistants.

## What is MCP?

The Model Context Protocol (MCP) is an open standard that enables AI assistants to communicate with external tools and services in a standardized way. By supporting MCP, Git-Iris can be seamlessly used from any MCP-compatible client, including:

- Claude Desktop
- Cursor
- Visual Studio Code (with appropriate extensions)
- Zed
- Continue
- Many other editors and AI assistants

## Using Git-Iris as an MCP Server

To start Git-Iris as an MCP server, use the `serve` command:

```bash
# Start with stdio transport (default)
git-iris serve

# Start with development mode for more verbose logging
git-iris serve --dev

# Start with SSE transport on specific port
git-iris serve --transport sse --port 3077 --listen-address 127.0.0.1
```

## Available Tools

Git-Iris exposes the following tools through MCP:

### `git_iris_commit`

Generate commit messages and perform Git commits.

**Parameters:**

- `auto_commit`: (boolean) Whether to generate and perform the commit (true) or just generate a message (false)
- `use_gitmoji`: (boolean) Whether to use gitmoji in commit messages
- `no_verify`: (boolean) Skip verification for commit action
- `preset`: (string) Instruction preset to use
- `custom_instructions`: (string) Custom instructions for the AI
- `repository`: (string, required) Repository path (local) or URL (remote). **Required.**

Example (local path):

```json
{
  "auto_commit": true,
  "use_gitmoji": true,
  "preset": "conventional",
  "custom_instructions": "Include the ticket number from JIRA",
  "repository": "/home/bliss/dev/myproject"
}
```

Example (remote URL):

```json
{
  "auto_commit": true,
  "use_gitmoji": true,
  "preset": "conventional",
  "custom_instructions": "Include the ticket number from JIRA",
  "repository": "https://github.com/example/repo.git"
}
```

### `git_iris_code_review`

Generate comprehensive code reviews with options for staged changes, unstaged changes, or specific commits.

**Parameters:**

- `include_unstaged`: (boolean) Include unstaged changes in the review
- `commit_id`: (string) Specific commit to review (hash, branch name, or reference)
- `preset`: (string) Preset instruction set to use for the review
- `custom_instructions`: (string) Custom instructions for the AI
- `repository`: (string, required) Repository path (local) or URL (remote). **Required.**

Example (local path):

```json
{
  "preset": "security",
  "custom_instructions": "Focus on performance issues",
  "include_unstaged": true,
  "repository": "/home/bliss/dev/myproject"
}
```

Example (remote URL):

```json
{
  "preset": "security",
  "custom_instructions": "Focus on performance issues",
  "include_unstaged": true,
  "repository": "https://github.com/example/repo.git"
}
```

### `git_iris_changelog`

Generate a detailed changelog between two Git references.

**Parameters:**

- `from`: (string, required) Starting reference (commit hash, tag, or branch name)
- `to`: (string) Ending reference (defaults to HEAD if not specified)
- `detail_level`: (string) Level of detail for the changelog (minimal, standard, detailed)
- `custom_instructions`: (string) Custom instructions for the AI
- `repository`: (string, required) Repository path (local) or URL (remote). **Required.**
- `version_name`: (string) Explicit version name to use in the changelog instead of getting it from Git

Example (local path):

```json
{
  "from": "v1.0.0",
  "to": "v2.0.0",
  "detail_level": "detailed",
  "custom_instructions": "Group changes by component",
  "repository": "/home/bliss/dev/myproject"
}
```

Example with version name (useful in release workflows):

```json
{
  "from": "v1.0.0",
  "to": "HEAD",
  "detail_level": "detailed",
  "version_name": "v2.0.0-beta",
  "repository": "/home/bliss/dev/myproject"
}
```

### `git_iris_release_notes`

Generate comprehensive release notes between two Git references.

**Parameters:**

- `from`: (string, required) Starting reference (commit hash, tag, or branch name)
- `to`: (string) Ending reference (defaults to HEAD if not specified)
- `detail_level`: (string) Level of detail for the release notes (minimal, standard, detailed)
- `custom_instructions`: (string) Custom instructions for the AI
- `repository`: (string, required) Repository path (local) or URL (remote). **Required.**
- `version_name`: (string) Explicit version name to use in the release notes instead of getting it from Git

Example (local path):

```json
{
  "from": "v1.0.0",
  "to": "v2.0.0",
  "detail_level": "standard",
  "custom_instructions": "Highlight breaking changes",
  "repository": "/home/bliss/dev/myproject"
}
```

Example with version name:

```json
{
  "from": "v1.0.0",
  "to": "HEAD",
  "detail_level": "standard",
  "version_name": "v2.0.0-rc1",
  "repository": "/home/bliss/dev/myproject"
}
```

## Why is `repository` required?

Due to limitations in some MCP clients (such as Cursor and others), the server cannot reliably infer the project root or repository path. To ensure Git-Iris always operates on the correct repository, you must explicitly specify the `repository` parameter for every tool call. This can be either a local filesystem path (for local projects) or a remote repository URL (for remote operations). This eliminates ambiguity and ensures your commands are always precise and predictable.

## Using Git-Iris with Claude

### Claude Desktop Integration

1. Start Git-Iris as an MCP server: `git-iris serve`
2. Open Claude Desktop
3. Add Git-Iris as an MCP server through Claude's settings
4. You can now use Git-Iris through Claude using the tools described above

### Cursor Integration

When using Cursor with Claude, Git-Iris tools are automatically available as long as the server is running.

## Using Git-Iris with Visual Studio Code

1. Start Git-Iris as an MCP server: `git-iris serve --transport sse --port 3077`
2. Install an MCP-compatible extension in VS Code
3. Configure the extension to connect to Git-Iris at `http://localhost:3077`
4. Use the tools through the extension's interface

## Technical Implementation

The MCP integration uses the `rmcp` crate to implement the MCP server. The implementation is structured as follows:

- `src/mcp/mod.rs`: The main module for MCP-related functionality
- `src/mcp/server.rs`: Server implementation for handling MCP requests
- `src/mcp/config.rs`: Configuration for the MCP server
- `src/mcp/tools/`: Tool implementations for exposing Git-Iris features
  - `changelog.rs`: Changelog generation tool
  - `releasenotes.rs`: Release notes generation tool
  - `commit.rs`: Commit message generation and execution tool
  - `codereview.rs`: Code review tool
  - `utils.rs`: Shared utilities for MCP tools
  - `mod.rs`: Tool registration and handling

Currently, the stdio and SSE transports are fully implemented.

## Future Improvements

Planned improvements for the MCP integration include:

1. Additional tools for repository analysis and pull requests
2. Enhanced security and authentication features
3. Expanded integration with more MCP clients
4. Support for remote operation through SSH or other protocols
