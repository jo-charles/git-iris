# Contributing to Git-Iris

Ready to contribute your extension back to Git-Iris? This guide covers development setup, coding standards, testing requirements, and the PR process.

## Quick Start

```bash
# Fork the repository on GitHub
git clone https://github.com/YOUR_USERNAME/git-iris.git
cd git-iris

# Create a feature branch
git checkout -b feature/my-extension

# Make your changes
# ...

# Test your changes
cargo test
cargo clippy
cargo fmt

# Commit and push
git add .
git commit -m "Add feature: my extension"
git push origin feature/my-extension

# Open a pull request on GitHub
```

## Development Setup

### Prerequisites

- **Rust**: 1.75 or later (`rustup update`)
- **Git**: 2.30 or later
- **LLM Provider**: At least one API key (OpenAI, Anthropic, or Google)

### Environment Setup

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone your fork
git clone https://github.com/YOUR_USERNAME/git-iris.git
cd git-iris

# Build
cargo build

# Set up API key for testing
export ANTHROPIC_API_KEY=sk-ant-...
# or
export OPENAI_API_KEY=sk-...

# Run tests
cargo test

# Try it out
cargo run -- gen
cargo run -- studio
```

### Development Workflow

```bash
# Create feature branch from main
git checkout main
git pull upstream main
git checkout -b feature/my-feature

# Make changes, test frequently
cargo build
cargo test
cargo run -- gen --debug

# Check code quality
cargo clippy
cargo fmt

# Commit with descriptive messages
git add .
git commit -m "Add X: brief description

Detailed explanation of what changed and why."

# Push and create PR
git push origin feature/my-feature
```

## Coding Standards

### Rust Style

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Fix common issues
cargo clippy --fix
```

### Code Organization

**Keep modules focused:**

```rust
// Good - clear separation
mod tools;
mod capabilities;
mod state;

// Bad - mixed concerns
mod stuff;
mod utils;
```

**Use clear names:**

```rust
// Good
pub struct GitDiff;
pub fn handle_commit_key(...) -> Vec<SideEffect>;

// Bad
pub struct Helper;
pub fn process(...) -> Vec<Thing>;
```

**Document public APIs:**

```rust
/// Analyzes project dependencies from package manifests.
///
/// Supports Cargo.toml, package.json, and requirements.txt.
/// Auto-detects manifest type if not specified.
pub struct DependencyAnalyzer;
```

### Error Handling

**Use descriptive errors:**

```rust
// Good
return Err(anyhow::anyhow!(
    "Failed to read Cargo.toml: file not found in {}",
    path.display()
));

// Bad
return Err(anyhow::anyhow!("error"));
```

**Use error context:**

```rust
use anyhow::Context;

let content = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read file: {}", path.display()))?;
```

### Tool Development Standards

**Clear tool descriptions:**

```rust
async fn definition(&self, _: String) -> ToolDefinition {
    ToolDefinition {
        name: "my_tool".to_string(),
        // Good - actionable, specific
        description: "Analyze project dependencies from package manifests (Cargo.toml, package.json, requirements.txt)".to_string(),
        parameters: parameters_schema::<MyToolArgs>(),
    }
}
```

**Structured output:**

```rust
// Good - organized, parseable
Ok(format!(
    "## Dependencies\n{}\n\n## Dev Dependencies\n{}\n",
    deps, dev_deps
))

// Bad - unstructured
Ok(format!("{} {}", deps, dev_deps))
```

**Reasonable defaults:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MyToolArgs {
    pub query: String,  // Required

    #[serde(default = "default_limit")]
    pub limit: usize,  // Optional with default

    #[serde(default)]
    pub verbose: bool,  // Optional, defaults to false
}

