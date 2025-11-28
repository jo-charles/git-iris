# Code Reviews

Iris performs comprehensive multi-dimensional code analysis across 10+ dimensions including complexity, security, performance, maintainability, and more.

## Quick Example

```bash
# Review staged changes
git-iris review

# Review specific commit
git-iris review --commit abc1234

# Review branch comparison
git-iris review --from main --to feature-branch

# Include unstaged changes
git-iris review --include-unstaged
```

## Command Reference

```bash
git-iris review [FLAGS] [OPTIONS]
```

### Key Flags

| Flag                 | Description                                            |
| -------------------- | ------------------------------------------------------ |
| `-p, --print`        | Print review to stdout and exit                        |
| `--raw`              | Output raw markdown without console formatting         |
| `--include-unstaged` | Include unstaged changes in review                     |
| `--commit <ref>`     | Review specific commit (hash, branch, or reference)    |
| `--from <ref>`       | Starting reference for comparison (defaults to `main`) |
| `--to <ref>`         | Target reference for comparison                        |

### Global Options

| Option                      | Description                  |
| --------------------------- | ---------------------------- |
| `--provider <name>`         | Override LLM provider        |
| `--preset <name>`           | Use instruction preset       |
| `-i, --instructions "text"` | Custom review focus          |
| `--debug`                   | Show agent execution details |

## Review Modes

### Staged Changes (Default)

Review what's currently staged:

```bash
git-iris review
```

Analyzes all staged changes as a cohesive unit.

### Include Unstaged

Review both staged and unstaged changes:

```bash
git-iris review --include-unstaged
```

Useful for pre-commit analysis of all working changes.

### Specific Commit

Review a single commit:

```bash
# By hash
git-iris review --commit abc1234

# By branch name
git-iris review --commit feature-branch

# By reference
git-iris review --commit HEAD~1
```

### Branch Comparison

Review differences between branches:

```bash
# Compare feature branch to main
git-iris review --from main --to feature-branch

# From main to current branch (auto-detects HEAD)
git-iris review --to feature-branch

# Custom base branch
git-iris review --from develop --to feature-xyz
```

## Review Dimensions

Iris analyzes code across these dimensions:

| Dimension          | Focus                                              |
| ------------------ | -------------------------------------------------- |
| **Complexity**     | Code clarity, readability, maintainability         |
| **Security**       | Vulnerabilities, unsafe patterns, input validation |
| **Performance**    | Efficiency, algorithmic complexity, resource usage |
| **Error Handling** | Edge cases, error propagation, recovery            |
| **Testing**        | Test coverage, testability, quality                |
| **Documentation**  | Code comments, API docs, clarity                   |
| **Architecture**   | Design patterns, separation of concerns            |
| **Dependencies**   | External libraries, version management             |
| **Code Style**     | Consistency, conventions, formatting               |
| **Best Practices** | Language idioms, community standards               |

## Output Format

Reviews are formatted as markdown with sections for each relevant dimension:

```markdown
# Code Review

## Summary

High-level overview of changes and overall assessment.

## Security

- ✓ Input validation added for user-supplied data
- ⚠ Consider rate limiting for authentication endpoint
- ✗ Hardcoded secret detected in config file

## Performance

- ✓ Efficient algorithm with O(n log n) complexity
- → Database queries could benefit from indexing

## Recommendations

1. Add integration tests for authentication flow
2. Extract configuration to environment variables
3. Consider caching for frequently accessed data
```

Symbols:

- `✓` Positive findings
- `⚠` Warnings/considerations
- `✗` Critical issues
- `→` Suggestions

## Customizing Reviews

### Using Presets

```bash
# Concise review focusing on critical issues
git-iris review --preset concise

# Detailed analysis with explanations
git-iris review --preset detailed

# Technical deep dive
git-iris review --preset technical
```

### Custom Instructions

```bash
# Security-focused review
git-iris review --instructions "Focus on security vulnerabilities and authentication"

# Performance review
git-iris review --instructions "Analyze performance impacts and database queries"

# Architecture review
git-iris review --instructions "Evaluate design patterns and code organization"
```

## Output Modes

### Interactive (Default)

Pretty-printed to console with syntax highlighting:

```bash
git-iris review
```

### Print Mode

Clean output for piping:

```bash
# Save to file
git-iris review --print > review.md

# Pipe to pager
git-iris review --print | less
```

### Raw Mode

Pure markdown without ANSI formatting:

```bash
# For CI/CD pipelines
git-iris review --raw > review.md

# For markdown processors
git-iris review --raw | pandoc -f markdown -t html
```

## Integration Workflows

### Pre-Commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
git-iris review --print --quiet
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: AI Code Review
  run: |
    git-iris review --from ${{ github.base_ref }} --to ${{ github.head_ref }} --raw > review.md
    gh pr comment --body-file review.md
```

### Git Alias

Add to `~/.gitconfig`:

```ini
[alias]
    ai-review = !git-iris review
    review-commit = !git-iris review --commit
```

Usage:

```bash
git ai-review
git review-commit abc1234
```

## Tips

**For Large Changes:**

- Iris uses parallel subagent analysis for 20+ files
- Break large reviews into smaller chunks when possible
- Use `--from` and `--to` to review specific ranges

**For Security Focus:**

```bash
git-iris review --instructions "Deep security audit focusing on authentication, authorization, and data validation"
```

**For Performance Analysis:**

```bash
git-iris review --preset technical --instructions "Focus on performance bottlenecks and optimization opportunities"
```

**For Quick Checks:**

```bash
git-iris review --preset concise --print
```

## Examples

```bash
# Review staged changes with detailed analysis
git-iris review --preset detailed

# Security-focused review of branch
git-iris review --from main --to security-fixes --instructions "Focus on security"

# Quick review of last commit
git-iris review --commit HEAD~1 --preset concise --print

# Review PR changes
git-iris review --from origin/main --to feature-branch --raw

# Include unstaged for complete analysis
git-iris review --include-unstaged --preset detailed

# Debug agent execution
git-iris review --debug
```

## Error Handling

**No Changes to Review:**

```
⚠ No changes found to review
→ Stage changes with 'git add' or specify a commit/range
```

**Invalid Reference:**

```
✗ Invalid Git reference: 'nonexistent-branch'
→ Use 'git log' to find valid commits/branches
```

**Conflicting Options:**

```
✗ Cannot use --commit with --from/--to
→ Use either --commit for single commit or --from/--to for ranges
```
