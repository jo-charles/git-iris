# Changelog Generation

Iris generates structured changelogs following the [Keep a Changelog](https://keepachangelog.com/) format between Git references. She analyzes commits, categorizes changes, and produces professional changelog entries.

## Quick Example

```bash
# Generate changelog between tags
git-iris changelog --from v1.0.0 --to v2.0.0

# From tag to HEAD
git-iris changelog --from v2.0.0

# Update CHANGELOG.md file
git-iris changelog --from v2.0.0 --update

# Custom changelog file
git-iris changelog --from v2.0.0 --update --file HISTORY.md
```

## Command Reference

```bash
git-iris changelog --from <ref> [FLAGS] [OPTIONS]
```

### Required Arguments

| Argument       | Description                       |
| -------------- | --------------------------------- |
| `--from <ref>` | Starting Git reference (required) |

### Key Flags

| Flag                    | Description                                       |
| ----------------------- | ------------------------------------------------- |
| `--to <ref>`            | Ending reference (defaults to HEAD)               |
| `--update`              | Update the changelog file with new changes        |
| `--file <path>`         | Path to changelog file (defaults to CHANGELOG.md) |
| `--version-name <name>` | Explicit version name instead of Git tag          |
| `--raw`                 | Output raw markdown without console formatting    |

### Global Options

| Option                      | Description                  |
| --------------------------- | ---------------------------- |
| `--provider <name>`         | Override LLM provider        |
| `--preset <name>`           | Use instruction preset       |
| `-i, --instructions "text"` | Custom changelog focus       |
| `--debug`                   | Show agent execution details |

## Usage Patterns

### Between Version Tags

Most common for releases:

```bash
# Generate changelog for new version
git-iris changelog --from v1.0.0 --to v2.0.0

# From previous tag to HEAD
git-iris changelog --from v1.0.0

# Multiple version jumps
git-iris changelog --from v0.9.0 --to v2.0.0
```

### Between Commits

For pre-release or development changelogs:

```bash
# Between commit hashes
git-iris changelog --from abc1234 --to def5678

# From commit to HEAD
git-iris changelog --from abc1234

# Last week's changes (using commitish)
git-iris changelog --from HEAD~20
```

### Between Branches

For feature branch summaries:

```bash
# Changes in feature branch
git-iris changelog --from main --to feature-branch

# Development branch summary
git-iris changelog --from main --to develop
```

## Changelog Format

Iris generates Keep a Changelog-formatted entries:

```markdown
## [2.0.0] - 2025-11-28

### Added

- JWT authentication system with RS256 signing
- Refresh token rotation for enhanced security
- Redis-based session management
- Rate limiting for authentication endpoints

### Changed

- Updated Docker configuration for microservices architecture
- Improved error handling in API middleware
- Enhanced logging with structured JSON format

### Fixed

- Security vulnerability in token validation
- Race condition in session cleanup
- Memory leak in WebSocket connections

### Removed

- Deprecated legacy authentication endpoints
- Unused database migrations from v1.x

### Security

- Fixed SQL injection vulnerability in user queries
- Updated dependencies to patch known CVEs
```

Categories automatically detected:

- **Added** - New features
- **Changed** - Changes to existing functionality
- **Deprecated** - Soon-to-be-removed features
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Security fixes

## File Management

### Update Existing Changelog

Iris prepends new changes to your CHANGELOG.md:

```bash
# Add new section to CHANGELOG.md
git-iris changelog --from v2.0.0 --update
```

Before:

```markdown
# Changelog

## [2.0.0] - 2025-10-01

...existing content...
```

After:

```markdown
# Changelog

## [2.1.0] - 2025-11-28

...new changes...

## [2.0.0] - 2025-10-01

...existing content...
```

### Custom Changelog File

```bash
# Update HISTORY.md instead
git-iris changelog --from v2.0.0 --update --file HISTORY.md

# Update docs/CHANGELOG.md
git-iris changelog --from v2.0.0 --update --file docs/CHANGELOG.md
```

### Custom Version Name

Override auto-detected version:

```bash
# Use custom version string
git-iris changelog --from v2.0.0 --version-name "3.0.0-beta.1" --update

# For unreleased changes
git-iris changelog --from v2.0.0 --version-name "Unreleased" --update
```

## Output Modes

### Interactive (Default)

Pretty-printed to console:

```bash
git-iris changelog --from v1.0.0 --to v2.0.0
```

### Raw Mode

Pure markdown without formatting:

```bash
# Save to file
git-iris changelog --from v1.0.0 --to v2.0.0 --raw > changes.md

# For processing
git-iris changelog --from v1.0.0 --raw | pandoc -f markdown -t html
```

### Update Mode

Directly update changelog file:

```bash
# Update and display
git-iris changelog --from v2.0.0 --update

# Update quietly
git-iris changelog --from v2.0.0 --update --quiet
```

## Customizing Changelogs

### Using Presets

```bash
# Concise changelog
git-iris changelog --from v1.0.0 --preset concise

# Detailed with explanations
git-iris changelog --from v1.0.0 --preset detailed

# Technical focus
git-iris changelog --from v1.0.0 --preset technical
```

### Custom Instructions

```bash
# User-facing changelog
git-iris changelog --from v1.0.0 \
  --instructions "Focus on user-visible changes, avoid internal details"

# Developer changelog
git-iris changelog --from v1.0.0 \
  --instructions "Include API changes, breaking changes, and migration notes"

# Security-focused
git-iris changelog --from v1.0.0 \
  --instructions "Emphasize security fixes and vulnerability patches"
```

## Integration Workflows

### Release Script

Add to `scripts/release.sh`:

```bash
#!/bin/bash
VERSION=$1

# Generate and update changelog
git-iris changelog --from $(git describe --tags --abbrev=0) \
  --version-name "$VERSION" --update --quiet

# Commit changelog
git add CHANGELOG.md
git commit -m "📝 Update changelog for $VERSION"

# Create tag
git tag -a "$VERSION" -m "Release $VERSION"
```

Usage:

```bash
./scripts/release.sh v2.1.0
```

### GitHub Actions

```yaml
name: Update Changelog
on:
  push:
    tags:
      - 'v*'

jobs:
  changelog:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install git-iris
        run: cargo install git-iris
      - name: Generate Changelog
        run: |
          git-iris changelog --from $(git describe --tags --abbrev=0 HEAD^) \
            --to ${{ github.ref_name }} --update --quiet
      - name: Commit Changelog
        run: |
          git config user.name "github-actions"
          git config user.email "github-actions@github.com"
          git add CHANGELOG.md
          git commit -m "📝 Update changelog for ${{ github.ref_name }}"
          git push
```

### Git Alias

Add to `~/.gitconfig`:

```ini
[alias]
    changelog = !f() { git-iris changelog --from ${1:-$(git describe --tags --abbrev=0)} --to ${2:-HEAD}; }; f
    changelog-update = !f() { git-iris changelog --from $(git describe --tags --abbrev=0) --update; }; f
```

Usage:

```bash
# Generate from last tag
git changelog

# Update CHANGELOG.md
git changelog-update
```

## Tips

**For Releases:**

```bash
# Generate and update in one step
git-iris changelog --from v2.0.0 --version-name v2.1.0 --update
```

**For Pre-Releases:**

```bash
# Track unreleased changes
git-iris changelog --from v2.0.0 --version-name "Unreleased" --update
```

**For Hotfixes:**

```bash
# Quick changelog for patch release
git-iris changelog --from v2.0.1 --to v2.0.2 --preset concise --update
```

**For Major Releases:**

```bash
# Detailed changelog for major version
git-iris changelog --from v1.0.0 --to v2.0.0 --preset detailed --update
```

## Examples

```bash
# Standard release changelog
git-iris changelog --from v2.0.0 --to v2.1.0

# Update changelog file
git-iris changelog --from v2.0.0 --update

# Custom version and file
git-iris changelog --from v2.0.0 --version-name "3.0.0-beta" \
  --file docs/CHANGELOG.md --update

# Concise changelog for patch
git-iris changelog --from v2.0.1 --to v2.0.2 --preset concise

# User-facing changelog
git-iris changelog --from v1.0.0 --to v2.0.0 \
  --instructions "Focus on features users will notice" --update

# Save to custom location
git-iris changelog --from v2.0.0 --raw > release-notes/v2.1.0.md

# Debug changelog generation
git-iris changelog --from v2.0.0 --debug
```

## Error Handling

**Invalid Reference:**

```
✗ Invalid Git reference: 'v99.0.0'
→ Use 'git tag' to list available tags
```

**No Changes Found:**

```
⚠ No changes between v2.0.0 and v2.0.1
→ Ensure references point to different commits
```

**File Write Error:**

```
✗ Failed to update changelog file: Permission denied
→ Check file permissions for CHANGELOG.md
```

## Best Practices

1. **Update During Release:** Make changelog updates part of your release process
2. **Use Semantic Versioning:** Tag releases with semver for clarity
3. **Keep Changelogs User-Focused:** Use instructions to emphasize user-facing changes
4. **Review Before Committing:** Check generated content before `--update`
5. **Maintain History:** Keep all changelog entries, don't overwrite
