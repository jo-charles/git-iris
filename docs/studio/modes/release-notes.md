# Release Notes Mode

**Release Notes Mode** generates user-facing release documentation with highlights, breaking changes, and migration guidance. Perfect for public releases, customer communication, and GitHub releases.

## When to Use Release Notes Mode

- **Public releases**: Announce new versions to users
- **GitHub releases**: Generate polished release notes
- **Customer communication**: Highlight what matters to end-users
- **Marketing material**: Showcase new features and improvements
- **Internal updates**: Inform team about production changes

## Panel Layout

```
┌─────────────┬──────────────────────┬─────────────────────┐
│ Commits     │  Release Notes       │   Diff Summary      │
│             │                      │                     │
│ abc123f Add │ # Git-Iris v1.1.0    │ @@ -10,6 +10,8 @@  │
│ def456a Fix │                      │                     │
│ ghi789b Upd │ ## Highlights        │ 20 files changed   │
│             │                      │ +650 -180 lines    │
│ v1.0.0..    │ 🎨 **Custom Emoji    │                     │
│ v1.1.0      │ Selection**          │ src/studio/        │
│             │                      │ src/agents/        │
│ 5 commits   │ Choose from 100+...  │ ...                │
│ 20 files    │                      │                     │
│             │ ## What's New        │ [n] Next file       │
│ [f] From    │                      │ [p] Prev file       │
│ [t] To      │ ### Features         │ []] Next hunk       │
│ [r] Generate│ - Emoji selector...  │ [[] Prev hunk       │
└─────────────┴──────────────────────┴─────────────────────┘
```

### Left Panel: Commit List

- All commits in version range
- Version tags
- Commit titles
- Selection indicator

### Center Panel: Release Notes

- User-focused narrative
- Highlights section
- Feature descriptions
- Breaking changes
- Migration guidance
- Acknowledgments

### Right Panel: Diff Summary

- Change statistics
- File overview
- Context for technical details

## Essential Keybindings

### Commit List (Left Panel)

| Key                            | Action                 |
| ------------------------------ | ---------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>    | Select next commit     |
| <kbd>k</kbd> / <kbd>↑</kbd>    | Select previous commit |
| <kbd>g</kbd> / <kbd>Home</kbd> | Jump to first commit   |
| <kbd>G</kbd> / <kbd>End</kbd>  | Jump to last commit    |
| <kbd>f</kbd>                   | Select "from" version  |
| <kbd>t</kbd>                   | Select "to" version    |
| <kbd>r</kbd>                   | Generate release notes |

### Release Notes (Center Panel)

| Key                                 | Action                   |
| ----------------------------------- | ------------------------ |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Scroll down              |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Scroll up                |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down                |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up                  |
| <kbd>g</kbd> / <kbd>Home</kbd>      | Jump to top              |
| <kbd>G</kbd> / <kbd>End</kbd>       | Jump to bottom           |
| <kbd>r</kbd>                        | Regenerate release notes |
| <kbd>Shift+R</kbd>                  | Reset (clear notes)      |
| <kbd>y</kbd>                        | Copy to clipboard        |

### Diff View (Right Panel)

| Key                         | Action                |
| --------------------------- | --------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd> | Scroll down           |
| <kbd>k</kbd> / <kbd>↑</kbd> | Scroll up             |
| <kbd>[</kbd>                | Jump to previous hunk |
| <kbd>]</kbd>                | Jump to next hunk     |
| <kbd>n</kbd>                | Jump to next file     |
| <kbd>p</kbd>                | Jump to previous file |

## Release Notes Format

Iris generates **user-focused** documentation (unlike technical changelogs):

````markdown
# Git-Iris v1.1.0

Released on January 28, 2024

## 🎯 Highlights

### 🎨 Custom Emoji Selection

Choose from 100+ emojis for your commit messages! The new emoji
selector lets you pick the perfect icon to categorize your work.
Press `g` in Commit mode to explore.

