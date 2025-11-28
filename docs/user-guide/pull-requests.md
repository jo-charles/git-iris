# Pull Request Descriptions

Iris generates comprehensive PR descriptions by analyzing commit ranges, branch differences, or individual commits as atomic units. She creates professional descriptions with summaries, detailed explanations, and testing notes.

## Quick Example

```bash
# Compare feature branch to main
git-iris pr --from main --to feature-branch

# Review last 3 commits
git-iris pr --from HEAD~3

# Single commit analysis
git-iris pr --from abc1234

# From main to current branch
git checkout feature-branch
git-iris pr --to feature-branch
```

## Command Reference

```bash
git-iris pr [FLAGS] [OPTIONS]
```

### Key Flags

| Flag           | Description                                    |
| -------------- | ---------------------------------------------- |
| `-p, --print`  | Print PR description to stdout and exit        |
| `--raw`        | Output raw markdown without console formatting |
| `--from <ref>` | Starting reference (commit, branch, commitish) |
| `--to <ref>`   | Target reference (commit, branch, commitish)   |

### Global Options

| Option                      | Description                  |
| --------------------------- | ---------------------------- |
| `--provider <name>`         | Override LLM provider        |
| `--preset <name>`           | Use instruction preset       |
| `-i, --instructions "text"` | Custom PR focus              |
| `--debug`                   | Show agent execution details |

## Usage Patterns

### Branch Comparison (Most Common)

Compare a feature branch against main:

```bash
# Full comparison
git-iris pr --from main --to feature-branch

# Auto-detect base (defaults to main)
git-iris pr --to feature-branch

# Custom base branch
git-iris pr --from develop --to feature-xyz
```

### Commit Range Analysis

Review multiple commits:

```bash
# Last 3 commits
git-iris pr --from HEAD~3

# Last 5 commits
git-iris pr --from HEAD~5

# Between two commits
git-iris pr --from abc1234 --to def5678
```

### Single Commit

Generate description for one commit:

```bash
# By hash
git-iris pr --from abc1234

# By commitish
git-iris pr --to HEAD~2

# Recent commit
git-iris pr --from HEAD~1
```

### Commitish Syntax

Supports Git's commitish syntax:

| Syntax         | Meaning                  |
| -------------- | ------------------------ |
| `HEAD~2`       | 2 commits before HEAD    |
| `HEAD^`        | Parent of HEAD           |
| `@~3`          | 3 commits before current |
| `main~1`       | 1 commit before main tip |
| `origin/main^` | Parent of remote main    |

Examples:

```bash
git-iris pr --from HEAD~3  # Last 3 commits
git-iris pr --from main~1 --to feature-branch
git-iris pr --to @~2  # 2 commits before current
```

## PR Description Format

Iris generates structured PR descriptions:

```markdown
## Summary

High-level overview of what this PR accomplishes and why.

## Changes

### Feature: Authentication System

- Implements JWT-based authentication with RS256 signing
- Adds refresh token rotation for enhanced security
- Includes automatic token expiry handling

### Infrastructure

- Updates Docker configuration for auth service
- Adds Redis for session management

## Technical Details

**Security Considerations:**

- Tokens signed with RS256 asymmetric encryption
- Refresh tokens rotated on each use
- Configurable expiry times via environment variables

**Performance Impact:**

- Redis caching reduces database queries by ~60%
- Token validation adds ~5ms to request latency

## Testing

- [ ] Unit tests for token generation and validation
- [ ] Integration tests for authentication flow
- [ ] Load testing for concurrent authentications
- [ ] Security audit of token handling

## Migration Notes

Breaking changes requiring action:

- Update `.env` with `JWT_SECRET` and `REFRESH_SECRET`
- Run `npm run migrate:auth` to create auth tables
```

## Customizing Descriptions

### Using Presets

```bash
# Concise PR focused on key points
git-iris pr --preset concise --from main --to feature-branch

# Detailed with comprehensive explanations
git-iris pr --preset detailed --from main --to feature-branch

# Technical deep dive
git-iris pr --preset technical --from main --to feature-branch
```

### Custom Instructions

