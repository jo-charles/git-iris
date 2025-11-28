# Changelog Mode

**Changelog Mode** generates structured changelogs following the [Keep a Changelog](https://keepachangelog.com) format. Organize commits into categories (Added, Changed, Fixed, etc.) for clear release documentation.

## When to Use Changelog Mode

- **Release preparation**: Document changes between versions
- **Maintain CHANGELOG.md**: Keep structured change history
- **Communicate with users**: Explain what's new, changed, or fixed
- **Version planning**: See what's accumulated since last release

## Panel Layout

```
┌─────────────┬──────────────────────┬─────────────────────┐
│ Commits     │   Changelog Output   │   Diff Summary      │
│             │                      │                     │
│ abc123f Add │ ## [1.1.0] - 2024... │ @@ -10,6 +10,8 @@  │
│ def456a Fix │                      │                     │
│ ghi789b Rem │ ### Added            │ 15 files changed   │
│             │ - Emoji selector UI  │ +450 -120 lines    │
│ v1.0.0..    │ - Custom presets     │                     │
│ HEAD        │                      │ src/iris.rs        │
│             │ ### Changed          │ src/state.rs       │
│ 3 commits   │ - EmojiMode enum...  │ ...                │
│ 15 files    │                      │                     │
│             │ ### Fixed            │ [n] Next file       │
│ [f] From    │ - File tree sync     │ [p] Prev file       │
│ [t] To      │                      │ []] Next hunk       │
│ [r] Generate│ [Unreleased]         │ [[] Prev hunk       │
└─────────────┴──────────────────────┴─────────────────────┘
```

### Left Panel: Commit List

- All commits in version range
- Commit hash (short)
- Commit title
- Version tags displayed
- Ref range summary

### Center Panel: Changelog Output

- Keep a Changelog format
- Categorized changes (Added, Changed, Fixed, etc.)
- Markdown-formatted
- Version headers with dates
- Links to commits/PRs (if configured)

### Right Panel: Diff Summary

- Aggregated diff for context
- File change statistics
- Hunk navigation

## Essential Keybindings

### Commit List (Left Panel)

| Key                            | Action                    |
| ------------------------------ | ------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>    | Select next commit        |
| <kbd>k</kbd> / <kbd>↑</kbd>    | Select previous commit    |
| <kbd>g</kbd> / <kbd>Home</kbd> | Jump to first commit      |
| <kbd>G</kbd> / <kbd>End</kbd>  | Jump to last commit       |
| <kbd>f</kbd>                   | Select "from" version/tag |
| <kbd>t</kbd>                   | Select "to" version/tag   |
| <kbd>r</kbd>                   | Generate changelog        |

### Changelog Output (Center Panel)

| Key                                 | Action                      |
| ----------------------------------- | --------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Scroll down                 |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Scroll up                   |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down                   |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up                     |
| <kbd>g</kbd> / <kbd>Home</kbd>      | Jump to top                 |
| <kbd>G</kbd> / <kbd>End</kbd>       | Jump to bottom              |
| <kbd>r</kbd>                        | Regenerate changelog        |
| <kbd>Shift+R</kbd>                  | Reset (clear changelog)     |
| <kbd>y</kbd>                        | Copy changelog to clipboard |

### Diff View (Right Panel)

| Key                         | Action                |
| --------------------------- | --------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd> | Scroll down           |
| <kbd>k</kbd> / <kbd>↑</kbd> | Scroll up             |
| <kbd>[</kbd>                | Jump to previous hunk |
| <kbd>]</kbd>                | Jump to next hunk     |
| <kbd>n</kbd>                | Jump to next file     |
| <kbd>p</kbd>                | Jump to previous file |

## Keep a Changelog Format

Iris generates changelogs following the [Keep a Changelog](https://keepachangelog.com) standard:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2024-01-28

### Added

- Emoji selector modal for custom commit emoji selection
- EmojiMode enum to replace boolean gitmoji flag
- Quick toggle keybinding (Shift+E) for None/Auto modes
- Preset selector for commit message styles

### Changed

- Refactored commit mode to use unified message editor
- Improved file tree navigation with keyboard shortcuts
- Updated theme system to support SilkCircuit variants

### Fixed

- File selection sync between tree and diff views
- Scroll position persistence when switching modes
- Emoji display in commit message preview

### Deprecated

- `use_gitmoji` config option (use `emoji_mode` instead)

## [1.0.0] - 2024-01-15

### Added

- Initial release
- Studio TUI with six modes
- AI-powered commit message generation
- Semantic code exploration with blame
```

## Categories

### Added

**New features or capabilities**

Examples:

- New commands
- New modes
- New keybindings
- New UI components

### Changed

**Changes to existing functionality**

Examples:

- Improved algorithms
- Refactored code
- Updated dependencies
- Modified behavior

### Deprecated

**Features marked for future removal**

Examples:

- Config options being phased out
- Old API methods
- Legacy command syntax

### Removed

**Features removed in this version**

Examples:

- Deleted deprecated code
- Removed old APIs
- Dropped support for old versions

### Fixed

**Bug fixes**

Examples:

- Crashes
- Incorrect behavior
- UI glitches
- Performance issues

### Security

**Security vulnerability fixes**

Examples:

- CVE patches
- Authentication fixes
- Input validation improvements

## Ref Selection

Press <kbd>f</kbd> (from) or <kbd>t</kbd> (to) to select **version tags**:

### Common Workflows

| Scenario             | From       | To       | Description                |
| -------------------- | ---------- | -------- | -------------------------- |
| **Next release**     | `v1.0.0`   | `HEAD`   | Changes since last release |
| **Between releases** | `v1.0.0`   | `v1.1.0` | Specific version diff      |
| **Unreleased**       | Latest tag | `HEAD`   | What's coming next         |
| **Major version**    | `v1.0.0`   | `v2.0.0` | Breaking changes           |

### Version Tag Format

Supports:

- Semantic versioning: `v1.2.3`
- Without 'v': `1.2.3`
- Pre-release: `v1.2.3-beta.1`
- Build metadata: `v1.2.3+20240128`

## Workflow Examples

### Example 1: Prepare Release Changelog

**Goal**: Generate changelog for v1.1.0 release

1. Tag current HEAD: `git tag v1.1.0`
2. Switch to Changelog mode (<kbd>Shift+L</kbd>)
3. Press <kbd>f</kbd> → select `v1.0.0` (previous release)
4. Press <kbd>t</kbd> → select `v1.1.0` (new release)
5. Press <kbd>r</kbd> to generate changelog
6. Review categorization in center panel
7. Press <kbd>y</kbd> to copy
8. Append to CHANGELOG.md

### Example 2: Preview Unreleased Changes

**Goal**: See what's accumulated since last release

1. Open Changelog mode
2. Press <kbd>f</kbd> → select latest tag (e.g., `v1.0.0`)
3. Press <kbd>t</kbd> → leave as `HEAD`
4. Press <kbd>r</kbd> to generate
5. Section shows: `## [Unreleased]`
6. Review what's ready for next release

### Example 3: Maintaining CHANGELOG.md

**Goal**: Update existing CHANGELOG.md with new version

1. Generate changelog for new version
2. Press <kbd>y</kbd> to copy
3. Outside Studio: edit CHANGELOG.md
4. Paste new version section at top (below `[Unreleased]`)
5. Move previous `[Unreleased]` items to new version section
6. Commit updated CHANGELOG.md

### Example 4: Refining Categories

**Goal**: Adjust categorization with chat

1. Generate initial changelog
2. See commit "Refactor emoji handling" in "Changed"
3. Press <kbd>/</kbd> to chat: "Move 'Refactor emoji handling' to 'Added' since it adds new functionality"
4. Iris recategorizes and updates output
5. Review and copy

### Example 5: Multiple Versions

**Goal**: Document changes across several releases

1. Generate v1.0.0..v1.1.0
2. Copy output (<kbd>y</kbd>)
3. Press <kbd>f</kbd> → select `v1.1.0`
4. Press <kbd>t</kbd> → select `v1.2.0`
5. Press <kbd>r</kbd> to generate next version
6. Combine both outputs in CHANGELOG.md

## Categorization Logic

Iris uses commit analysis to categorize:

### How Commits Are Categorized

**Added**:

- Conventional commits: `feat:`, `feature:`
- Keywords: "add", "introduce", "implement", "create"
- Emoji: ✨ (sparkles), 🎉 (tada)

**Changed**:

- Conventional commits: `refactor:`, `perf:`, `improve:`
- Keywords: "update", "change", "improve", "refactor"
- Emoji: ♻️ (recycle), ⚡ (zap), 🎨 (art)

**Fixed**:

- Conventional commits: `fix:`, `bugfix:`
- Keywords: "fix", "resolve", "correct", "patch"
- Emoji: 🐛 (bug), 🚑 (ambulance)

**Deprecated**:

- Keywords: "deprecate", "obsolete"
- Emoji: 🗑️ (wastebasket)

**Removed**:

- Conventional commits: `remove:`, `delete:`
- Keywords: "remove", "delete", "drop"
- Emoji: 🔥 (fire), ➖ (minus)

**Security**:

- Conventional commits: `security:`
- Keywords: "security", "vulnerability", "CVE"
- Emoji: 🔒 (lock), 🔐 (closed lock)

### Override Categorization

Use chat to adjust:

```
You: Move "Update dependencies" from Changed to Security

Iris: Recategorized:
      ### Security
      - Update dependencies (addresses CVE-2024-1234)

      (Removed from Changed section)
```

## Special Features

### Unreleased Section

When "to" ref is `HEAD` (not a tag):

```markdown
## [Unreleased]

### Added

- Feature X (in progress)
- Feature Y (completed)

### Changed

- Refactored Z
```

Indicates changes not yet in a tagged release.

### Version Links

If git remote is configured, Iris adds comparison links:

```markdown
## [1.1.0] - 2024-01-28

...

[Unreleased]: https://github.com/user/repo/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/user/repo/compare/v1.0.0...v1.1.0
```

Enables clicking to view full diff on GitHub.

### Breaking Changes Callout

Major version bumps (v1.x.x → v2.0.0) get special treatment:

```markdown
## [2.0.0] - 2024-02-01

### ⚠️ BREAKING CHANGES

- Removed legacy config format
- Changed API signatures in ...

### Added

...
```

### Empty Category Suppression

Categories with no changes are omitted:

```markdown
## [1.1.0] - 2024-01-28

### Added

- ...

### Fixed

- ...

<!-- No "Changed", "Deprecated", "Removed", or "Security" sections -->
```

Keeps output clean.

## Chat Integration

Refine changelogs with chat:

### Add Context

```
You: Add more detail to "Fixed file tree sync"

Iris: Updated:
      - Fixed file selection sync between tree and diff views
        (Previously selection state was lost when switching panels)
```

### Merge Items

```
You: Combine the emoji-related items into one entry

Iris: Updated:
      Before:
      - Emoji selector modal
      - EmojiMode enum
      - Quick toggle keybinding

      After:
      - Comprehensive emoji selection system with modal UI,
        three-state mode (None/Auto/Custom), and keyboard shortcuts
```

### Add Migration Notes

```
You: Add migration guidance for the deprecated use_gitmoji config

Iris: Added to Deprecated section:
      - `use_gitmoji` config option
        (Migration: use `emoji_mode = "auto"` instead.
         Existing configs auto-migrate on first run.)
```

## Tips & Tricks

### 1. Generate on Every Release

Make it automatic:

```bash
# In release script
git tag v1.1.0
git-iris studio  # Changelog mode, f=prev tag, t=new tag, r, y
# Paste into CHANGELOG.md, commit
```

### 2. Use Unreleased Section for Planning

1. Generate `last-tag..HEAD`
2. Review Unreleased section
3. Decide if it's enough for a release
4. If yes: tag version and regenerate

### 3. Combine with Conventional Commits

Write conventional commit messages:

```
feat: Add emoji selector
fix: Resolve file tree sync issue
refactor: Simplify state management
```

Iris categorizes automatically → perfect changelogs.

### 4. Keep CHANGELOG.md Updated

Don't wait for release:

1. After each major feature merge
2. Generate unreleased changelog
3. Update CHANGELOG.md with new items
4. Commit: "docs: Update CHANGELOG with feature X"

### 5. Include in Release Notes

1. Generate changelog for version
2. Copy (<kbd>y</kbd>)
3. Switch to Release Notes mode (<kbd>Shift+N</kbd>)
4. Generate release notes
5. Use chat: "Include this changelog: [paste]"

### 6. Highlight User Impact

```
You: Rewrite "Added" items to focus on user benefits

Iris: Updated:
      Before:
      - Emoji selector modal

      After:
      - Choose custom emojis for commit messages, making it easier
        to visually categorize changes at a glance
```

## Troubleshooting

### All commits in one category

**Symptom**: Everything appears under "Changed"

**Cause**: Commit messages don't follow conventional format

**Fix**:

1. Use chat: "Recategorize based on actual changes, not just message format"
2. Or improve commit message discipline going forward

### Missing commits

**Symptom**: Some commits don't appear in changelog

**Cause**: Iris filtered out noise (merge commits, version bumps, etc.)

**Fix**:

1. Press <kbd>j</kbd>/<kbd>k</kbd> in left panel to see full commit list
2. Use chat: "Include commit abc123f in changelog"

### Wrong version number

**Symptom**: Header shows `## [1.0.0]` but should be `## [1.1.0]`

**Cause**: Wrong "to" ref selected

**Fix**:

1. Press <kbd>t</kbd> to select correct version tag
2. Press <kbd>r</kbd> to regenerate

### Date is wrong

**Symptom**: `## [1.1.0] - 2023-12-01` but today is 2024-01-28

**Cause**: Iris uses tag creation date

**Fix**:

1. Copy changelog (<kbd>y</kbd>)
2. Manually edit date when pasting into CHANGELOG.md
3. Or use chat: "Update date to 2024-01-28"

## Next Steps

- Learn [Release Notes Mode](release-notes.md) for user-facing documentation
- Use [PR Mode](pr.md) for individual feature documentation
- Master [Chat](../chat.md) for changelog refinement
- Read [Keep a Changelog](https://keepachangelog.com) standard