### ⚡ Faster Navigation

Keyboard shortcuts have been streamlined across all modes. Switch
between panels with Tab, jump to files with Enter, and navigate
code with vim-like hjkl keys.

## What's New

### Features

- **Emoji Selector**: Interactive modal with type-to-filter
- **Preset System**: Choose commit message styles (concise, detailed, technical)
- **Visual Selection**: Select multiple lines in Explore mode (vim-style `v`)
- **Heat Map**: See which lines change most frequently

### Improvements

- Smoother panel transitions with instant focus updates
- Syntax highlighting now supports TypeScript and TOML
- Diff view shows hunk indicators ([2/5])
- File tree remembers expanded state when switching modes

### Bug Fixes

- Fixed file selection sync between tree and diff panels
- Resolved scroll position loss when changing focus
- Corrected emoji display in message preview
- Addressed race condition in blame analysis

## 🚨 Breaking Changes

### Config Format Update

The `use_gitmoji` boolean has been replaced with `emoji_mode`:

**Before** (v1.0.x):

```toml
use_gitmoji = true
```
````

**After** (v1.1.0+):

```toml
emoji_mode = "auto"  # or "none" or "custom:<emoji>"
```

**Migration**: Your existing config will auto-migrate on first run.
No manual changes needed.

## Installation

### macOS

```bash
brew install git-iris
```

### Linux

```bash
cargo install git-iris
```

### From Source

```bash
git clone https://github.com/user/git-iris
cd git-iris
cargo build --release
```

## Upgrade Guide

If you're upgrading from v1.0.x:

1. **Backup your config**:

   ```bash
   cp ~/.config/git-iris/config.toml ~/.config/git-iris/config.toml.backup
   ```

2. **Update Git-Iris**:

   ```bash
   brew upgrade git-iris  # or cargo install --force
   ```

3. **Run once** to trigger config migration:

   ```bash
   git-iris studio
   ```

4. **Verify emoji mode**:
   - Open Settings (press `,` in Studio)
   - Check "Emoji" field shows your preference

## Known Issues

- Heat map may be slow on very large files (10,000+ lines)
  [Workaround: Use Explore mode for smaller files]
- Emoji selector doesn't filter by category yet
  [Planned for v1.2.0]

## Contributors

Thank you to everyone who contributed to this release:

- @alice - Emoji selector UI
- @bob - Performance optimizations
- @charlie - Bug fixes and testing

## What's Next

Looking ahead to v1.2.0:

- Search across entire codebase
- Jump to symbol/function
- Multi-repository support
- Plugin system

Follow development at https://github.com/user/git-iris

---

**Full Changelog**: https://github.com/user/git-iris/compare/v1.0.0...v1.1.0

