# Commit Mode

**Commit Mode** generates AI-powered commit messages with emoji, custom presets, and manual editing. Stage files, review changes, and create meaningful commits—all in one interface.

## When to Use Commit Mode

- **Creating commits**: Generate messages that capture intent, not just changes
- **Refining messages**: Edit AI suggestions to match your style
- **Bulk staging**: Stage/unstage files with keyboard shortcuts
- **Quick commits**: Skip the `git add` + `git commit -m` dance

## Panel Layout

```
┌─────────────┬──────────────────────┬─────────────────────┐
│ Changed     │  Commit Message      │   Diff Preview      │
│ Files       │                      │                     │
│             │ ✨ Add emoji selector│ @@ -10,6 +10,8 @@  │
│ ● iris.rs M │                      │  impl CommitMode {  │
│   state.rs M│ Introduces emoji     │ +  pub emoji_mode   │
│   commit.rs │ selection UI for     │ +  pub use_gitmoji  │
│             │ commit messages with │                     │
│ 2 staged    │ three modes: None,   │ Hunk 1/3           │
│ 1 modified  │ Auto (AI picks), and │                     │
│             │ Custom (user picks). │ [[] Previous hunk   │
│ [s] Stage   │                      │ []] Next hunk       │
│ [a] All     │ [e] Edit  [r] Regen  │ [n] Next file       │
│ [u] Unstage │ [g] Emoji [p] Preset │ [p] Prev file       │
└─────────────┴──────────────────────┴─────────────────────┘
```

### Left Panel: Changed Files

- Staged files (ready to commit)
- Modified files (unstaged)
- Untracked files
- Directory tree structure
- Git status indicators

### Center Panel: Message Editor

- AI-generated commit message
- Title + body format
- Emoji (if enabled)
- Manual editing support
- Multiple message variants

### Right Panel: Diff Preview

- Unified diff for selected file
- Syntax-highlighted changes
- Hunk navigation
- Multi-file diff support

## Essential Keybindings

### File Tree (Left Panel)

| Key                            | Action                                          |
| ------------------------------ | ----------------------------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>    | Select next file                                |
| <kbd>k</kbd> / <kbd>↑</kbd>    | Select previous file                            |
| <kbd>h</kbd> / <kbd>←</kbd>    | Collapse directory                              |
| <kbd>l</kbd> / <kbd>→</kbd>    | Expand directory                                |
| <kbd>Enter</kbd>               | Load file diff (moves focus to diff panel)      |
| <kbd>s</kbd>                   | Stage selected file                             |
| <kbd>u</kbd>                   | Unstage selected file                           |
| <kbd>a</kbd>                   | Stage all files                                 |
| <kbd>Shift+U</kbd>             | Unstage all files                               |
| <kbd>Shift+A</kbd>             | Toggle view (changed files ↔ all tracked files) |
| <kbd>g</kbd> / <kbd>Home</kbd> | Jump to first file                              |
| <kbd>G</kbd> / <kbd>End</kbd>  | Jump to last file                               |

### Message Editor (Center Panel)

| Key                         | Action                                 |
| --------------------------- | -------------------------------------- |
| <kbd>e</kbd>                | Edit message (enter text editing mode) |
| <kbd>r</kbd>                | Regenerate message with AI             |
| <kbd>Shift+R</kbd>          | Reset to original generated message    |
| <kbd>i</kbd>                | Add custom instructions for generation |
| <kbd>g</kbd>                | Open emoji selector                    |
| <kbd>Shift+E</kbd>          | Quick toggle emoji (None ↔ Auto)       |
| <kbd>p</kbd>                | Open preset selector (style templates) |
| <kbd>y</kbd>                | Copy message to clipboard              |
| <kbd>Enter</kbd>            | Execute commit                         |
| <kbd>←</kbd> / <kbd>→</kbd> | Navigate between message variants      |

### Diff View (Right Panel)

| Key                                 | Action                        |
| ----------------------------------- | ----------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Scroll down                   |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Scroll up                     |
| <kbd>[</kbd>                        | Jump to previous hunk         |
| <kbd>]</kbd>                        | Jump to next hunk             |
| <kbd>n</kbd>                        | Jump to next file in diff     |
| <kbd>p</kbd>                        | Jump to previous file in diff |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down                     |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up                       |

### Text Editing Mode

When editing message (after pressing <kbd>e</kbd>):

