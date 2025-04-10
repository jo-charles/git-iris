# Git-Iris MCP Integration

Git-Iris now supports the Model Context Protocol (MCP), allowing it to be used directly from compatible editors and AI assistants.

## What is MCP?

The Model Context Protocol (MCP) is an open standard that enables AI assistants to communicate with external tools and services in a standardized way. By supporting MCP, Git-Iris can be seamlessly used from any MCP-compatible client, including:

- Claude Desktop
- Cursor
- VSCode (with appropriate extensions)
- Zed
- Continue
- Many other editors and AI assistants

## Using Git-Iris as an MCP Server

To start Git-Iris as an MCP server, use the new `serve` command:

```bash
# Start with stdio transport (default)
git-iris serve

# Start with development mode for more verbose logging
git-iris serve --dev

# In the future, additional transports will be available:
# git-iris serve --transport sse --port 3077
# git-iris serve --transport websocket --port 3078
```

## Available Tools

Git-Iris exposes the following tools through MCP:

### `git_iris_commit`

Generate commit messages, analyze staged changes, and perform commits.

**Parameters:**
- `action`: "generate" (default), "analyze", or "commit"
- `use_gitmoji`: Whether to use Gitmojis in the message (boolean)
- `custom_instructions`: Custom instructions for the AI
- `preset`: Instruction preset to use
- `no_verify`: Skip verification for commit action (boolean)

### `git_iris_review`

Generate comprehensive code reviews for staged changes.

**Parameters:**
- `target`: "staged" (default), "pr", or "branch"  
- `target_id`: Target ID for PR or branch name
- `detail_level`: "minimal", "standard" (default), or "detailed"
- `dimension`: Specific quality dimension to focus on
- `custom_instructions`: Custom instructions for the AI

### `git_iris_changelog`

Generate changelogs and release notes between two Git references.

**Parameters:**
- `from`: Starting reference (commit hash, tag, or branch name)
- `to`: Ending reference (defaults to HEAD)
- `type`: "changelog" (default) or "release_notes"
- `detail_level`: "minimal", "standard" (default), or "detailed"
- `custom_instructions`: Custom instructions for the AI

### `git_iris_analyze`

Analyze repository content and provide insights.

**Parameters:**
- `action`: "repo_summary" (default), "explain_commit", "suggest_tasks"
- `target`: Target for analysis (e.g., commit hash)
- `detail_level`: "minimal", "standard" (default), or "detailed"
- `custom_instructions`: Custom instructions for the AI

## Using Git-Iris with Claude Desktop

1. Start Git-Iris as an MCP server: `git-iris serve`
2. Open Claude Desktop
3. Add Git-Iris as an MCP server through Claude's settings
4. You can now use Git-Iris through Claude using the tools described above

## Technical Implementation

The MCP integration uses the `mcp_daemon` crate to implement the MCP server. The implementation is structured as follows:

- `src/mcp/mod.rs`: The main module for MCP-related functionality
- `src/mcp/server.rs`: Server implementation for handling MCP requests
- `src/mcp/config.rs`: Configuration for the MCP server
- `src/mcp/tools/`: Tool implementations for exposing Git-Iris features
- `src/mcp/transports.rs`: Transport implementations for different communication modes

Currently, only the stdio transport is fully implemented, with SSE and WebSocket support planned for future releases.

## Future Improvements

Planned improvements for the MCP integration include:

1. Full implementation of all tool functions
2. Support for SSE and WebSocket transports
3. Additional resource sharing capabilities
4. Integration with more MCP clients
5. Security and authentication features 