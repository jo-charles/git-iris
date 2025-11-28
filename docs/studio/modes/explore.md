# Explore Mode

**Explore Mode** is your semantic code browser. Navigate files, view syntax-highlighted source, and ask Iris "why does this code exist?" to get AI-powered historical analysis.

## When to Use Explore Mode

- **Understanding unfamiliar code**: Jump into a new codebase and get oriented
- **Investigating bugs**: Find out why a suspicious line was added
- **Code archaeology**: Trace the evolution of a feature through commits
- **Learning patterns**: See how the codebase implements specific patterns

## Panel Layout

```
┌─────────────┬──────────────────────┬─────────────────────┐
│  File Tree  │    Code View         │   Blame Analysis    │
│             │                      │                     │
│ src/        │  1  pub fn new() {   │ Why does this       │
│ ├─ agents/  │  2    Self {         │ code exist?         │
│ │  ├─ iris  │  3      mode: Auto,  │                     │
│ │  └─ tools │  4    }              │ This initialization │
│ ├─ studio ▸ │  5  }                │ was added to handle │
│ │  └─ state │                      │ emoji selection...  │
│ └─ types/   │  ← Line 3 selected   │                     │
│             │                      │ Commits:            │
│ 📄 Selected │  [Ctrl+H] Heat Map   │ • abc123f          │
│ iris.rs     │  [v] Visual Select   │ • def456a          │
│             │  [w] Why?            │                     │
└─────────────┴──────────────────────┴─────────────────────┘
```

### Left Panel: File Tree

- Shows repository directory structure
- Syntax-highlighted by file type
- Git status indicators (modified, staged, etc.)
- Collapsible directories

### Center Panel: Code View

- Syntax-highlighted source code
- Line numbers
- Current line indicator
- Visual selection support
- Optional heat map (change frequency)

### Right Panel: Blame Analysis

- Semantic "why" explanations (when requested)
- Related commit history
- Code evolution context
- Empty until you ask "why?"

## Essential Keybindings

### File Tree (Left Panel)

| Key                                 | Action                                                   |
| ----------------------------------- | -------------------------------------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Select next file                                         |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Select previous file                                     |
| <kbd>h</kbd> / <kbd>←</kbd>         | Collapse directory                                       |
| <kbd>l</kbd> / <kbd>→</kbd>         | Expand directory                                         |
| <kbd>Enter</kbd>                    | Expand directory or load file (moves focus to code view) |
| <kbd>g</kbd> / <kbd>Home</kbd>      | Jump to first file                                       |
| <kbd>G</kbd> / <kbd>End</kbd>       | Jump to last file                                        |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down                                                |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up                                                  |

### Code View (Center Panel)

