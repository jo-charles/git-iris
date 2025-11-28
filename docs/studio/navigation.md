# Navigation Patterns

Iris Studio follows **vim-inspired keybindings** for consistent, keyboard-driven navigation across all modes. This guide covers movement, selection, and panel focus.

## Core Philosophy

- **Vim-like movement**: <kbd>hjkl</kbd> or arrow keys
- **Context-aware actions**: Keys behave intelligently based on what you're viewing
- **Modal focus**: Different panels respond to the same keys differently
- **No mouse required**: Everything is keyboard-accessible

## Movement Keys

These work in **all scrollable contexts** (file trees, code views, diffs, lists):

| Key                                  | Action                             | Context           |
| ------------------------------------ | ---------------------------------- | ----------------- |
| <kbd>j</kbd> or <kbd>↓</kbd>         | Move down one line/item            | Universal         |
| <kbd>k</kbd> or <kbd>↑</kbd>         | Move up one line/item              | Universal         |
| <kbd>h</kbd> or <kbd>←</kbd>         | Collapse directory / Previous item | Context-dependent |
| <kbd>l</kbd> or <kbd>→</kbd>         | Expand directory / Next item       | Context-dependent |
| <kbd>g</kbd> or <kbd>Home</kbd>      | Jump to first item                 | Universal         |
| <kbd>G</kbd> or <kbd>End</kbd>       | Jump to last item                  | Universal         |
| <kbd>Ctrl+d</kbd> or <kbd>PgDn</kbd> | Scroll down one page (~20 lines)   | Universal         |
| <kbd>Ctrl+u</kbd> or <kbd>PgUp</kbd> | Scroll up one page (~20 lines)     | Universal         |

### File Tree Navigation

In file tree panels (Explore, Commit, Review):

```
src/
├─ agents/          ← Current selection
│  ├─ iris.rs
│  └─ tools/
└─ studio/
```

- <kbd>j</kbd>/<kbd>k</kbd> — Move between files/directories
- <kbd>l</kbd> or <kbd>Enter</kbd> — Expand directory or load file
- <kbd>h</kbd> — Collapse expanded directory
- <kbd>g</kbd>/<kbd>G</kbd> — Jump to top/bottom of tree

### Code View Navigation

In code panels (Explore mode center panel):

```rust
  1  pub fn handle_commit_key(...) {
  2      if state.editing {           ← Current line (highlighted)
  3          return handle_editing_key(state, key);
  4      }
```

- <kbd>j</kbd>/<kbd>k</kbd> — Move cursor up/down by line
- <kbd>Ctrl+d</kbd>/<kbd>Ctrl+u</kbd> — Page up/down
- <kbd>g</kbd>/<kbd>G</kbd> — Jump to first/last line
- <kbd>v</kbd> — Enter visual selection mode (Explore only)

### Diff Navigation

In diff panels (Commit, Review, PR, Changelog, Release modes):

```diff
@@ -10,6 +10,8 @@ impl CommitMode {

 fn handle_files_key(...) {
     match key.code {
+        KeyCode::Char('s') => { ... }  ← Current hunk
+        KeyCode::Char('u') => { ... }
```

- <kbd>j</kbd>/<kbd>k</kbd> — Scroll diff line by line
- <kbd>[</kbd> / <kbd>]</kbd> — Jump to previous/next hunk
- <kbd>n</kbd> / <kbd>p</kbd> — Jump to next/previous file in diff
- <kbd>Ctrl+d</kbd>/<kbd>Ctrl+u</kbd> — Page through diff

### List Navigation

In commit lists (PR, Changelog, Release modes):

```
● abc123f Fix authentication bug        ← Selected
● def456a Add user settings panel
● ghi789b Update dependencies
```

- <kbd>j</kbd>/<kbd>k</kbd> — Select previous/next commit
- <kbd>g</kbd>/<kbd>G</kbd> — Jump to first/last commit
- <kbd>Enter</kbd> — View commit details (context-dependent)

