# Release Notes Generation

Iris generates comprehensive release documentation between Git references, providing user-facing highlights, technical details, upgrade instructions, and known issues.

## Quick Example

```bash
# Generate release notes between tags
git-iris release-notes --from v1.0.0 --to v2.0.0

# From tag to HEAD
git-iris release-notes --from v2.0.0

# Custom version name
git-iris release-notes --from v2.0.0 --version-name "3.0.0-beta.1"

# Raw markdown output
git-iris release-notes --from v2.0.0 --raw > RELEASE_NOTES.md
```

## Command Reference

```bash
git-iris release-notes --from <ref> [FLAGS] [OPTIONS]
```

### Required Arguments

| Argument       | Description                       |
| -------------- | --------------------------------- |
| `--from <ref>` | Starting Git reference (required) |

### Key Flags

| Flag                    | Description                                    |
| ----------------------- | ---------------------------------------------- |
| `--to <ref>`            | Ending reference (defaults to HEAD)            |
| `--version-name <name>` | Explicit version name instead of Git tag       |
| `--raw`                 | Output raw markdown without console formatting |

### Global Options

| Option                      | Description                  |
| --------------------------- | ---------------------------- |
| `--provider <name>`         | Override LLM provider        |
| `--preset <name>`           | Use instruction preset       |
| `-i, --instructions "text"` | Custom release notes focus   |
| `--debug`                   | Show agent execution details |

## Usage Patterns

### Between Version Tags

Most common for official releases:

```bash
# Full release notes
git-iris release-notes --from v1.0.0 --to v2.0.0

# From previous tag to HEAD
git-iris release-notes --from v1.0.0

# Major version jump
git-iris release-notes --from v1.5.0 --to v2.0.0
```

### Pre-Release Notes

For beta, RC, or development releases:

```bash
# Beta release
git-iris release-notes --from v2.0.0 --version-name "3.0.0-beta.1"

# Release candidate
git-iris release-notes --from v2.0.0 --version-name "3.0.0-rc.1"

# Development snapshot
git-iris release-notes --from v2.0.0 --version-name "3.0.0-dev"
```

### Between Branches

For feature branch release summaries:

```bash
# Feature branch release
git-iris release-notes --from main --to feature-branch

# Development branch summary
git-iris release-notes --from stable --to develop
```

## Release Notes Format

Iris generates comprehensive release documentation:

````markdown
# Release v2.0.0

**Release Date:** 2025-11-28

## Overview

This major release introduces a complete authentication system, microservices
architecture, and significant performance improvements. Breaking changes require
migration steps outlined below.

## Highlights

✨ **New Authentication System**

- JWT-based authentication with RS256 signing
- Refresh token rotation for enhanced security
- Redis-backed session management
- Rate limiting on authentication endpoints

🚀 **Performance Improvements**

- 60% reduction in database queries through Redis caching
- Optimized WebSocket connection handling
- Reduced API latency by ~40ms on average

🏗️ **Architecture Updates**

- Migrated to microservices architecture
- Docker Compose for local development
- Kubernetes manifests for production deployment

## What's New

### Features

- JWT authentication system with asymmetric encryption
- User session management with Redis
- API rate limiting middleware
- WebSocket real-time notifications
- Admin dashboard with analytics

### Improvements

- Enhanced error handling across all services
- Structured JSON logging
- Improved API documentation with OpenAPI specs
- Better test coverage (now at 85%)

## Breaking Changes

⚠️ **Authentication Required**
All API endpoints now require authentication. Update clients to include
JWT tokens in the Authorization header.

⚠️ **Configuration Changes**
Environment variables restructured. See Migration Guide below.

⚠️ **Removed Endpoints**

- `POST /auth/legacy-login` - Use `/auth/login` instead
- `GET /users/all` - Use paginated `/users` instead

## Migration Guide

### 1. Update Environment Variables

```bash
# Old
AUTH_SECRET=your-secret

# New
JWT_SECRET=your-jwt-secret
REFRESH_SECRET=your-refresh-secret
REDIS_URL=redis://localhost:6379
```
````

### 2. Run Database Migrations

```bash
npm run migrate:auth
npm run migrate:users
```

### 3. Update Client Applications

Add JWT token to API requests:

```javascript
headers: {
  'Authorization': `Bearer ${token}`
}
```

## Installation

### Via Cargo

```bash
cargo install git-iris@2.0.0
```

### Via Homebrew

```bash
brew install git-iris
```

### Docker

```bash
docker pull ghcr.io/user/git-iris:2.0.0
```

## Known Issues