```

## Release Notes vs. Changelog

| Aspect | Changelog | Release Notes |
|--------|-----------|---------------|
| **Audience** | Developers | End-users |
| **Tone** | Technical | Accessible |
| **Structure** | Categorized (Added, Changed, Fixed) | Narrative (Highlights, Features, Fixes) |
| **Detail** | Implementation specifics | User benefits |
| **Format** | Keep a Changelog standard | Flexible, storytelling |
| **Use case** | CHANGELOG.md file | GitHub releases, blog posts |

## Workflow Examples

### Example 1: GitHub Release

**Goal**: Publish v1.1.0 release on GitHub

1. Tag release: `git tag v1.1.0`
2. Push tag: `git push origin v1.1.0`
3. Switch to Release Notes mode (<kbd>Shift+N</kbd>)
4. Press <kbd>f</kbd> → select `v1.0.0`
5. Press <kbd>t</kbd> → select `v1.1.0`
6. Press <kbd>r</kbd> to generate
7. Press <kbd>y</kbd> to copy
8. Go to GitHub → Releases → Draft new release
9. Paste notes, publish

### Example 2: Blog Post Announcement

**Goal**: Write release announcement for blog

1. Generate release notes for version
2. Press <kbd>/</kbd> to chat: "Make this more conversational for a blog post"
3. Iris rewrites in friendly tone
4. Press <kbd>/</kbd> again: "Add a 'Why This Matters' section"
5. Iris adds context about significance
6. Copy (<kbd>y</kbd>) and paste into blog draft

### Example 3: Customer Email

**Goal**: Send release update to customers

1. Generate release notes
2. Press <kbd>/</kbd>: "Rewrite for non-technical users, focus on benefits"
3. Iris simplifies language
4. Press <kbd>/</kbd>: "Limit to top 3 highlights only"
5. Iris condenses to key points
6. Copy and use in email template

### Example 4: Internal Team Update

**Goal**: Inform team about production deployment

1. Generate release notes
2. Press <kbd>/</kbd>: "Add 'Deployment Impact' section for ops team"
3. Iris adds operational notes:
   - Config changes
   - Performance impact
   - Rollback procedure
4. Copy and share in Slack/Teams

### Example 5: Multi-Version Summary

**Goal**: Document changes across several releases

1. Generate v1.0.0..v1.1.0 notes
2. Copy (<kbd>y</kbd>)
3. Press <kbd>f</kbd> → `v1.1.0`
4. Press <kbd>t</kbd> → `v1.2.0`
5. Generate again
6. Combine both for "What's new in Q1" post

## Special Features

### Highlights Extraction

Iris identifies the **3-5 most impactful changes** and features them prominently:

- Major new features
- Significant performance improvements
- Breaking changes
- Security fixes

### User Benefit Focus

Technical commits are translated to user benefits:

**Commit**: `refactor: Extract sync logic to trait`

**Release Note**:
```

### Improvements

- Smoother file synchronization across all modes
  (Previously selection state could drift between panels)

````

### Breaking Changes Callout

Major version changes get special attention:

```markdown
## 🚨 Breaking Changes

### API Signature Update
The `generate_commit` function now requires...

[Migration guide]
````

### Upgrade Instructions

Iris generates step-by-step upgrade guidance:

1. Backup steps
2. Installation commands
3. Migration procedures
4. Verification steps

### Visual Formatting

- **Emoji headers** for quick scanning (🎯, 🎨, ⚡, 🚨)
- **Bold feature names** for emphasis
- **Code blocks** for commands
- **Before/after examples** for migrations

## Chat Integration

Refine release notes with chat:

### Adjust Tone

```
You: Make the highlights more exciting, this is a major release!

Iris: Updated highlights:
      🎨 **Custom Emoji Selection - Your Commits, Your Style!**

      Say goodbye to boring commit messages! Choose from 100+
      vibrant emojis to make your git history pop. Just press
      `g` in Commit mode and pick your favorite.
```

### Add Sections

```
You: Add a "Community Thanks" section

Iris: Added:
      ## 💙 Community Thanks

      This release wouldn't be possible without our amazing
      community:
      - 15 contributors
      - 50+ issues resolved
      - 1,200+ stars on GitHub

      Thank you for making Git-Iris better!
```

### Simplify Language

```
You: The "Breaking Changes" section is too technical. Simplify.

Iris: Updated:
      Before:
      The `use_gitmoji` boolean flag has been deprecated in favor
      of an enum-based EmojiMode with three variants...

      After:
      Emoji settings now have three options instead of on/off.
      Your existing settings will update automatically.
```

### Add Screenshots/GIFs

```
You: Add placeholders for screenshots