## Panel Focus

Studio has three panels: **Left**, **Center**, **Right**. Only one has focus at a time.

```
┌─────────┬────────────┬──────────┐
│  LEFT   │   CENTER   │  RIGHT   │  ← Focus indicators
│  (dim)  │  (bright)  │  (dim)   │     shown via border color
└─────────┴────────────┴──────────┘
```

### Focus Control

| Key                  | Action                                          |
| -------------------- | ----------------------------------------------- |
| <kbd>Tab</kbd>       | Focus next panel (Left → Center → Right → Left) |
| <kbd>Shift+Tab</kbd> | Focus previous panel (reverse)                  |

### Smart Focus

Some actions automatically move focus:

- **File selection** (<kbd>Enter</kbd> on file in tree) → Focus moves to content panel
- **Mode switch** → Focus defaults to most relevant panel for that mode
  - Commit: Center (message editor)
  - Review/PR/Changelog/Release: Center (output)
  - Explore: Left (file tree)

## Visual Selection

**Explore mode only**: Select multiple lines for copying or analysis.

### Entering Visual Mode

Press <kbd>v</kbd> while viewing code:

```rust
  1  pub fn handle_commit_key(...) {
  2      if state.editing {           ← Anchor point
  3          return handle_editing_key(state, key);  ← Selection extends here
  4      }
```

### Selection Controls

| Key                       | Action                               |
| ------------------------- | ------------------------------------ |
| <kbd>v</kbd>              | Enter/exit visual selection mode     |
| <kbd>j</kbd>/<kbd>k</kbd> | Extend selection up/down             |
| <kbd>y</kbd>              | Copy selected lines to clipboard     |
| <kbd>Esc</kbd>            | Clear selection and exit visual mode |

Selected lines are highlighted in Electric Purple (`#e135ff`).

## Context-Specific Actions

Some keys have different meanings based on panel focus:

### In File Trees (Left Panel)

| Key                         | Action                                 |
| --------------------------- | -------------------------------------- |
| <kbd>Enter</kbd>            | Expand directory / Load file into view |
| <kbd>h</kbd> / <kbd>l</kbd> | Collapse / Expand directory            |
| <kbd>s</kbd>                | Stage file (Commit mode only)          |
| <kbd>u</kbd>                | Unstage file (Commit mode only)        |
| <kbd>a</kbd>                | Stage all files (Commit mode only)     |
| <kbd>Shift+U</kbd>          | Unstage all files (Commit mode only)   |

### In Code Views (Center Panel)

| Key                         | Action                             |
| --------------------------- | ---------------------------------- |
| <kbd>j</kbd> / <kbd>k</kbd> | Navigate by line                   |
| <kbd>v</kbd>                | Visual selection (Explore)         |
| <kbd>w</kbd>                | Semantic blame (Explore)           |
| <kbd>e</kbd>                | Edit message (Commit)              |
| <kbd>r</kbd>                | Regenerate (Commit/Review/PR/etc.) |
| <kbd>y</kbd>                | Copy to clipboard                  |

### In Diff Views (Right Panel)

| Key                                   | Action             |
| ------------------------------------- | ------------------ |
| <kbd>[</kbd> / <kbd>]</kbd>           | Jump between hunks |
| <kbd>n</kbd> / <kbd>p</kbd>           | Jump between files |
| <kbd>Ctrl+d</kbd> / <kbd>Ctrl+u</kbd> | Page through diff  |

## Clipboard Operations

Copy content to system clipboard with <kbd>y</kbd>:

### What Gets Copied

- **File Tree**: File path
- **Code View**: Current line (or selected range in visual mode)
- **Code View** (<kbd>Shift+Y</kbd>): Entire file content
- **Message Editor**: Full commit message
- **Review/PR/Changelog/Release**: Full generated content

After copying, you'll see a success notification:

```
✓ Copied to clipboard
```

## Scrolling Behavior