| Key                                 | Action                                               |
| ----------------------------------- | ---------------------------------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Move down one line                                   |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Move up one line                                     |
| <kbd>g</kbd> / <kbd>Home</kbd>      | Jump to first line                                   |
| <kbd>G</kbd> / <kbd>End</kbd>       | Jump to last line                                    |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down                                            |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up                                              |
| <kbd>v</kbd>                        | Enter/exit visual selection mode                     |
| <kbd>w</kbd>                        | Ask "why does this code exist?" (semantic blame)     |
| <kbd>y</kbd>                        | Copy current line (or selection if in visual mode)   |
| <kbd>Shift+Y</kbd>                  | Copy entire file content                             |
| <kbd>Shift+H</kbd>                  | Toggle heat map (shows change frequency)             |
| <kbd>o</kbd>                        | Open in $EDITOR (shows command, doesn't suspend TUI) |

### Context Panel (Right Panel)

| Key                                 | Action      |
| ----------------------------------- | ----------- |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Scroll down |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Scroll up   |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down   |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up     |

## Visual Selection Mode

Press <kbd>v</kbd> to enter **vim-style visual selection**:

```rust
 42  pub fn new() {
 43    Self {                    ← Press 'v' here (anchor point)
 44      mode: EmojiMode::Auto,
 45      editing: false,
 46    }                          ← Press 'j' to extend selection
 47  }
```

### Visual Mode Controls

| Key                         | Action                               |
| --------------------------- | ------------------------------------ |
| <kbd>v</kbd>                | Toggle visual mode on/off            |
| <kbd>j</kbd> / <kbd>k</kbd> | Extend selection up/down             |
| <kbd>y</kbd>                | Copy selected lines to clipboard     |
| <kbd>Esc</kbd>              | Clear selection and exit visual mode |

### What You Can Do with Selection

1. **Copy code snippets**: Select + <kbd>y</kbd> → paste anywhere
2. **Ask about multiple lines**: Select + <kbd>w</kbd> → Iris explains the entire block
3. **Visual feedback**: Selected lines highlighted in Electric Purple

## Semantic Blame: The "Why?" Feature

Press <kbd>w</kbd> on any line to ask **"why does this code exist?"**

### What Happens

1. Iris gathers git blame data for that line
2. Analyzes related commits
3. Reads commit messages and diffs
4. Generates a semantic explanation

### Example

You press <kbd>w</kbd> on line 43:

```rust
 42  pub fn new() {
 43    Self {
 44      mode: EmojiMode::Auto,  ← Cursor here, press 'w'
```

Iris responds in the right panel:

```
Why does this code exist?

This initialization was added to support automatic emoji
selection in commit messages. Previously, emoji mode was a
simple boolean flag (use_gitmoji).

The change to an enum (EmojiMode::Auto) allows three states:
- None (no emoji)
- Auto (AI chooses)
- Custom (user picks)

This enables smarter defaults while preserving user control.

Related Commits:
• abc123f (2024-01-15) "Add emoji mode enum"
  Introduced EmojiMode to replace boolean flag

• def456a (2024-01-14) "Add emoji selector modal"
  Created UI for manual emoji selection
```

### With Visual Selection

Select multiple lines, then press <kbd>w</kbd>:

```rust
 43    Self {
 44      mode: EmojiMode::Auto,
 45      editing: false,
 46      messages: vec![],  ← Selection from 43-46, press 'w'
 47    }
```

Iris explains the **entire block** and how it evolved together.

## Heat Map

Press <kbd>Shift+H</kbd> to toggle the **change frequency heat map**:

```rust
 42  pub fn new() {               [████░░░░░░] 60% changed
 43    Self {                     [██████████] 100% hotspot!
 44      mode: EmojiMode::Auto,   [██████░░░░] 80%
 45      editing: false,          [██░░░░░░░░] 30%
```

- **Dark/Empty**: Rarely changed
- **Bright bars**: Frequently modified
- **Red-ish hue**: Very hot (changed in many commits)

### What Heat Map Shows

- Lines that change frequently (potential code smells)
- Stable areas (well-tested, trusted)
- Recent churn (active development)

Useful for:

- Finding fragile code
- Identifying core vs. peripheral logic
- Spotting refactor candidates

## Clipboard Integration

### Copy Current Line

Position cursor, press <kbd>y</kbd>:

```
✓ Line copied to clipboard
```

### Copy Selection

Enter visual mode (<kbd>v</kbd>), select lines, press <kbd>y</kbd>:

```
✓ 5 lines copied to clipboard
```

### Copy Entire File

Press <kbd>Shift+Y</kbd> anywhere:

```
✓ File content copied to clipboard
```

## Syntax Highlighting

Code is syntax-highlighted based on file extension:

- **Rust** (`.rs`): Keywords in Electric Purple, types in Neon Cyan
- **JavaScript/TypeScript** (`.js`, `.ts`, `.tsx`): Standard syntax colors
- **Markdown** (`.md`): Headers, links, code blocks
- **TOML/YAML** (`.toml`, `.yml`): Config-specific highlighting
- **Plain text**: Monochrome

Colors follow the SilkCircuit Neon palette.

## Workflow Examples

### Example 1: Understanding a New Codebase

**Goal**: Learn how the state management works

1. Open Studio in Explore mode
2. Navigate to `src/studio/state/mod.rs` in file tree
3. Press <kbd>Enter</kbd> to load file
4. Scan through code with <kbd>j</kbd>/<kbd>k</kbd>
5. See `StudioState` struct at line 789
6. Press <kbd>w</kbd> to ask why it exists
7. Read Iris's explanation in right panel
8. Press <kbd>/</kbd> to open chat: "Show me how state flows through the reducer"

### Example 2: Investigating a Bug

**Goal**: Find out why file selection is broken

1. Navigate to `src/studio/handlers/commit.rs`
2. Find suspicious function `sync_file_selection` at line 35
3. Press <kbd>v</kbd> to start visual selection
4. Press <kbd>j</kbd> × 4 to select the function body
5. Press <kbd>w</kbd> to ask why this code exists
6. Iris explains: "Added to fix race condition between tree and diff views"
7. Press <kbd>/</kbd> to chat: "Is there a better way to sync these components?"

### Example 3: Learning Code Patterns

**Goal**: See how the codebase uses Result types

1. Navigate to `src/agents/iris.rs`
2. Toggle heat map (<kbd>Shift+H</kbd>) to see active areas
3. Navigate to hot spots (frequently changed lines)
4. Press <kbd>w</kbd> on error handling code
5. Iris explains: "Added to gracefully handle JSON parse errors from LLM"
6. Copy pattern with <kbd>y</kbd> for reuse

### Example 4: Code Review Prep

**Goal**: Understand changes before creating a PR

1. In Explore mode, navigate through changed files
2. For each file, press <kbd>w</kbd> on key changes
3. Build mental model of "why" changes were made
4. Switch to Review mode (<kbd>Shift+R</kbd>)
5. Generate review with full context
6. Switch to PR mode (<kbd>Shift+P</kbd>)
7. Generate PR description (Iris remembers your exploration)

## Special Features

### File Status Indicators

In the file tree, files show git status:

- **Green** `M` — Modified
- **Purple** `A` — Staged (added)
- **Yellow** `?` — Untracked
- **Cyan** `R` — Renamed
- **Red** `D` — Deleted

### Smart Navigation

- Press <kbd>Enter</kbd> on a directory → Expands it
- Press <kbd>Enter</kbd> on a file → Loads it **and** moves focus to code view
- No need to <kbd>Tab</kbd> manually

### Context Persistence

When you switch modes, Explore remembers:

- Current file
- Cursor position
- Expanded directories
- Visual selection state (if any)

Return to Explore mode → pick up where you left off.

## Tips & Tricks

### 1. Use Visual Selection for Context

Don't just ask "why" about a single line. Select the entire function/struct/block for richer explanations.

### 2. Heat Map + Blame Combo

1. Toggle heat map (<kbd>Shift+H</kbd>)
2. Navigate to hottest lines
3. Press <kbd>w</kbd> to understand why they change often
4. Consider refactoring high-churn areas

### 3. Copy Before Switching Modes

If you find useful code in Explore:

1. Select it (<kbd>v</kbd> + <kbd>j</kbd>/<kbd>k</kbd>)
2. Copy it (<kbd>y</kbd>)
3. Switch to Commit mode (<kbd>Shift+C</kbd>)
4. Paste into commit message if relevant

### 4. Chat for Deeper Dives

Semantic blame gives you "why this line."
Chat gives you "how does this relate to everything else?"

- Press <kbd>w</kbd> for quick blame
- Press <kbd>/</kbd> for deep architectural questions

### 5. File Tree Filtering (Coming Soon)

Soon you'll be able to type in the file tree to filter:

- Type `iris` → Shows only files matching "iris"
- Clear filter → Back to full tree

## Limitations

### What Explore Can't Do

- **Edit files**: Read-only (use `$EDITOR` outside Studio)
- **Show uncommitted changes**: Displays HEAD version (use Commit mode for diffs)
- **Navigate by symbol**: No function/class jump (yet)

### Performance Notes

- **Large files** (>10,000 lines): May scroll slower
- **Binary files**: Not displayed (shows placeholder)
- **Very deep trees**: Consider using `fd` or ripgrep outside TUI

## Troubleshooting

### "No file selected" when pressing `w`

You're in the file tree panel. Press <kbd>Tab</kbd> to move to code view, then try again.

### Heat map shows all zeros

File hasn't been modified in tracked history. Try a file with recent commits.

### Semantic blame takes too long

For large files or deep history:

- Select fewer lines (narrow scope)
- Use chat instead: "Why was X added?" (faster, cached)

### Syntax highlighting looks wrong

Check file extension. Studio infers language from extension. Rename file or submit an issue if highlighting is broken.

## Next Steps

- Learn [Visual Selection](../navigation.md#visual-selection) techniques
- Master [Chat](../chat.md) for code questions
- Switch to [Commit Mode](commit.md) to act on what you learned
- See [Review Mode](review.md) for quality analysis
