# Creating Custom Themes

This guide walks you through creating custom themes for Git-Iris, from basic color changes to advanced gradient definitions.

## Quick Start

### 1. Create Your Theme File

Themes are stored in `~/.config/git-iris/themes/` as TOML files:

```bash
mkdir -p ~/.config/git-iris/themes
touch ~/.config/git-iris/themes/my-theme.toml
```

### 2. Define Metadata

Every theme starts with metadata:

```toml
[meta]
name = "My Custom Theme"
author = "Your Name"
variant = "dark"  # or "light"
version = "1.0"
description = "A brief description of your theme"
```

**Required fields:**

- `name` — Display name shown in theme selector
- All other fields are optional but recommended

**Variant:**

- `dark` — Dark background theme (default)
- `light` — Light background theme

### 3. Define Your Palette

The palette contains raw color primitives referenced by tokens:

```toml
[palette]
# Brand colors
primary = "#ff00ff"
secondary = "#00ffff"
accent = "#ff6ac1"

# Backgrounds
bg_dark = "#1a1a2e"
bg_light = "#25254a"

# Text colors
text_bright = "#ffffff"
text_dim = "#666699"
```

**Color formats:**

- Hex RGB: `"#ff00ff"` or `"#f0f"`
- Lowercase recommended for consistency

**Naming conventions:**

- Use semantic names: `purple_500`, `cyan_400`
- Indicate intensity: `bg_dark`, `bg_light`
- Follow numeric scales: `gray_50` to `gray_950`

### 4. Define Semantic Tokens

Tokens map palette colors to semantic meanings:

```toml
[tokens]
# Text hierarchy
"text.primary" = "text_bright"
"text.secondary" = "text_dim"

# Backgrounds
"bg.base" = "bg_dark"
"bg.panel" = "bg_light"

# Accents
"accent.primary" = "primary"
"accent.secondary" = "secondary"

# Git status
"git.staged" = "#50fa7b"
"git.modified" = "#f1fa8c"
```

**Token types:**

- Palette references: `"primary"` → looks up `[palette]` key
- Direct hex colors: `"#ff00ff"` → inline color definition
- Token references: See [Token Reference](./tokens.md) for complete list

### 5. Add Your Theme

Once saved, your theme is automatically available:

```bash
# List themes (yours will appear)
git-iris theme list

# Activate your theme
git-iris theme set my-theme

# Preview in Studio
git-iris studio --theme my-theme
```

## Complete Theme Template

Here's a minimal but complete theme template:

```toml
[meta]
name = "My Theme"
author = "Your Name"
variant = "dark"

# ═══════════════════════════════════════════════════════════════════════════════
# Palette — Raw color primitives
# ═══════════════════════════════════════════════════════════════════════════════

[palette]
# Core colors
purple = "#a855f7"
cyan = "#06b6d4"
pink = "#ec4899"
green = "#10b981"
red = "#ef4444"
yellow = "#f59e0b"

# Backgrounds
bg_base = "#0f172a"
bg_panel = "#1e293b"
bg_code = "#1e293b"
bg_highlight = "#334155"

# Text
text_primary = "#f8fafc"
text_secondary = "#cbd5e1"
text_muted = "#94a3b8"

# ═══════════════════════════════════════════════════════════════════════════════
# Tokens — Semantic color assignments
# ═══════════════════════════════════════════════════════════════════════════════

[tokens]
# Text hierarchy
"text.primary" = "text_primary"
"text.secondary" = "text_secondary"
"text.muted" = "text_muted"
"text.dim" = "text_muted"

# Backgrounds
"bg.base" = "bg_base"
"bg.panel" = "bg_panel"
"bg.code" = "bg_code"
"bg.highlight" = "bg_highlight"
"bg.elevated" = "bg_highlight"
"bg.active" = "bg_highlight"
"bg.selection" = "bg_highlight"

# Accent colors
"accent.primary" = "purple"
"accent.secondary" = "cyan"
"accent.tertiary" = "pink"
"accent.deep" = "purple"

# Semantic colors
success = "green"
error = "red"
warning = "yellow"
info = "cyan"

# Git status
"git.staged" = "green"
"git.modified" = "yellow"
"git.untracked" = "text_muted"
"git.deleted" = "red"

# Diff colors
"diff.added" = "green"
"diff.removed" = "red"
"diff.hunk" = "cyan"
"diff.context" = "text_muted"

# UI elements
"border.focused" = "cyan"
"border.unfocused" = "text_muted"

# Code syntax
"code.hash" = "pink"
"code.path" = "cyan"
"code.keyword" = "purple"
"code.function" = "cyan"
"code.string" = "green"
"code.number" = "pink"
"code.comment" = "text_muted"
"code.type" = "yellow"
"code.line_number" = "text_muted"

# Mode tabs
"mode.active" = "purple"
"mode.inactive" = "text_muted"
"mode.hover" = "cyan"

# Chat
"chat.user" = "cyan"
"chat.iris" = "purple"

# ═══════════════════════════════════════════════════════════════════════════════
# Styles — Composed styles with modifiers (optional)
# ═══════════════════════════════════════════════════════════════════════════════

[styles]
keyword = { fg = "accent.primary", bold = true }
file_path = { fg = "code.path" }
selected = { fg = "accent.secondary", bg = "bg.highlight" }

# ═══════════════════════════════════════════════════════════════════════════════
# Gradients — Color transitions (optional)
# ═══════════════════════════════════════════════════════════════════════════════

[gradients]
primary = ["purple", "cyan"]
warm = ["pink", "yellow"]
```