| Key                  | Action            |
| -------------------- | ----------------- |
| Type                 | Enter text        |
| <kbd>Backspace</kbd> | Delete character  |
| <kbd>Ctrl+w</kbd>    | Delete word       |
| <kbd>Ctrl+u</kbd>    | Delete line       |
| <kbd>Enter</kbd>     | New line          |
| <kbd>Esc</kbd>       | Exit editing mode |

## Emoji Selector

Press <kbd>g</kbd> to open the **emoji selector modal**:

```
┌─────────────────────────────────────────────┐
│  Select Emoji                          [Esc] │
├─────────────────────────────────────────────┤
│  Type to filter...                           │
│                                              │
│  ∅   none     No emoji                       │
│  ✨  auto     Let AI choose              ◀  │
│  🎨  art      Improve structure/format       │
│  ⚡  zap      Improve performance            │
│  🐛  bug      Fix a bug                      │
│  ✨  sparkles Introduce new features         │
│  📝  memo     Add or update documentation    │
│  🚀  rocket   Deploy stuff                   │
│  💄  lipstick Update UI/style                │
│  ♻️   recycle Refactor code                  │
│  🔒  lock     Fix security issues            │
│                                              │
│  [j/k] Navigate  [Enter] Select  [/] Search  │
└─────────────────────────────────────────────┘
```

### Emoji Modes

| Mode       | Behavior                     | Display        |
| ---------- | ---------------------------- | -------------- |
| **None**   | No emoji in message          | ∅              |
| **Auto**   | AI picks appropriate emoji   | ✨ (example)   |
| **Custom** | You choose from gitmoji list | 🎨 (your pick) |

### Emoji Selector Controls

| Key                                                        | Action                      |
| ---------------------------------------------------------- | --------------------------- |
| <kbd>j</kbd> / <kbd>k</kbd> or <kbd>↓</kbd> / <kbd>↑</kbd> | Navigate options            |
| <kbd>Enter</kbd>                                           | Select emoji                |
| <kbd>Esc</kbd>                                             | Cancel (keep current emoji) |
| Type                                                       | Filter by name/description  |

### Quick Toggle

Press <kbd>Shift+E</kbd> to quickly toggle between **None** and **Auto** without opening the selector.

## Preset Selector

Press <kbd>p</kbd> to open the **preset selector**:

```
┌─────────────────────────────────────────────┐
│  Select Preset Style                   [Esc] │
├─────────────────────────────────────────────┤
│  Type to filter...                           │
│                                              │
│  ✨ default                              ◀  │
│     Standard commit with context             │
│                                              │
│  🎯 concise                                  │
│     Brief, single-line commits               │
│                                              │
│  📚 detailed                                 │
│     Verbose with rationale                   │
│                                              │
│  🔧 technical                                │
│     Implementation-focused                   │
│                                              │
│  [j/k] Navigate  [Enter] Select              │
└─────────────────────────────────────────────┘
```

### What Presets Do

Presets guide Iris's message generation style:

- **default**: Balanced, descriptive, follows conventional commits
- **concise**: Short title, minimal body
- **detailed**: Extensive context, rationale, tradeoffs
- **technical**: Implementation details, API changes

### Custom Presets

You can add your own in `~/.config/git-iris/presets.toml`:

```toml
[[preset]]
name = "quirky"
emoji = "🎪"
description = "Fun, playful commit messages"
instructions = """
Write commit messages that are fun and playful while still
being informative. Use metaphors and light humor.
"""
```

## Custom Instructions

Press <kbd>i</kbd> to add **one-time instructions** for the next generation:

```
┌─────────────────────────────────────────────┐
│  Custom Instructions                   [Esc] │
├─────────────────────────────────────────────┤
│  These will be added to the next generation  │
│                                              │
│  Make it more technical and mention the      │
│  use of the reducer pattern_                 │
│                                              │
│  [Enter] Apply  [Esc] Cancel                 │
└─────────────────────────────────────────────┘
```

Instructions are used **once** for the next <kbd>r</kbd> (regenerate), then cleared.

## Workflow Examples

### Example 1: Quick Commit

**Goal**: Stage changes and commit with AI message

1. Open Studio in Commit mode (or <kbd>Shift+C</kbd>)
2. Files are already loaded in left panel
3. Press <kbd>a</kbd> to stage all
4. Studio auto-generates commit message
5. Review message in center panel
6. Press <kbd>Enter</kbd> to commit