fn default_limit() -> usize { 10 }
```

### Capability Development Standards

**Explicit workflow steps:**

```toml
## Workflow
1. Call `project_docs(doc_type="context")` to understand the project
2. Get changes with `git_diff(detail="summary")`
3. Analyze key files with `file_analyzer()`
4. Synthesize findings into structured output
```

**Clear output requirements:**

```toml
## Output Requirements
- **Field1**: Description, constraints
- **Field2**: Description, format
- Use definitive language, not "probably" or "might"
```

**Context strategies:**

```toml
## Context Strategy by Size
- **Small**: Consider all files
- **Medium**: Focus on high-relevance files
- **Large**: Use top 5-7 files, summarize rest
- **Very Large**: Use `parallel_analyze` to distribute work
```

### Studio Mode Standards

**Pure reducer pattern:**

```rust
// Good - pure function
pub fn reduce(state: StudioState, event: StudioEvent) -> (StudioState, Vec<SideEffect>) {
    let mut state = state;
    let effects = match event {
        StudioEvent::KeyPress(key) => handle_key(&mut state, key),
        // ...
    };
    (state, effects)
}

// Bad - side effects in reducer
pub fn reduce(state: &mut StudioState, event: StudioEvent) {
    tokio::spawn(async { ... });  // Don't do this!
}
```

**Focused handlers:**

```rust
// Good - clear responsibility
fn handle_file_list_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect>
fn handle_content_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect>

// Bad - monolithic
fn handle_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // 500 lines of match arms...
}
```

## Testing Requirements

### Unit Tests

Every tool and capability should have tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dependency_analyzer_cargo() {
        let tool = DependencyAnalyzer;
        let args = DependencyAnalyzerArgs {
            manifest_type: Some("cargo".to_string()),
            include_dev: false,
        };

        let result = tool.call(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dependency_analyzer_auto_detect() {
        let tool = DependencyAnalyzer;
        let args = DependencyAnalyzerArgs {
            manifest_type: None,
            include_dev: true,
        };

        let result = tool.call(args).await;
        // Should auto-detect and succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_manifest_type() {
        let path = PathBuf::from("./");
        let result = detect_manifest_type(&path);
        // Project has Cargo.toml
        assert_eq!(result.unwrap(), "cargo");
    }
}
```

### Integration Tests

For modes and end-to-end flows:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_commit_generation_flow() {
        // Set up test repo
        let temp_dir = tempdir::TempDir::new("test_repo").unwrap();
        // ... create test commits ...

        // Generate commit message
        let service = IrisAgentService::new(test_config())?;
        let response = service.execute_capability("commit", &[]).await?;

        // Verify output
        assert!(matches!(response, StructuredResponse::CommitMessage(_)));
    }
}
```

### Manual Testing

Before submitting PR:

- [ ] Build succeeds: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] Clippy passes: `cargo clippy`
- [ ] Format applied: `cargo fmt`
- [ ] Feature works in CLI: `cargo run -- gen`
- [ ] Feature works in Studio: `cargo run -- studio`
- [ ] Debug mode works: `cargo run -- gen --debug`

## Pull Request Process

### Before Opening PR

1. **Rebase on latest main:**

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure tests pass:**

   ```bash
   cargo test
   cargo clippy
   ```

3. **Clean commit history:**
   ```bash
   # Squash commits if needed
   git rebase -i HEAD~3
   ```

### PR Title and Description

**Good PR title:**

```
Add dependency analyzer tool
```

**Bad PR title:**

```
feat: add some new stuff
```

**Good PR description:**

```markdown
## Summary

Adds a new tool that analyzes project dependencies from package manifests.

## Changes

- New `DependencyAnalyzer` tool in `src/agents/tools/dependency_analyzer.rs`
- Supports Cargo.toml, package.json, and requirements.txt
- Auto-detects manifest type
- Unit tests for all supported formats

## Testing

- [x] Tested with Rust project (Cargo.toml)
- [x] Tested with Node.js project (package.json)
- [x] Tested with Python project (requirements.txt)
- [x] Auto-detection works correctly
- [x] All tests pass

## Documentation

