# Semantic Token Reference

Complete reference of all semantic tokens used by Git-Iris. Every token must be defined in your theme for it to be valid.

## Token Naming Convention

Tokens use dot notation to create hierarchical namespaces:

```toml
"text.primary"       # Namespace: text, Property: primary
"bg.highlight"       # Namespace: bg, Property: highlight
"git.staged"         # Namespace: git, Property: staged
```

This structure makes it easy to understand token purpose and modify related colors together.

## Text Hierarchy

Controls text rendering throughout the UI.

| Token            | Usage                                  | Example                      |
| ---------------- | -------------------------------------- | ---------------------------- |
| `text.primary`   | Primary text, highest contrast         | File names, headings         |
| `text.secondary` | Secondary text, medium contrast        | Descriptions, metadata       |
| `text.muted`     | Tertiary text, lower contrast          | Labels, hints                |
| `text.dim`       | Lowest priority text, minimal contrast | Disabled items, placeholders |

**Example:**

```toml
[tokens]
"text.primary" = "#f8f8f2"
"text.secondary" = "#bcbcca"
"text.muted" = "#82879f"
"text.dim" = "#6e7daf"
```

**Visual hierarchy:**

```
Primary:   ████████ 100% contrast
Secondary: ██████░░  75% contrast
Muted:     ████░░░░  50% contrast
Dim:       ██░░░░░░  25% contrast
```

## Background Surfaces

Defines background layers and elevation.

| Token          | Usage                                | Example                    |
| -------------- | ------------------------------------ | -------------------------- |
| `bg.base`      | Main application background          | Canvas, root window        |
| `bg.panel`     | Panel/section backgrounds            | Sidebar, main content area |
| `bg.code`      | Code block backgrounds               | Diff view, file contents   |
| `bg.highlight` | Highlighted/hovered items            | Cursor line, row hover     |
| `bg.elevated`  | Elevated surfaces (modals, tooltips) | Popups, overlays           |
| `bg.active`    | Currently active/selected state      | Active tab, focused panel  |
| `bg.selection` | Text selection background            | Selected text range        |

**Example:**

```toml
[tokens]
"bg.base" = "#121218"        # Darkest
"bg.panel" = "#181820"       # Slightly lighter
"bg.code" = "#1e1e28"        # Code context
"bg.highlight" = "#37324b"   # Hover state
"bg.elevated" = "#37324b"    # Floating elements
"bg.active" = "#3c2d55"      # Active selection
"bg.selection" = "#3c3c50"   # Lightest
```

**Elevation model:**

```
Base      Panel     Code      Highlight Elevated  Active    Selection
████░░    █████░    ██████    ███████░  ███████░  ████████  █████████
```

## Accent Colors

Brand colors for emphasis and interaction.

| Token              | Usage                               | Example                |
| ------------------ | ----------------------------------- | ---------------------- |
| `accent.primary`   | Primary brand color, main actions   | Active mode, keywords  |
| `accent.secondary` | Secondary brand color, interactions | Links, hover states    |
| `accent.tertiary`  | Tertiary brand color, decorative    | Icons, badges          |
| `accent.deep`      | Deeper variant of primary           | Shadows, depth effects |

**Example:**

```toml
[tokens]
"accent.primary" = "#e135ff"     # Electric Purple
"accent.secondary" = "#80ffea"   # Neon Cyan
"accent.tertiary" = "#ff6ac1"    # Coral
"accent.deep" = "#bd93f9"        # Deep Purple
```

## Semantic Status Colors

Universal status indicators.

| Token     | Usage                            | Example                        |
| --------- | -------------------------------- | ------------------------------ |
| `success` | Positive states, confirmations   | Staged files, success messages |
| `error`   | Negative states, errors          | Deleted files, error messages  |
| `warning` | Caution states, attention needed | Modified files, warnings       |
| `info`    | Informational, neutral           | Hints, info messages           |

**Example:**

```toml
[tokens]
success = "#50fa7b"   # Green
error = "#ff6363"     # Red
warning = "#f1fa8c"   # Yellow
info = "#80ffea"      # Cyan
```

## Git Status Colors

Git file state indicators.

| Token           | Usage                            | Git Status     |
| --------------- | -------------------------------- | -------------- |
| `git.staged`    | Staged changes (ready to commit) | `A ` added     |
| `git.modified`  | Modified but unstaged            | ` M` modified  |
| `git.untracked` | Untracked files                  | `??` untracked |
| `git.deleted`   | Deleted files                    | ` D` deleted   |

**Example:**