### Automatic Scrolling

When you navigate past the visible area, the view automatically scrolls to keep your selection visible:

```
Visible area (20 lines)
┌─────────────────────┐
│  18  line           │
│  19  line           │
│  20  line  ← cursor │  ← Scrolls down when you press j
└─────────────────────┘
```

### Page Scrolling

<kbd>Ctrl+d</kbd> and <kbd>Ctrl+u</kbd> move by ~20 lines (one page):

- Keeps cursor in view
- Overlaps a few lines for context
- Works in all scrollable panels

## Search & Jump

### Quick Jump to File

In Explore mode:

1. Focus file tree (Left panel)
2. Start typing filename → files auto-filter (coming soon)
3. Navigate filtered results with <kbd>j</kbd>/<kbd>k</kbd>

### Jump to Ref (Review/PR/Changelog/Release modes)

| Key          | Action                              |
| ------------ | ----------------------------------- |
| <kbd>f</kbd> | Select "from" ref (base branch/tag) |
| <kbd>t</kbd> | Select "to" ref (target branch/tag) |

Opens a filterable ref selector modal.

## Hunk & File Navigation in Diffs

When viewing diffs (Commit, Review, PR modes):

### Hunks

A hunk is a contiguous block of changes:

```diff
@@ -10,6 +10,8 @@  ← Hunk header
 existing line
+added line      ← This is hunk 1
 existing line

@@ -20,3 +22,4 @@  ← Hunk header
 another line
+another addition ← This is hunk 2
```

- <kbd>[</kbd> — Jump to previous hunk
- <kbd>]</kbd> — Jump to next hunk

### Files

When a diff contains multiple files:

```diff
diff --git a/src/agents/iris.rs b/src/agents/iris.rs  ← File 1
...

diff --git a/src/studio/state.rs b/src/studio/state.rs  ← File 2
...
```

- <kbd>n</kbd> — Jump to next file in diff
- <kbd>p</kbd> — Jump to previous file in diff

## Modal Navigation

When modals are open (Help, Chat, Settings, Selectors):

| Key                       | Action                                      |
| ------------------------- | ------------------------------------------- |
| <kbd>Esc</kbd>            | Close modal                                 |
| <kbd>j</kbd>/<kbd>k</kbd> | Navigate options (selectors)                |
| <kbd>Enter</kbd>          | Confirm selection                           |
| Text input                | Type to filter (ref/emoji/preset selectors) |

See specific mode documentation for modal-specific keys.

## Tips for Efficient Navigation

### 1. Use Panel Focus Strategically

- Start in Left panel to select files
- <kbd>Enter</kbd> to auto-focus content panel
- <kbd>Tab</kbd> to Right panel for diff details

### 2. Combine Movement Keys

- <kbd>g</kbd> then <kbd>]</kbd> — Jump to first hunk
- <kbd>G</kbd> then <kbd>[</kbd> — Jump to last hunk
- <kbd>v</kbd> then <kbd>G</kbd> — Select from current to end

### 3. Leverage Smart Actions

- <kbd>Enter</kbd> on a directory = expand it
- <kbd>Enter</kbd> on a file = load and move focus
- <kbd>r</kbd> anywhere = regenerate current context

### 4. Visual Selection Workflow

```
1. Press v to anchor
2. Press j/k to extend
3. Press y to copy
4. Press Esc to clear
```

## Accessibility Notes

- **No mouse required**: All actions are keyboard-accessible
- **Visual feedback**: Active panel has bright border (Electric Purple)
- **Status indicators**: Current line/item highlighted in selection color
- **Scroll indicators**: `↓` and `↑` arrows show when content extends beyond view

## Next Steps

- See mode-specific guides for specialized keys:
  - [Explore Mode](modes/explore.md)
  - [Commit Mode](modes/commit.md)
  - [Review Mode](modes/review.md)
- Learn about [Chat with Iris](chat.md) for command-based navigation
