# Theme Token Reference

Quick reference for all SilkCircuit theme tokens.

## What Are Theme Tokens?

Theme tokens are semantic color names that adapt to different theme variants. Instead of hardcoding colors, Git-Iris uses tokens like `accent.primary` that map to colors in the active theme.

## Available Themes

| Theme                 | Variant | Description                             |
| --------------------- | ------- | --------------------------------------- |
| `silkcircuit-neon`    | Dark    | Electric purple and neon cyan (default) |
| `silkcircuit-soft`    | Dark    | Muted, gentle tones                     |
| `silkcircuit-glow`    | Dark    | Vibrant glowing accents                 |
| `silkcircuit-vibrant` | Dark    | High saturation colors                  |
| `silkcircuit-dawn`    | Light   | Warm pastel palette                     |

## Using Themes

```bash
# List available themes
git-iris themes

# Set theme in config
git-iris config --theme silkcircuit-glow

# Override for one session
git-iris studio --theme silkcircuit-vibrant
```

## Token Categories

### Text Tokens

| Token            | Purpose            | Default (Neon) |
| ---------------- | ------------------ | -------------- |
| `text.primary`   | Primary text       | `#f8f8f2`      |
| `text.secondary` | Secondary text     | `#bcbcca`      |
| `text.muted`     | De-emphasized text | `#82879f`      |
| `text.dim`       | Very subtle text   | `#6e7daf`      |

### Background Tokens

| Token          | Purpose               | Default (Neon) |
| -------------- | --------------------- | -------------- |
| `bg.base`      | Main background       | `#121218`      |
| `bg.panel`     | Panel background      | `#181820`      |
| `bg.code`      | Code block background | `#1e1e28`      |
| `bg.highlight` | Highlighted areas     | `#37324b`      |
| `bg.elevated`  | Raised surfaces       | `#37324b`      |
| `bg.active`    | Active/focused state  | `#3c2d55`      |
| `bg.selection` | Selected items        | `#3c3c50`      |

### Accent Tokens

| Token              | Purpose               | Default (Neon)              |
| ------------------ | --------------------- | --------------------------- |
| `accent.primary`   | Primary brand color   | `#e135ff` (Electric Purple) |
| `accent.secondary` | Secondary brand color | `#80ffea` (Neon Cyan)       |
| `accent.tertiary`  | Tertiary brand color  | `#ff6ac1` (Coral)           |
| `accent.deep`      | Deep accent           | `#bd93f9` (Purple 400)      |

### Semantic Tokens

| Token     | Purpose        | Default (Neon)     |
| --------- | -------------- | ------------------ |
| `success` | Success states | `#50fa7b` (Green)  |
| `error`   | Error states   | `#ff6363` (Red)    |
| `warning` | Warning states | `#f1fa8c` (Yellow) |
| `info`    | Information    | `#80ffea` (Cyan)   |

### Git Status Tokens

| Token           | Purpose         | Default (Neon)     |
| --------------- | --------------- | ------------------ |
| `git.staged`    | Staged files    | `#50fa7b` (Green)  |
| `git.modified`  | Modified files  | `#f1fa8c` (Yellow) |
| `git.untracked` | Untracked files | `#6e7daf` (Gray)   |
| `git.deleted`   | Deleted files   | `#ff6363` (Red)    |

### Diff Tokens

| Token          | Purpose       | Default (Neon)    |
| -------------- | ------------- | ----------------- |
| `diff.added`   | Added lines   | `#50fa7b` (Green) |
| `diff.removed` | Removed lines | `#ff6363` (Red)   |
| `diff.hunk`    | Hunk headers  | `#80ffea` (Cyan)  |
| `diff.context` | Context lines | `#6e7daf` (Gray)  |

### Border Tokens

| Token              | Purpose                | Default (Neon)   |
| ------------------ | ---------------------- | ---------------- |
| `border.focused`   | Focused panel border   | `#80ffea` (Cyan) |
| `border.unfocused` | Unfocused panel border | `#82879f` (Gray) |

### Code Highlighting Tokens

| Token              | Purpose         | Default (Neon)     |
| ------------------ | --------------- | ------------------ |
| `code.hash`        | Commit hashes   | `#ff6ac1` (Coral)  |
| `code.path`        | File paths      | `#80ffea` (Cyan)   |
| `code.keyword`     | Keywords        | `#e135ff` (Purple) |
| `code.function`    | Function names  | `#80ffea` (Cyan)   |
| `code.string`      | String literals | `#ff99ff` (Pink)   |
| `code.number`      | Numbers         | `#ff6ac1` (Coral)  |
| `code.comment`     | Comments        | `#6e7daf` (Gray)   |
| `code.type`        | Type names      | `#f1fa8c` (Yellow) |
| `code.line_number` | Line numbers    | `#6e7daf` (Gray)   |

### Mode Tokens

| Token           | Purpose            | Default (Neon)     |
| --------------- | ------------------ | ------------------ |
| `mode.active`   | Active mode tab    | `#e135ff` (Purple) |
| `mode.inactive` | Inactive mode tabs | `#6e7daf` (Gray)   |
| `mode.hover`    | Hovered mode tab   | `#80ffea` (Cyan)   |

### Chat Tokens

| Token       | Purpose       | Default (Neon)     |
| ----------- | ------------- | ------------------ |
| `chat.user` | User messages | `#80ffea` (Cyan)   |
| `chat.iris` | Iris messages | `#e135ff` (Purple) |

## Style Tokens

Styles combine colors with modifiers (bold, italic, etc.).

### Common Styles