```toml
[tokens]
"git.staged" = "#50fa7b"     # Green (ready)
"git.modified" = "#f1fa8c"   # Yellow (changed)
"git.untracked" = "#6e7daf"  # Gray (new)
"git.deleted" = "#ff6363"    # Red (removed)
```

**File tree rendering:**

```
src/
  main.rs         (staged)    █ Green
  config.rs       (modified)  █ Yellow
  temp.txt        (untracked) █ Gray
  old_code.rs     (deleted)   █ Red
```

## Diff Colors

Unified diff view syntax highlighting.

| Token          | Usage                      | Diff Line Prefix |
| -------------- | -------------------------- | ---------------- |
| `diff.added`   | Added lines                | `+`              |
| `diff.removed` | Removed lines              | `-`              |
| `diff.hunk`    | Hunk headers (`@@ ... @@`) | `@@`             |
| `diff.context` | Unchanged context lines    | ` ` (space)      |

**Example:**

```toml
[tokens]
"diff.added" = "#50fa7b"     # Green
"diff.removed" = "#ff6363"   # Red
"diff.hunk" = "#80ffea"      # Cyan
"diff.context" = "#6e7daf"   # Gray
```

**Diff rendering:**

```diff
@@ -12,6 +12,8 @@                    (diff.hunk)
 fn main() {                          (diff.context)
-    println!("old");                  (diff.removed)
+    println!("new");                  (diff.added)
+    println!("another");              (diff.added)
 }                                     (diff.context)
```

## UI Elements

Interface components and interactions.

| Token              | Usage                  | Example                |
| ------------------ | ---------------------- | ---------------------- |
| `border.focused`   | Focused panel border   | Active panel outline   |
| `border.unfocused` | Unfocused panel border | Inactive panel outline |

**Example:**

```toml
[tokens]
"border.focused" = "#80ffea"    # Bright cyan (attention)
"border.unfocused" = "#82879f"  # Gray (subtle)
```

**Border states:**

```
┌─ Focused Panel ──────┐   ┌─ Unfocused Panel ───┐
│ (border.focused)     │   │ (border.unfocused)  │
│ Bright, attention-   │   │ Subtle, recedes to  │
│ grabbing             │   │ background          │
└──────────────────────┘   └─────────────────────┘
```

## Code Syntax

Syntax highlighting tokens for source code.

| Token              | Usage                     | Example                    |
| ------------------ | ------------------------- | -------------------------- |
| `code.hash`        | Commit hashes, checksums  | `a3f2c9b`                  |
| `code.path`        | File paths, URLs          | `src/main.rs`              |
| `code.keyword`     | Programming keywords      | `fn`, `let`, `if`          |
| `code.function`    | Function/method names     | `calculate()`, `get_value` |
| `code.string`      | String literals           | `"hello"`, `'world'`       |
| `code.number`      | Numeric literals          | `42`, `3.14`, `0xFF`       |
| `code.comment`     | Code comments             | `// comment`, `/* ... */`  |
| `code.type`        | Type names, classes       | `String`, `Option<T>`      |
| `code.line_number` | Line numbers in code view | `1`, `2`, `3`              |

**Example:**

```toml
[tokens]
"code.hash" = "#ff6ac1"      # Coral
"code.path" = "#80ffea"      # Cyan
"code.keyword" = "#e135ff"   # Purple
"code.function" = "#80ffea"  # Cyan
"code.string" = "#ff99ff"    # Pink
"code.number" = "#ff6ac1"    # Coral
"code.comment" = "#6e7daf"   # Gray
"code.type" = "#f1fa8c"      # Yellow
"code.line_number" = "#6e7daf" # Gray
```

**Syntax highlighting:**

```rust
1  fn calculate(x: i32) -> i32 {  // Calculate result
   ██ ████████ ███ ███   ███         ████████████████
   │  │        │   │     │           └─ code.comment
   │  │        │   │     └─ code.keyword (return type)
   │  │        │   └─ code.type
   │  │        └─ code.type
   │  └─ code.function
   └─ code.line_number

2      let value = "test";
       ███ █████   ██████
       │   │       └─ code.string
       │   └─ identifier
       └─ code.keyword
```

## Mode Tabs

Navigation tab states.

| Token           | Usage                             | Example          |
| --------------- | --------------------------------- | ---------------- |
| `mode.active`   | Currently active mode             | Selected tab     |
| `mode.inactive` | Inactive modes                    | Unselected tabs  |
| `mode.hover`    | Hovered mode (future enhancement) | Tab under cursor |

**Example:**