Iris: Added visual preview section with emoji selector and heat map documentation.
```

## Tips & Tricks

### 1. Generate Early for Feedback

Don't wait until release day:

1. Generate notes from `last-tag..HEAD`
2. Share with team: "This is what we're shipping"
3. Adjust roadmap if notes don't tell a good story

### 2. Use Multiple Audiences

Generate different versions:

- **Developers**: Default output (technical details)
- **Users**: Chat → "Simplify for end-users"
- **Marketing**: Chat → "Make this exciting and benefit-focused"

### 3. Include Visuals

After generating:

1. Add screenshot placeholders
2. Record GIFs of new features
3. Create before/after comparisons
4. Embed in final notes

### 4. Combine with Changelog

1. Generate Changelog (<kbd>Shift+L</kbd>)
2. Copy to CHANGELOG.md
3. Switch to Release Notes (<kbd>Shift+N</kbd>)
4. Generate and copy
5. Use for GitHub release
6. Link release notes to CHANGELOG: "See CHANGELOG.md for details"

### 5. Save as Template

First release notes become template for future releases:

1. Generate v1.0.0 notes
2. Save structure (sections, tone, format)
3. For v1.1.0, use chat: "Use the same structure as v1.0.0 notes"

### 6. Highlight Contributors

```
You: Add a contributors section with GitHub @mentions

Iris: Added:
      ## Contributors

      Thank you to everyone who contributed:
      - @alice - Emoji selector UI (#123, #125)
      - @bob - Performance optimizations (#130)
      - @charlie - Bug fixes (#128, #131, #134)
```

## Release Notes Checklist

Before publishing, verify:

- [ ] **Version number** is correct in title
- [ ] **Date** is accurate (or "Released today")
- [ ] **Highlights** showcase 3-5 key changes
- [ ] **Breaking changes** are clearly called out
- [ ] **Migration guide** is step-by-step
- [ ] **Installation instructions** are current
- [ ] **Links** work (changelog, issues, commits)
- [ ] **Tone** matches audience (technical vs. user-friendly)
- [ ] **Visuals** are included (screenshots, GIFs)
- [ ] **Contributors** are acknowledged

## Troubleshooting

### Notes are too technical

**Symptom**: Reads like a changelog, not user-facing

**Fix**:

1. Press <kbd>/</kbd> to chat: "Rewrite for non-technical users"
2. Iris translates technical details to benefits

### Missing key features

**Symptom**: Important change not in highlights

**Fix**:

1. Press <kbd>/</kbd>: "Add [feature] to highlights"
2. Or manually scroll to find feature and chat: "Move this to highlights"

### No breaking changes section

**Symptom**: You know there are breaking changes but section is missing

**Fix**:

1. Press <kbd>/</kbd>: "Add breaking changes section for [change]"
2. Iris creates section with migration guidance

### Version number wrong

**Symptom**: Title shows wrong version

**Cause**: Wrong "to" ref selected

**Fix**:

1. Press <kbd>t</kbd> to select correct version tag
2. Press <kbd>r</kbd> to regenerate

## Publishing Checklist

After copying release notes:

### GitHub Releases

1. Go to repo → Releases → Draft new release
2. Choose tag (or create new one)
3. Title: `v1.1.0` or `Git-Iris v1.1.0`
4. Paste release notes
5. Attach binaries (if applicable)
6. Mark as "pre-release" if beta
7. Publish

### Blog/Website

1. Create new post
2. Paste release notes
3. Add hero image/banner
4. Embed screenshots/GIFs
5. Add "Download" or "Upgrade" CTA
6. Cross-link to GitHub release

### Social Media

1. Extract highlights (2-3 bullet points)
2. Add link to full notes
3. Include screenshot
4. Post to Twitter, LinkedIn, etc.

### Email Newsletter

1. Use simplified version (chat → "Summarize in 200 words")
2. Link to full notes on blog/GitHub
3. Add upgrade instructions
4. Include support contact

## Next Steps

- Learn [Changelog Mode](changelog.md) for technical change history
- Use [PR Mode](pr.md) for feature-specific documentation
- Master [Chat](../chat.md) for tone and content refinement
- See [Commit Mode](commit.md) for better commit messages → better release notes