## Advanced Techniques

### Token Chaining

Tokens can reference other tokens for consistency:

```toml
[palette]
purple_500 = "#e135ff"

[tokens]
"accent.primary" = "purple_500"
"mode.active" = "accent.primary"      # References accent.primary
"border.focused" = "mode.active"      # References mode.active
```

All three tokens resolve to `#e135ff`, but you can change the entire chain by updating `purple_500`.

### Custom Styles

Define composed styles with foreground, background, and modifiers:

```toml
[styles]
# Bold keyword
keyword = { fg = "accent.primary", bold = true }

# Highlighted selection
selected = { fg = "accent.secondary", bg = "bg.highlight" }

# Dimmed text
muted = { fg = "text.muted", dim = true }

# Italic comments
comment = { fg = "code.comment", italic = true }

# Underlined links
link = { fg = "accent.secondary", underline = true }

# Complex combination
error_highlight = { fg = "error", bg = "bg.highlight", bold = true }
```

**Available modifiers:**

- `bold` — Bold text
- `italic` — Italic text
- `underline` — Underlined text
- `dim` — Dimmed/faint text

### Multi-Stop Gradients

Create smooth color transitions with multiple stops:

```toml
[gradients]
# Two-stop gradient (simple)
primary = ["purple_500", "cyan_400"]

# Three-stop gradient (middle accent)
warm = ["coral_400", "yellow_400", "green_400"]

# Five-stop gradient (complex)
rainbow = ["#ff0000", "#ff7f00", "#ffff00", "#00ff00", "#0000ff"]

# Aurora gradient (signature SilkCircuit sweep)
aurora = ["purple_500", "#f31bff", "#ff00ff", "#bf80f4", "cyan_400"]
```

Gradients interpolate smoothly between stops. Access with:

```rust
// In Rust code
let color = theme.gradient("aurora", 0.5);  // Midpoint color

// Generate N evenly-spaced colors
let colors = theme.get_gradient("aurora").unwrap().generate(10);
```

### Light Theme Considerations

When creating light themes:

```toml
[meta]
variant = "light"

[palette]
# Darker accent colors for contrast
purple = "#7e2bd5"
cyan = "#007f8e"

# Light backgrounds
bg_base = "#faf8ff"
bg_panel = "#f1ecff"
bg_code = "#efeaff"

# Dark text
text_primary = "#2b2540"
text_secondary = "#3d3558"
text_muted = "#5a4d78"
```

**Tips:**

- Use darker, more saturated accent colors
- Ensure sufficient contrast (WCAG AA: 4.5:1 minimum)
- Test with syntax highlighting
- Consider ambient light conditions

## Best Practices

### Color Psychology

Choose colors that convey the right meaning:

| Semantic | Traditional Color | Reason                       |
| -------- | ----------------- | ---------------------------- |
| Success  | Green             | Universal positive indicator |
| Error    | Red               | Universal danger/stop signal |
| Warning  | Yellow/Orange     | Caution without severity     |
| Info     | Cyan/Blue         | Neutral, informative         |
| Modified | Yellow            | Changed, needs attention     |
| Staged   | Green             | Ready for commit (positive)  |
| Deleted  | Red               | Removal (negative)           |

### Accessibility

Ensure your theme is accessible:

1. **Contrast ratios**
   - Normal text: 4.5:1 minimum (WCAG AA)
   - Large text: 3:1 minimum
   - Use tools like [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)

2. **Color blindness**
   - Don't rely solely on color to convey information
   - Test with simulators (Coblis, ColorOracle)
   - Consider red-green color blindness (most common)