```toml
[tokens]
"mode.active" = "#e135ff"    # Purple (bold)
"mode.inactive" = "#6e7daf"  # Gray (dim)
"mode.hover" = "#80ffea"     # Cyan (highlight)
```

**Tab bar:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 EXPLORE   COMMIT   REVIEW   PR   CHAT
 ███████   ██████   ██████   ██   ████
 (active)  (inactive)...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Chat

Conversational UI colors.

| Token       | Usage               | Example                |
| ----------- | ------------------- | ---------------------- |
| `chat.user` | User messages       | Your questions to Iris |
| `chat.iris` | Iris agent messages | Iris responses         |

**Example:**

```toml
[tokens]
"chat.user" = "#80ffea"    # Cyan (you)
"chat.iris" = "#e135ff"    # Purple (AI)
```

**Chat rendering:**

```
You:  What changed in this commit?
████  (chat.user)

Iris: This commit adds theme support...
████  (chat.iris)
```

## Token Validation

Git-Iris validates all required tokens at theme load time. If any token is missing or invalid, you'll see a clear error message:

```
Error: Theme validation failed

Missing tokens:
  - text.primary
  - bg.base
  - accent.primary

Invalid token references:
  - "code.keyword" references "nonexistent_color"

Invalid color values:
  - "text.muted" = "#gggggg" (invalid hex)
```

## Complete Token Checklist

Use this checklist when creating a new theme:

### Text (4 tokens)

- [ ] `text.primary`
- [ ] `text.secondary`
- [ ] `text.muted`
- [ ] `text.dim`

### Backgrounds (7 tokens)

- [ ] `bg.base`
- [ ] `bg.panel`
- [ ] `bg.code`
- [ ] `bg.highlight`
- [ ] `bg.elevated`
- [ ] `bg.active`
- [ ] `bg.selection`

### Accents (4 tokens)

- [ ] `accent.primary`
- [ ] `accent.secondary`
- [ ] `accent.tertiary`
- [ ] `accent.deep`

### Semantic Status (4 tokens)

- [ ] `success`
- [ ] `error`
- [ ] `warning`
- [ ] `info`

### Git Status (4 tokens)

- [ ] `git.staged`
- [ ] `git.modified`
- [ ] `git.untracked`
- [ ] `git.deleted`

### Diff (4 tokens)

- [ ] `diff.added`
- [ ] `diff.removed`
- [ ] `diff.hunk`
- [ ] `diff.context`

### UI Elements (2 tokens)

- [ ] `border.focused`
- [ ] `border.unfocused`

### Code Syntax (9 tokens)

- [ ] `code.hash`
- [ ] `code.path`
- [ ] `code.keyword`
- [ ] `code.function`
- [ ] `code.string`
- [ ] `code.number`
- [ ] `code.comment`
- [ ] `code.type`
- [ ] `code.line_number`

### Mode Tabs (3 tokens)

- [ ] `mode.active`
- [ ] `mode.inactive`
- [ ] `mode.hover`

### Chat (2 tokens)

- [ ] `chat.user`
- [ ] `chat.iris`

**Total: 43 required tokens**

## Token Evolution

As Git-Iris evolves, new tokens may be added. Your existing themes will continue to work—missing tokens fall back to safe defaults. However, for the best experience, update your themes when new tokens are introduced.

### Version Compatibility

| Git-Iris Version | Token Count | New Tokens              |
| ---------------- | ----------- | ----------------------- |
| 1.0.0            | 43          | Initial token set       |
| Future           | TBD         | Additions will be noted |

Check release notes for new token announcements.

## Usage in Code

Tokens are accessed throughout Git-Iris codebase:

```rust
use git_iris::theme;

// Get current theme
let theme = theme::current();

// Access token
let color = theme.color("accent.primary");

// Use in Ratatui style
let style = theme.style("keyword");
let ratatui_style = style.to_ratatui();
```

## Token Naming Philosophy

Token names follow these principles:

1. **Semantic over visual** — `accent.primary` not `purple`
2. **Hierarchical** — Use dots for namespacing
3. **Consistent** — Same pattern across categories
4. **Self-documenting** — Name reveals purpose

**Good token names:**

- `text.primary` — Clear hierarchy and purpose
- `git.staged` — Obvious semantic meaning
- `diff.hunk` — Specific, unambiguous

**Poor token names:**

- `color1` — No semantic meaning
- `purple_text` — Too specific, not flexible
- `important` — Vague, subjective

---

**Next Steps:**

- [Learn about styles](./styles.md)
- [Create your own theme](./creating.md)
- [View theme gallery](./gallery.md)