```bash
# Emphasize architectural changes
git-iris pr --from main --to refactor-branch \
  --instructions "Focus on architectural improvements and design patterns"

# Security-focused PR
git-iris pr --from main --to security-fixes \
  --instructions "Emphasize security improvements and threat mitigation"

# Breaking changes
git-iris pr --from v1.0 --to v2.0 \
  --instructions "Highlight breaking changes and migration steps"
```

## Output Modes

### Interactive (Default)

Pretty-printed to console:

```bash
git-iris pr --from main --to feature-branch
```

### Print Mode

For piping or saving:

```bash
# Save to file
git-iris pr --from main --to feature-branch --print > pr-description.md

# Copy to clipboard (macOS)
git-iris pr --from main --to feature-branch --print | pbcopy

# Copy to clipboard (Linux)
git-iris pr --from main --to feature-branch --print | xclip -selection clipboard
```

### Raw Mode

Pure markdown without formatting:

```bash
# For CI/CD
git-iris pr --from main --to feature-branch --raw > pr.md

# For GitHub CLI
git-iris pr --from main --to feature-branch --raw | gh pr create --body-file -
```

## Integration Workflows

### GitHub CLI

Create PR with generated description:

```bash
# Generate description and create PR
git-iris pr --from main --to feature-branch --raw | \
  gh pr create --title "Add authentication system" --body-file -

# Or save and review first
git-iris pr --from main --to feature-branch --print > pr.md
gh pr create --title "Add authentication system" --body-file pr.md
```

### GitLab CLI

```bash
git-iris pr --from main --to feature-branch --raw | \
  glab mr create --title "Add authentication" --description-file -
```

### Git Alias

Add to `~/.gitconfig`:

```ini
[alias]
    pr-desc = !f() { git-iris pr --from ${1:-main} --to ${2:-HEAD} --print; }; f
    pr-create = !f() { git-iris pr --from main --to $(git branch --show-current) --raw | gh pr create --body-file -; }; f
```

Usage:

```bash
# Generate description
git pr-desc main feature-branch

# Create PR with description
git pr-create
```

## Tips

**For Feature PRs:**

```bash
git-iris pr --from main --to feature-branch --preset detailed
```

**For Hotfixes:**

```bash
git-iris pr --from main --to hotfix --preset concise \
  --instructions "Focus on the bug fix and impact"
```

**For Refactoring:**

```bash
git-iris pr --from main --to refactor --preset technical \
  --instructions "Emphasize code quality improvements and architectural changes"
```

**For Breaking Changes:**

```bash
git-iris pr --from v1.0 --to v2.0 --preset detailed \
  --instructions "Clearly highlight breaking changes and provide migration guide"
```

**For Large Changes:**

- Iris uses parallel subagent analysis for 20+ files
- Consider splitting into multiple PRs when possible
- Use `--preset concise` to keep descriptions focused

## Examples

```bash
# Standard feature PR
git-iris pr --from main --to add-auth-system

# Review last 3 commits before creating PR
git-iris pr --from HEAD~3 --print

# Detailed PR for complex change
git-iris pr --from develop --to major-refactor --preset detailed

# Quick PR for small fix
git-iris pr --from main --to fix-typo --preset concise

# Security fix with focus
git-iris pr --from main --to security-patch \
  --instructions "Focus on vulnerability details and mitigation"

# Create PR with GitHub CLI
git-iris pr --from main --to feature --raw | \
  gh pr create --title "Add feature" --body-file -

# Debug PR generation
git-iris pr --from main --to feature --debug --print
```

## Error Handling

**No Changes Found:**

```
⚠ No changes between main and feature-branch
→ Ensure branches have diverged
```

**Invalid Reference:**

```
✗ Invalid Git reference: 'nonexistent-branch'
→ Use 'git log' or 'git branch' to find valid references
```

**Default Branch Detection:**
If `--from` is omitted, Iris defaults to `main` or `master`:

```bash
# Automatically uses main as base
git-iris pr --to feature-branch
```

## Best Practices

1. **Review Before Creating PR:** Use `--print` to review the description first
2. **Customize for Audience:** Use presets and instructions to match your team's style
3. **Include Testing Notes:** Iris automatically generates testing checklists
4. **Highlight Breaking Changes:** Use custom instructions to emphasize migrations
5. **Keep PRs Focused:** Smaller PRs get better descriptions and faster reviews