3. **Brightness**
   - Avoid pure black (`#000000`) backgrounds
   - Avoid pure white (`#ffffff`) on dark themes
   - Use slightly off-black/off-white for reduced eye strain

### Consistency

Maintain visual hierarchy:

```toml
# Text hierarchy (decreasing contrast)
"text.primary" = "gray_50"    # Highest contrast
"text.secondary" = "gray_200"  # Medium contrast
"text.muted" = "gray_400"      # Low contrast
"text.dim" = "gray_500"        # Lowest contrast
```

### Performance

- Themes load instantly (compiled into binary)
- No runtime performance impact
- Feel free to define many palette colors
- Gradients are computed on-demand

## Testing Your Theme

### Visual Testing

1. **Preview in Studio**

   ```bash
   git-iris studio --theme my-theme
   ```

2. **Cycle through modes**
   - Explore mode: Test file tree, code view
   - Commit mode: Test diff colors, git status
   - Review mode: Test syntax highlighting
   - PR mode: Test markdown rendering

3. **Test edge cases**
   - Long commit messages
   - Large diffs
   - Empty states
   - Error messages

### Validation

Git-Iris validates themes at load time:

```bash
# If invalid, you'll see helpful errors
git-iris theme set my-broken-theme

# Error: Theme validation failed
#   Missing required token: "accent.primary"
#   Invalid color reference: "nonexistent_color"
#   Invalid hex color: "#gggggg"
```

**Common issues:**

- Missing required tokens
- Invalid color references
- Malformed hex colors
- Circular token references

### Iterative Refinement

1. **Start simple** — Copy a builtin theme and modify colors
2. **Test frequently** — Preview after each major change
3. **Compare themes** — Switch between yours and builtins
4. **Get feedback** — Share with others for fresh perspectives
5. **Refine gradually** — Small tweaks compound over time

## Sharing Your Theme

### Export Your Theme

```bash
# Your theme is already in a shareable location
cat ~/.config/git-iris/themes/my-theme.toml
```

### Contribute to Git-Iris

To add your theme to the builtin collection:

1. Create a pull request with your theme TOML
2. Place it in `src/theme/builtins/`
3. Update `src/theme/builtins/mod.rs`
4. Add tests for color validation
5. Include screenshots in the PR

See the [Contributing Guide](/extending/contributing) for guidelines.

### Community Themes

Share your themes:

- GitHub Gists
- Git-Iris discussions
- Reddit r/unixporn
- Terminal theme repositories

## Examples

### Monochrome Theme

```toml
[meta]
name = "Grayscale"
variant = "dark"

[palette]
gray_50 = "#f5f5f5"
gray_400 = "#9ca3af"
gray_700 = "#374151"
gray_900 = "#111827"

[tokens]
"text.primary" = "gray_50"
"text.muted" = "gray_400"
"bg.base" = "gray_900"
"bg.panel" = "gray_700"
"accent.primary" = "gray_50"
"accent.secondary" = "gray_400"
# ... (all other tokens use grayscale)
```

### High Contrast Theme

```toml
[meta]
name = "Maximum Contrast"
variant = "dark"

[palette]
white = "#ffffff"
black = "#000000"
pure_cyan = "#00ffff"
pure_magenta = "#ff00ff"

[tokens]
"text.primary" = "white"
"bg.base" = "black"
"accent.primary" = "pure_magenta"
"accent.secondary" = "pure_cyan"
# ... (pure colors only)
```

### Pastel Theme

```toml
[meta]
name = "Soft Pastels"
variant = "light"

[palette]
pastel_purple = "#dcc9ff"
pastel_pink = "#ffd9e8"
pastel_blue = "#c9f0ff"
pastel_green = "#d4f4dd"
cream = "#fffef9"

[tokens]
"bg.base" = "cream"
"accent.primary" = "pastel_purple"
"accent.secondary" = "pastel_blue"
# ... (soft, muted colors)
```

## Troubleshooting

### Theme Not Appearing

```bash
# Check theme file location
ls ~/.config/git-iris/themes/

# Verify file has .toml extension
mv my-theme.txt my-theme.toml

# Check for TOML syntax errors
git-iris theme set my-theme
```

### Colors Look Wrong

- Verify terminal true color support: `echo $COLORTERM` should be `truecolor`
- Check terminal emulator settings
- Test in different terminals (iTerm2, Alacritty, WezTerm)
- Verify hex colors are valid RGB

### Missing Tokens Error

See [Token Reference](./tokens.md) for the complete list of required tokens. All semantic tokens must be defined.

---

**Next Steps:**

- [Explore semantic tokens](./tokens.md)
- [Learn about styles and gradients](./styles.md)
- [View theme gallery](./gallery.md)