Done! Full workflow in 4 keystrokes.

### Example 2: Selective Staging

**Goal**: Commit only specific files

1. Navigate file tree with <kbd>j</kbd>/<kbd>k</kbd>
2. Press <kbd>s</kbd> on `iris.rs` to stage it
3. Press <kbd>j</kbd> to move to next file
4. Press <kbd>s</kbd> on `state.rs` to stage it
5. Press <kbd>r</kbd> to generate message
6. Press <kbd>Enter</kbd> to commit

### Example 3: Custom Message with Emoji

**Goal**: Pick a specific emoji and refine message

1. Stage files (<kbd>a</kbd>)
2. Press <kbd>g</kbd> to open emoji selector
3. Type "bug" to filter
4. Select 🐛 with <kbd>Enter</kbd>
5. Press <kbd>r</kbd> to regenerate with bug context
6. Press <kbd>e</kbd> to edit message
7. Type refinements
8. Press <kbd>Esc</kbd> to exit editing
9. Press <kbd>Enter</kbd> to commit

### Example 4: Using Presets

**Goal**: Generate a very detailed commit message

1. Stage files
2. Press <kbd>p</kbd> to open preset selector
3. Select "detailed" preset
4. Press <kbd>r</kbd> to regenerate
5. Review detailed message
6. Press <kbd>i</kbd> to add instructions: "Mention performance impact"
7. Press <kbd>Enter</kbd> in instructions modal
8. Press <kbd>r</kbd> to regenerate again
9. Review + commit

### Example 5: Exploring Multiple Variants

**Goal**: Generate several messages and pick the best

1. Stage files
2. Message auto-generates (variant 1)
3. Press <kbd>r</kbd> to generate variant 2
4. Press <kbd>r</kbd> again for variant 3
5. Use <kbd>←</kbd>/<kbd>→</kbd> to navigate between variants
6. Press <kbd>y</kbd> to copy favorite to clipboard
7. Press <kbd>e</kbd> to manually combine best parts
8. Press <kbd>Enter</kbd> to commit

### Example 6: Chat-Assisted Refinement

**Goal**: Iteratively refine message with Iris

1. Stage files, auto-generate message
2. Press <kbd>/</kbd> to open chat
3. Type: "Make this more concise"
4. Iris updates message directly
5. Press <kbd>Esc</kbd> to close chat
6. Review updated message
7. Press <kbd>/</kbd> again: "Add emoji"
8. Iris updates with emoji
9. Press <kbd>Enter</kbd> to commit

## Staging Shortcuts

### Stage Individual Files

```
● iris.rs    M  ← Press 's' to stage
  state.rs   M
  commit.rs  M
```

After <kbd>s</kbd>:

```
● iris.rs    A  ← Now staged (green)
  state.rs   M
  commit.rs  M
```

### Stage All

Press <kbd>a</kbd> to stage everything:

```
Before:           After:
● iris.rs    M    ● iris.rs    A
  state.rs   M      state.rs   A
  commit.rs  M      commit.rs  A
```

### Unstage Individual

Press <kbd>u</kbd> on a staged file to unstage it.

### Unstage All

Press <kbd>Shift+U</kbd> to unstage everything.

### Toggle View

Press <kbd>Shift+A</kbd> to toggle between:

- **Changed files** (default): Only modified/staged/untracked
- **All tracked files**: Entire repository tree

Useful when you want to see unchanged files for context.

## Message Format

Iris generates messages in this format:

```
[emoji] Title (max 50 chars)

Body paragraph explaining what changed and why.
Can span multiple lines.

- Bullet points for details
- Implementation notes
- Breaking changes if any
```

### Title Rules

- Max 50 characters
- Imperative mood ("Add" not "Added")
- No period at end
- Emoji prefix (if enabled)

### Body Guidelines

- Wrap at 72 characters
- Explain **why**, not just **what**
- Mention **tradeoffs** for significant changes
- Use bullet points for lists

### Examples

**Concise preset**:

```
✨ Add emoji selector to commit UI
```

**Default preset**:

```
✨ Add emoji selector to commit UI

Introduces a modal for selecting commit message emojis with
three modes: None (no emoji), Auto (AI chooses), and Custom
(user picks from gitmoji list).
```

**Detailed preset**:

```
✨ Add emoji selector to commit UI

Introduces a comprehensive emoji selection interface that
replaces the previous boolean gitmoji flag with a richer
three-state system.

Previous behavior:
- use_gitmoji: true → AI picks emoji
- use_gitmoji: false → no emoji

New behavior:
- EmojiMode::None → no emoji
- EmojiMode::Auto → AI picks emoji
- EmojiMode::Custom(emoji) → user-selected emoji

This change enables:
- Manual emoji override (press 'g' in commit mode)
- Quick toggle (Shift+E for None/Auto)
- Preserved backward compatibility via config migration

Implementation uses a filterable modal with j/k navigation
and type-to-search functionality.
```

## Special Features

### Auto-Generation

As soon as you stage files, Iris **automatically generates** a commit message. No need to press <kbd>r</kbd> unless you want a new variant.

### Multiple Variants

Each time you press <kbd>r</kbd>, Iris generates a **new message** and adds it to the list. Navigate between them with <kbd>←</kbd>/<kbd>→</kbd>.

Current variant indicator:

```
Message 2 of 3  ◀ ◀ ◀ ◀
```

### Smart Emoji Selection

When emoji mode is **Auto**, Iris analyzes:

- File types changed
- Nature of changes (new feature, bug fix, refactor, etc.)
- Commit message content

Then picks the most appropriate gitmoji.

### Diff Sync

Select a file in the left panel → Diff automatically loads in the right panel. No manual action needed.

### Commit Preview

Before committing, you can:

- View full diff (<kbd>Tab</kbd> to right panel)
- Navigate through all hunks (<kbd>[</kbd>/<kbd>]</kbd>)
- Check staged vs. unstaged files (left panel)

## Tips & Tricks

### 1. Let AI Do The Work

Don't start by editing. Let Iris generate, then refine if needed:

1. Stage files → auto-generates
2. If not perfect, press <kbd>r</kbd> for variant
3. Still not perfect? Press <kbd>/</kbd> to chat: "Make it shorter"

### 2. Use Presets for Consistency

If your team has a commit style:

1. Create a custom preset
2. Select it with <kbd>p</kbd> at the start of each session
3. All generated messages follow that style

### 3. Copy Before Committing

Nervous about losing a good message?

1. Press <kbd>y</kbd> to copy to clipboard
2. Make edits
3. If you mess up, paste back with <kbd>e</kbd> then Ctrl+V (in edit mode)

### 4. Combine Chat with Editing

Chat for big changes, editing for small tweaks:

- **Chat** (<kbd>/</kbd>): "Rewrite to focus on performance"
- **Edit** (<kbd>e</kbd>): Fix typos, adjust wording

### 5. Stage Incrementally

Don't stage everything at once:

1. Stage related files (<kbd>s</kbd> on each)
2. Generate message
3. Commit
4. Repeat for next logical group

Creates cleaner git history.

### 6. Emoji as Signal

Use emoji mode to signal commit type:

- **Auto**: Standard workflow
- **Custom (🚧)**: Work in progress
- **Custom (🎨)**: Style/refactor
- **None**: Serious production fixes

## Troubleshooting

### Message not generating

**Symptom**: Center panel is empty after staging files

**Fix**:

1. Check that files are actually staged (green `A` in left panel)
2. Press <kbd>r</kbd> to manually trigger generation
3. Check bottom status bar for error messages

### Can't edit message

**Symptom**: Typing does nothing

**Fix**:

1. Press <kbd>Tab</kbd> to focus center panel
2. Press <kbd>e</kbd> to enter edit mode
3. Now type

### Emoji not showing

**Symptom**: Message has no emoji despite Auto mode

**Fix**:

1. Check emoji mode in bottom bar
2. Press <kbd>g</kbd> to confirm "Auto" is selected
3. Press <kbd>r</kbd> to regenerate
4. If still no emoji, Iris may have decided it's inappropriate (e.g., for merge commits)

### Commit fails

**Symptom**: Press <kbd>Enter</kbd> but nothing happens

**Fix**:

1. Check you have staged files (left panel should show green `A`)
2. Check message is not empty (center panel)
3. Look for error notification in bottom bar
4. Check terminal output if you started Studio with `--debug`

## Next Steps

- Learn [Chat integration](../chat.md) for message refinement
- Switch to [Review Mode](review.md) before committing
- See [Navigation Patterns](../navigation.md) for advanced movement
- Check [Explore Mode](explore.md) to understand changes before committing
