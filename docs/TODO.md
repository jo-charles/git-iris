# Documentation TODO

## Screenshots Needed

All screenshots should go in `/docs/assets/screenshots/`

### Studio Overview

| Filename                | Description                            | Where Used           |
| ----------------------- | -------------------------------------- | -------------------- |
| `studio-overview.png`   | Full Studio TUI showing 3-panel layout | `/studio/index.md`   |
| `studio-dark-mode.png`  | Studio in SilkCircuit Neon (dark)      | `/themes/gallery.md` |
| `studio-light-mode.png` | Studio in SilkCircuit Dawn (light)     | `/themes/gallery.md` |

### Studio Modes

| Filename                 | Description                                           | Where Used                       |
| ------------------------ | ----------------------------------------------------- | -------------------------------- |
| `mode-explore.png`       | Explore mode with file tree + code view + analysis    | `/studio/modes/explore.md`       |
| `mode-commit.png`        | Commit mode with staged files + message editor + diff | `/studio/modes/commit.md`        |
| `mode-review.png`        | Review mode showing AI review output                  | `/studio/modes/review.md`        |
| `mode-pr.png`            | PR mode with commit list + PR description             | `/studio/modes/pr.md`            |
| `mode-changelog.png`     | Changelog mode with categorized output                | `/studio/modes/changelog.md`     |
| `mode-release-notes.png` | Release Notes mode with user-facing content           | `/studio/modes/release-notes.md` |

### Features & Modals

| Filename             | Description                         | Where Used                |
| -------------------- | ----------------------------------- | ------------------------- |
| `chat-modal.png`     | Chat with Iris modal open           | `/studio/chat.md`         |
| `emoji-selector.png` | Emoji picker modal in Commit mode   | `/studio/modes/commit.md` |
| `settings-modal.png` | Settings modal with provider config | `/studio/navigation.md`   |
| `help-modal.png`     | Help modal showing keybindings      | `/studio/navigation.md`   |

### CLI Output

| Filename         | Description                          | Where Used                        |
| ---------------- | ------------------------------------ | --------------------------------- |
| `cli-gen.png`    | Terminal output of `git-iris gen`    | `/getting-started/quick-start.md` |
| `cli-review.png` | Terminal output of `git-iris review` | `/user-guide/reviews.md`          |

### Theme Variants

| Filename             | Description                  | Where Used           |
| -------------------- | ---------------------------- | -------------------- |
| `theme-neon.png`     | SilkCircuit Neon variant     | `/themes/gallery.md` |
| `theme-dawn.png`     | SilkCircuit Dawn variant     | `/themes/gallery.md` |
| `theme-midnight.png` | SilkCircuit Midnight variant | `/themes/gallery.md` |
| `theme-ember.png`    | SilkCircuit Ember variant    | `/themes/gallery.md` |

---

## Other Tasks

### High Priority

- [ ] Add `logo.svg` to `/docs/public/logo.svg`
- [ ] Create assets directory: `mkdir -p docs/assets/screenshots`
- [ ] Capture all screenshots listed above

### Medium Priority

- [ ] Add workflow examples to User Guide (real-world scenarios)
- [ ] Create video/GIF demos showing Studio in action
- [ ] Expand `/reference/troubleshooting.md` with FAQ section

### Low Priority (Polish)

- [ ] Consolidate `/themes/tokens.md` and `/reference/tokens.md` (some overlap)
- [ ] Add version badges showing which Git-Iris version docs apply to
- [ ] Consider adding copy buttons to code blocks

---

## Screenshot Capture Guidelines

### Terminal Setup

```bash
# Recommended terminal size
# Width: 120 columns
# Height: 35 rows

# Font: JetBrains Mono or similar monospace
# Font size: 14px recommended
```

### Capture Checklist

- [ ] Terminal font is JetBrains Mono or similar monospace
- [ ] Background is clean (no desktop visible)
- [ ] Cropped to terminal window only
- [ ] PNG format
- [ ] 2x resolution for retina displays if possible
- [ ] Dark mode for Neon theme screenshots
- [ ] Light mode for Dawn theme screenshots

### Commands for Screenshots

```bash
# Studio overview
git-iris studio

# Specific modes (use Shift+key to switch)
# Shift+E = Explore
# Shift+C = Commit
# Shift+R = Review
# Shift+P = PR
# Shift+L = Changelog
# Shift+N = Release Notes

# CLI screenshots
git-iris gen --print
git-iris review --print
```

---

## Content Improvements

### Writing Enhancements (Completed)

- [x] Add engaging hooks to Getting Started intro
- [x] Rewrite Studio mode descriptions with personality
- [x] Add teaching example disclaimers to Extending docs

### Future Content

- [ ] Add "Common Workflows" section showing end-to-end usage
- [ ] Add integration guides (GitHub Actions, GitLab CI, pre-commit hooks)
- [ ] Add migration guide for version upgrades
- [ ] Consider adding a glossary of terms