| Style              | Description                     |
| ------------------ | ------------------------------- |
| `keyword`          | Keywords (bold, primary accent) |
| `file_path`        | File paths (secondary accent)   |
| `file_path_bold`   | File paths (bold)               |
| `commit_hash`      | Commit hashes (coral)           |
| `line_number`      | Line numbers (dim)              |
| `cursor_line`      | Current line highlight          |
| `selected`         | Selected item                   |
| `active_selected`  | Active selected item (bold)     |
| `focused_border`   | Focused panel border            |
| `unfocused_border` | Unfocused panel border          |
| `success_style`    | Success messages                |
| `error_style`      | Error messages                  |
| `warning_style`    | Warning messages                |
| `info_style`       | Info messages                   |
| `dimmed`           | Dimmed text                     |
| `muted`            | Muted text                      |
| `inline_code`      | Inline code blocks              |
| `mode_active`      | Active mode tab (bold)          |
| `mode_inactive`    | Inactive mode tab               |
| `mode_hover`       | Hovered mode tab                |
| `git_staged`       | Staged file                     |
| `git_modified`     | Modified file                   |
| `git_untracked`    | Untracked file                  |
| `git_deleted`      | Deleted file                    |
| `diff_added`       | Added diff lines                |
| `diff_removed`     | Removed diff lines              |
| `diff_hunk`        | Diff hunk headers               |
| `diff_context`     | Diff context lines              |
| `author`           | Author names                    |
| `timestamp`        | Timestamps                      |

## Gradient Tokens

Gradients interpolate between colors. Access with `gradient(name, t)` where `t` is 0.0-1.0.

| Gradient           | Colors               | Purpose              |
| ------------------ | -------------------- | -------------------- |
| `primary`          | Purple → Cyan        | Brand gradient       |
| `warm`             | Coral → Yellow       | Warm accents         |
| `success_gradient` | Green → Cyan         | Success transitions  |
| `error_gradient`   | Red → Coral          | Error transitions    |
| `aurora`           | Purple → Pink → Cyan | 5-stop aurora effect |

## Color Values by Theme

### SilkCircuit Neon (Default)

| Token              | Hex       | RGB             |
| ------------------ | --------- | --------------- |
| `accent.primary`   | `#e135ff` | `225, 53, 255`  |
| `accent.secondary` | `#80ffea` | `128, 255, 234` |
| `accent.tertiary`  | `#ff6ac1` | `255, 106, 193` |
| `success`          | `#50fa7b` | `80, 250, 123`  |
| `error`            | `#ff6363` | `255, 99, 99`   |
| `warning`          | `#f1fa8c` | `241, 250, 140` |
| `text.primary`     | `#f8f8f2` | `248, 248, 242` |
| `bg.base`          | `#121218` | `18, 18, 24`    |

### SilkCircuit Soft

Muted variant with lower saturation for extended use.

### SilkCircuit Glow

Enhanced brightness with glowing accent colors.

### SilkCircuit Vibrant

Maximum saturation for high-contrast displays.

### SilkCircuit Dawn (Light)

Light variant with warm pastel tones.

## Creating Custom Themes

### Theme File Structure

```toml
[meta]
name = "My Theme"
author = "Your Name"
variant = "dark"  # or "light"

[palette]
purple_500 = "#e135ff"
cyan_400 = "#80ffea"

[tokens]
"text.primary" = "#f8f8f2"
"accent.primary" = "purple_500"  # Reference palette

[styles]
keyword = { fg = "accent.primary", bold = true }

[gradients]
primary = ["purple_500", "cyan_400"]
```

### Theme Location

Place custom themes in:

```
~/.config/git-iris/themes/mytheme.toml
```

Then load with:

```bash
git-iris config --theme mytheme
```

## Terminal Requirements

### True Color Support

Git-Iris uses 24-bit true color (RGB). Ensure your terminal supports it:

```bash
# Test true color support
printf "\x1b[38;2;255;100;0mTRUECOLOR\x1b[0m\n"
```

If you see "TRUECOLOR" in orange, your terminal supports it.

### Recommended Terminals

| Terminal         | Support                     |
| ---------------- | --------------------------- |
| iTerm2           | Full                        |
| Alacritty        | Full                        |
| WezTerm          | Full                        |
| Kitty            | Full                        |
| Terminal.app     | Full (macOS 10.15+)         |
| Windows Terminal | Full                        |
| tmux             | Full (with `tmux-256color`) |

### Tmux Configuration

Add to `~/.tmux.conf`:

```bash
set -g default-terminal "tmux-256color"
set -ga terminal-overrides ",*256col*:Tc"
```

## Token Usage Examples

### In Custom Themes

```toml
[tokens]
"accent.primary" = "#e135ff"
"accent.secondary" = "#80ffea"

[styles]
# Use tokens in styles
my_style = { fg = "accent.primary", bg = "bg.panel", bold = true }
```

### Programmatic Access

Themes are accessed internally via the theme API:

```rust
use git_iris::theme;

let theme = theme::current();
let color = theme.color("accent.primary");
let style = theme.style("keyword");
```

## Quick Reference Tables

### Essential Tokens

| Use Case      | Token              |
| ------------- | ------------------ |
| Headings      | `accent.primary`   |
| Links         | `accent.secondary` |
| Success       | `success`          |
| Errors        | `error`            |
| Warnings      | `warning`          |
| Added code    | `diff.added`       |
| Removed code  | `diff.removed`     |
| File paths    | `code.path`        |
| Commit hashes | `code.hash`        |

### Panel-Specific

| Panel            | Focused Border     | Background     |
| ---------------- | ------------------ | -------------- |
| Any              | `border.focused`   | `bg.panel`     |
| Any (unfocused)  | `border.unfocused` | `bg.panel`     |
| Code view        | `border.focused`   | `bg.code`      |
| Highlighted line | -                  | `bg.highlight` |