- WebSocket connections may timeout on slow networks (#123)
- Rate limiting is not yet configurable per-endpoint (#145)
- Session cleanup requires manual Redis flush in some cases (#156)

## Contributors

Special thanks to all contributors who made this release possible!

## Full Changelog

See CHANGELOG.md in your project root for detailed change list.

````

## Output Modes

### Interactive (Default)

Pretty-printed to console with syntax highlighting:

```bash
git-iris release-notes --from v1.0.0 --to v2.0.0
````

### Raw Mode

Pure markdown without formatting:

```bash
# Save to file
git-iris release-notes --from v1.0.0 --to v2.0.0 --raw > RELEASE_NOTES.md

# For GitHub releases
git-iris release-notes --from v1.0.0 --raw | gh release create v2.0.0 --notes-file -
```

## Customizing Release Notes

### Using Presets

```bash
# Concise release notes
git-iris release-notes --from v1.0.0 --preset concise

# Detailed with explanations
git-iris release-notes --from v1.0.0 --preset detailed

# Technical deep dive
git-iris release-notes --from v1.0.0 --preset technical
```

### Custom Instructions

```bash
# User-facing release notes
git-iris release-notes --from v1.0.0 \
  --instructions "Focus on user-visible features and benefits, minimal technical details"

# Developer-focused notes
git-iris release-notes --from v1.0.0 \
  --instructions "Include API changes, breaking changes, and migration steps"

# Marketing-style notes
git-iris release-notes --from v1.0.0 \
  --instructions "Emphasize benefits and improvements in an engaging way"

# Security release notes
git-iris release-notes --from v1.0.0 \
  --instructions "Focus on security fixes and vulnerability patches"
```

## Integration Workflows

### GitHub Releases

```bash
# Create GitHub release with notes
git-iris release-notes --from v1.0.0 --to v2.0.0 --raw | \
  gh release create v2.0.0 --title "Release v2.0.0" --notes-file -

# With binary attachments
git-iris release-notes --from v1.0.0 --raw > notes.md
gh release create v2.0.0 --notes-file notes.md --attach ./dist/*
```

### GitLab Releases

```bash
# Create GitLab release
git-iris release-notes --from v1.0.0 --raw > release_notes.md
glab release create v2.0.0 --notes-file release_notes.md
```

### Release Script

Create `scripts/release.sh`:

```bash
#!/bin/bash
set -e

VERSION=$1
PREV_TAG=$(git describe --tags --abbrev=0)

# Generate release notes
git-iris release-notes --from "$PREV_TAG" --to HEAD \
  --version-name "$VERSION" --raw > RELEASE_NOTES.md

# Create git tag
git tag -a "$VERSION" -m "Release $VERSION"

# Push tag
git push origin "$VERSION"

# Create GitHub release
gh release create "$VERSION" --title "Release $VERSION" \
  --notes-file RELEASE_NOTES.md

echo "✨ Released $VERSION"
```

Usage:

```bash
./scripts/release.sh v2.1.0
```

### GitHub Actions

```yaml
name: Create Release
on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0 # Need full history

      - name: Install git-iris
        run: cargo install git-iris

      - name: Generate Release Notes
        id: notes
        run: |
          PREV_TAG=$(git describe --tags --abbrev=0 HEAD^)
          git-iris release-notes --from "$PREV_TAG" \
            --to ${{ github.ref_name }} --raw > release_notes.md

      - name: Create Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref_name }}
          release_name: Release ${{ github.ref_name }}
          body_path: release_notes.md
          draft: false
          prerelease: false
```

## Tips

**For Major Releases:**

```bash
# Detailed release notes with migration guide
git-iris release-notes --from v1.0.0 --to v2.0.0 --preset detailed \
  --instructions "Include comprehensive migration guide for breaking changes"
```

**For Minor Releases:**

```bash
# Standard release notes
git-iris release-notes --from v2.0.0 --to v2.1.0
```

**For Patch Releases:**

```bash
# Concise notes focusing on fixes
git-iris release-notes --from v2.1.0 --to v2.1.1 --preset concise \
  --instructions "Focus on bug fixes and security patches"
```

**For Pre-Releases:**

```bash
# Beta release notes
git-iris release-notes --from v2.0.0 --version-name "3.0.0-beta.1" \
  --instructions "Note that this is a beta release with known issues"
```

**For Security Releases:**

```bash
# Security-focused release notes
git-iris release-notes --from v2.1.0 --to v2.1.1 \
  --instructions "Emphasize security fixes and recommend immediate upgrade"
```

## Examples

```bash
# Standard release
git-iris release-notes --from v2.0.0 --to v2.1.0

# Save to file
git-iris release-notes --from v2.0.0 --raw > RELEASE_NOTES.md

# Beta release
git-iris release-notes --from v2.0.0 --version-name "3.0.0-beta.1"

# Detailed release with migration focus
git-iris release-notes --from v1.0.0 --to v2.0.0 --preset detailed \
  --instructions "Include detailed migration guide"

# User-friendly release notes
git-iris release-notes --from v2.0.0 --preset concise \
  --instructions "Write for non-technical users"

# Create GitHub release
git-iris release-notes --from v2.0.0 --raw | \
  gh release create v2.1.0 --title "Release v2.1.0" --notes-file -

# Debug release notes generation
git-iris release-notes --from v2.0.0 --debug
```

## Comparison: Changelog vs Release Notes

| Aspect           | Changelog                        | Release Notes                   |
| ---------------- | -------------------------------- | ------------------------------- |
| **Format**       | Keep a Changelog structured list | Narrative documentation         |
| **Audience**     | Developers, maintainers          | End users, stakeholders         |
| **Detail Level** | Item-by-item changes             | High-level highlights + details |
| **Tone**         | Technical, factual               | Engaging, benefit-focused       |
| **Sections**     | Added, Changed, Fixed, etc.      | Overview, Highlights, Migration |
| **Use Case**     | Development tracking             | Release announcements           |

**When to use both:**

```bash
# Generate both for a release
git-iris changelog --from v2.0.0 --update
git-iris release-notes --from v2.0.0 --raw > RELEASE_NOTES.md
```

## Best Practices

1. **Include Migration Guides:** For breaking changes, provide clear upgrade steps
2. **Highlight User Benefits:** Focus on what users gain, not just technical details
3. **Be Honest About Known Issues:** Build trust by acknowledging limitations
4. **Provide Installation Instructions:** Make it easy for users to upgrade
5. **Link to Documentation:** Reference detailed docs for complex features
6. **Thank Contributors:** Acknowledge community contributions
7. **Test Before Publishing:** Review generated notes before creating release