- Added tool documentation in extending/tools.md example

## Related Issues

Closes #123
```

### PR Checklist

Before requesting review:

- [ ] Code follows style guidelines
- [ ] Tests added for new functionality
- [ ] Documentation updated (if needed)
- [ ] No breaking changes (or clearly documented)
- [ ] Commit messages are descriptive
- [ ] PR description explains what and why

### Review Process

1. **Automated checks**: CI will run tests and linting
2. **Code review**: Maintainers review your code
3. **Feedback**: Address review comments
4. **Approval**: Once approved, maintainers will merge

**Responding to feedback:**

```bash
# Make requested changes
git add .
git commit -m "Address review feedback: improve error messages"

# Push updates
git push origin feature/my-feature
```

## Commit Message Guidelines

### Format

```
<type>: <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **refactor**: Code refactoring
- **docs**: Documentation changes
- **test**: Test additions/changes
- **chore**: Build/tooling changes

### Examples

**Good commit message:**

```
feat: Add dependency analyzer tool

Implements a new tool that analyzes project dependencies from
package manifests (Cargo.toml, package.json, requirements.txt).

The tool auto-detects the manifest type and supports filtering
dev dependencies.
```

**Concise commit:**

```
fix: Handle empty git status correctly

Fixes panic when running in repository with no changes.
```

**Breaking change:**

```
refactor: Update Tool trait to async

BREAKING CHANGE: All tools must now implement async `call()` method.
Existing tool implementations need to be updated.
```

## Documentation Requirements

### When to Update Docs

Update documentation when you:

- Add a new capability
- Add a new tool
- Add a new Studio mode
- Change public APIs
- Add new configuration options

### Where to Document

- **User-facing features**: `README.md`
- **Developer features**: `CLAUDE.md` (Developer Guide)
- **Extension guides**: `docs/extending/*.md`
- **API docs**: Inline doc comments (`///`)

### Documentation Style

**Good API docs:**

````rust
/// Analyzes project dependencies from package manifests.
///
/// Supports Cargo.toml, package.json, and requirements.txt.
/// Auto-detects manifest type if not specified.
///
/// # Examples
///
/// ```
/// let tool = DependencyAnalyzer;
/// let args = DependencyAnalyzerArgs {
///     manifest_type: None,  // Auto-detect
///     include_dev: true,
/// };
/// let result = tool.call(args).await?;
/// ```
pub struct DependencyAnalyzer;
````

**Good guide content:**

- Start with what the feature does
- Show step-by-step examples
- Link to real code
- Explain the why, not just the what

## Common Issues

### Build Fails

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Tests Fail

```bash
# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

### Clippy Warnings

```bash
# Auto-fix where possible
cargo clippy --fix

# See all warnings
cargo clippy -- -W clippy::pedantic
```

## Getting Help

- **Questions about architecture**: Read `CLAUDE.md`
- **Extension guides**: Check `docs/extending/`
- **API questions**: Check inline docs and examples
- **Stuck on implementation**: Open a draft PR and ask for guidance
- **Found a bug**: Open an issue with reproduction steps

## Recognition

Contributors are recognized in:

- GitHub Contributors page
- Release notes for features you contribute
- `CONTRIBUTORS.md` (coming soon)

## Code of Conduct

- Be respectful and constructive
- Focus on the code, not the person
- Accept feedback gracefully
- Help others learn

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (check `LICENSE` file).

## Next Steps

Ready to contribute?

1. Pick an issue labeled `good first issue` or `help wanted`
2. Comment that you're working on it
3. Follow this guide to implement and test
4. Open a PR
5. Respond to feedback
6. Celebrate when it's merged

**Let's build something powerful together.** ⚡

## Additional Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Rig Framework**: https://docs.rs/rig-core
- **Ratatui TUI**: https://ratatui.rs/
- **Git-Iris Discussions**: GitHub Discussions tab

**Welcome to the Git-Iris community.**
