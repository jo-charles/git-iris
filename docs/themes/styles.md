# Styles & Gradients

Advanced styling features in the Git-Iris theme system—composed styles with modifiers and smooth color gradients.

## Composed Styles

Styles combine colors with text modifiers (bold, italic, underline, dim) to create reusable, semantic style definitions.

### Basic Style Definition

```toml
[styles]
keyword = { fg = "accent.primary", bold = true }
```

This creates a style named `keyword` that:

- Uses `accent.primary` token for foreground color
- Applies bold text modifier

### Style Properties

Every style supports these properties:

| Property    | Type        | Description             | Example                      |
| ----------- | ----------- | ----------------------- | ---------------------------- |
| `fg`        | Token/Color | Foreground (text) color | `"accent.primary"`, `"#fff"` |
| `bg`        | Token/Color | Background color        | `"bg.highlight"`, `"#000"`   |
| `bold`      | Boolean     | Bold text               | `true`, `false`              |
| `italic`    | Boolean     | Italic text             | `true`, `false`              |
| `underline` | Boolean     | Underlined text         | `true`, `false`              |
| `dim`       | Boolean     | Dimmed/faint text       | `true`, `false`              |

All properties are optional—omitted properties remain unset.

### Complete Style Examples

#### Simple Foreground Color

```toml
[styles]
file_path = { fg = "code.path" }
```

Renders file paths in cyan without any modifiers.

#### Foreground + Background

```toml
[styles]
selected = { fg = "accent.secondary", bg = "bg.highlight" }
```

Cyan text on highlighted background for selections.

#### Multiple Modifiers

```toml
[styles]
error_highlight = {
    fg = "error",
    bg = "bg.highlight",
    bold = true,
    underline = true
}
```

Bold, underlined red text on highlighted background—maximum emphasis.

#### Background Only

```toml
[styles]
cursor_line = { bg = "bg.highlight" }
```

Just a background color, text inherits from parent.

#### Text Modifiers Only

```toml
[styles]
emphasis = { bold = true, italic = true }
```

No colors, just modifiers—useful for layering.

## Builtin Styles

SilkCircuit themes define these standard styles:

### Text Emphasis

```toml
[styles]
# Bold keyword
keyword = { fg = "accent.primary", bold = true }

# Dimmed text
dimmed = { fg = "text.dim" }

# Muted text
muted = { fg = "text.muted" }
```

### File Paths

```toml
[styles]
# Normal path
file_path = { fg = "code.path" }

# Emphasized path
file_path_bold = { fg = "code.path", bold = true }
```

### Commit Hashes

```toml
[styles]
commit_hash = { fg = "code.hash" }
```

### Line Numbers

```toml
[styles]
line_number = { fg = "code.line_number" }
```

### Cursor and Selection

```toml
[styles]
# Cursor line background
cursor_line = { bg = "bg.highlight" }

# Selected item
selected = { fg = "accent.secondary", bg = "bg.highlight" }

# Active selected item
active_selected = { fg = "accent.primary", bg = "bg.active", bold = true }
```

### Borders

```toml
[styles]
focused_border = { fg = "border.focused" }
unfocused_border = { fg = "border.unfocused" }
```

### Status Messages

```toml
[styles]
success_style = { fg = "success" }
error_style = { fg = "error" }
warning_style = { fg = "warning" }
info_style = { fg = "info" }
```

### Code Elements

```toml
[styles]
inline_code = { fg = "success", bg = "bg.code" }
```

### Mode Tabs

```toml
[styles]
mode_active = { fg = "mode.active", bold = true }
mode_inactive = { fg = "mode.inactive" }
mode_hover = { fg = "mode.hover" }
```

### Git Status

```toml
[styles]
git_staged = { fg = "git.staged" }
git_modified = { fg = "git.modified" }
git_untracked = { fg = "git.untracked" }
git_deleted = { fg = "git.deleted" }
```

### Diff Syntax

```toml
[styles]
diff_added = { fg = "diff.added" }
diff_removed = { fg = "diff.removed" }
diff_hunk = { fg = "diff.hunk" }
diff_context = { fg = "diff.context" }
```

### Metadata

```toml
[styles]
author = { fg = "text.primary" }
timestamp = { fg = "warning" }
```

## Style Usage in Code

Styles are accessed via the theme API:

```rust
use git_iris::theme;

let theme = theme::current();

// Get a style
let keyword_style = theme.style("keyword");

// Convert to Ratatui style
use git_iris::theme::adapters::ratatui::to_ratatui_style;
let ratatui_style = to_ratatui_style(&keyword_style);

// Render with Ratatui
Span::styled("fn", ratatui_style)
```

Styles are automatically applied throughout the UI based on semantic context.

## Color Gradients

Gradients enable smooth color transitions between multiple stops. Perfect for progress bars, status indicators, and decorative elements.

### Basic Gradient Definition

```toml
[gradients]
primary = ["purple_500", "cyan_400"]
```

This creates a two-stop gradient that interpolates smoothly from purple to cyan.

### Gradient Syntax

```toml
[gradients]
gradient_name = ["color1", "color2", "color3", ...]
```

- **Color references**: Can be palette names (`"purple_500"`) or direct hex (`"#e135ff"`)
- **Stop count**: Minimum 1, no maximum (though 2-5 is typical)
- **Interpolation**: Linear RGB interpolation between stops

### Multi-Stop Gradients

#### Two Stops (Simple)

```toml
[gradients]
primary = ["purple_500", "cyan_400"]
```

**Color progression:**

```
0.0   0.25   0.5   0.75   1.0
█      █      █      █      █
Purple -----> Blend -----> Cyan
```

#### Three Stops (Accent)

```toml
[gradients]
warm = ["coral_400", "yellow_400", "green_400"]
```

**Color progression:**

```
0.0     0.25    0.5     0.75    1.0
█        █       █        █       █
Coral -> Yellow -> Green
```

#### Five Stops (Complex)

```toml
[gradients]
aurora = ["purple_500", "#f31bff", "#ff00ff", "#bf80f4", "cyan_400"]
```

**Color progression:**

```
0.0    0.25   0.5    0.75   1.0
█       █      █       █      █
Purple  Pink1  Pink2  Lavender Cyan
```

Creates a smooth rainbow-like sweep across five distinct colors.

### Builtin Gradients

SilkCircuit themes include these standard gradients:

#### Primary Brand Gradient

```toml
[gradients]
primary = ["purple_500", "cyan_400"]
```

The signature SilkCircuit gradient—electric purple to neon cyan. Used for:

- Brand elements
- Primary actions
- Loading states
- Decorative accents

#### Warm Accent Gradient

```toml
[gradients]
warm = ["coral_400", "yellow_400"]
```

Coral to yellow transition. Used for:

- Warning states
- Energy indicators
- Warm highlights

#### Success Gradient

```toml
[gradients]
success_gradient = ["green_400", "cyan_400"]
```

Green to cyan transition. Used for:

- Success states
- Progress indicators
- Positive feedback

#### Error Gradient

```toml
[gradients]
error_gradient = ["red_400", "coral_400"]
```

Red to coral transition. Used for:

- Error states
- Danger indicators
- Negative feedback

#### Aurora Gradient

```toml
[gradients]
aurora = ["purple_500", "#f31bff", "#ff00ff", "#bf80f4", "cyan_400"]
```

Five-stop signature gradient. Used for:

- Decorative elements
- Splash screens
- Brand showcases

## Gradient Usage

### Get Color at Position

```rust
use git_iris::theme;

let theme = theme::current();

// Get color at 50% through gradient
let color = theme.gradient("primary", 0.5);

// Render at specific position (0.0 to 1.0)
let start = theme.gradient("primary", 0.0);   // Purple
let mid = theme.gradient("primary", 0.5);     // Purple-cyan blend
let end = theme.gradient("primary", 1.0);     // Cyan
```

### Generate Gradient Steps

```rust
// Generate 10 evenly-spaced colors
let gradient = theme.get_gradient("aurora").unwrap();
let colors = gradient.generate(10);

// colors[0] = purple_500
// colors[5] = midpoint color
// colors[9] = cyan_400
```

Perfect for creating smooth transitions in progress bars, charts, or animations.

### Interpolation Algorithm

Git-Iris uses **linear RGB interpolation**:

```
For gradient [C1, C2] at position t (0.0 to 1.0):

result.r = C1.r + (C2.r - C1.r) * t
result.g = C1.g + (C2.g - C1.g) * t
result.b = C1.b + (C2.b - C1.b) * t
```

For multi-stop gradients, the position is mapped to the appropriate segment.

## Advanced Techniques

### Style Layering

Combine styles for complex effects:

```toml
[styles]
# Base style
base_text = { fg = "text.primary" }

# Layer with emphasis
emphasized = { bold = true, italic = true }

# Layer with background
highlighted = { bg = "bg.highlight" }
```

In code, merge styles:

```rust
let base = theme.style("base_text");
let emphasis = theme.style("emphasized");
let combined = base.merge(&emphasis);
```

### Conditional Styling

Apply different styles based on state:

```toml
[styles]
normal = { fg = "text.primary" }
hovered = { fg = "accent.secondary", bg = "bg.highlight" }
selected = { fg = "accent.primary", bg = "bg.active", bold = true }
```

### Gradient-Based Styles

Use gradient colors in styles:

```rust
// Get color from gradient
let color = theme.gradient("primary", 0.3);

// Create dynamic style
let style = ThemeStyle::fg(color).bold();
```

Useful for progress indicators that change color as they fill.

### Semantic Style Names

Name styles by purpose, not appearance:

**Good:**

```toml
[styles]
keyword = { fg = "accent.primary", bold = true }
selected_item = { fg = "accent.secondary", bg = "bg.highlight" }
error_message = { fg = "error", bold = true }
```

**Poor:**

```toml
[styles]
purple_bold = { fg = "#e135ff", bold = true }
cyan_highlight = { fg = "#80ffea", bg = "#37324b" }
red_text = { fg = "#ff6363", bold = true }
```

Semantic names remain valid even if colors change.

## Style Modifiers Reference

### Bold

**Effect:** Increases font weight

**Use for:**

- Keywords
- Headings
- Active states
- Emphasis

**Accessibility:** Improves scannability, works well for highlighting

```toml
keyword = { bold = true }
```

### Italic

**Effect:** Slants text

**Use for:**

- Comments
- Quotes
- Subtle emphasis
- Metadata

**Accessibility:** Less readable than bold, use sparingly

```toml
comment = { italic = true }
```

### Underline

**Effect:** Adds line below text

**Use for:**

- Links
- Important warnings
- Current item indicator

**Accessibility:** Strong visual cue, but can clutter

```toml
link = { underline = true }
```

### Dim

**Effect:** Reduces brightness/intensity

**Use for:**

- Disabled items
- Placeholder text
- Low-priority content

**Accessibility:** Reduces contrast, avoid for critical info

```toml
disabled = { dim = true }
```

### Combining Modifiers

Multiple modifiers can be applied:

```toml
critical = { bold = true, underline = true }
subtle_emphasis = { italic = true, dim = true }
maximum_attention = { bold = true, italic = true, underline = true }
```

**Best practice:** Limit to 2 modifiers maximum to avoid visual noise.

## Performance Considerations

### Style Resolution

- Styles are resolved once at theme load time
- No runtime performance cost
- Feel free to define many styles

### Gradient Computation

- Gradients interpolate on-demand
- Linear interpolation is fast (few CPU cycles)
- Pre-generate if using in tight loops:

```rust
// Generate once
let colors = gradient.generate(100);

// Use cached colors in loop
for (i, item) in items.iter().enumerate() {
    let color = colors[i % colors.len()];
    render_with_color(item, color);
}
```

## Common Patterns

### Status Indicators

```toml
[styles]
status_ok = { fg = "success" }
status_warn = { fg = "warning", bold = true }
status_error = { fg = "error", bold = true, underline = true }
```

Increasing emphasis for severity.

### Selection States

```toml
[styles]
unselected = { fg = "text.secondary" }
hovered = { fg = "accent.secondary" }
selected = { fg = "accent.primary", bold = true }
active = { fg = "accent.primary", bg = "bg.active", bold = true }
```

Progressive visual feedback.

### Code Syntax

```toml
[styles]
syntax_keyword = { fg = "code.keyword", bold = true }
syntax_string = { fg = "code.string" }
syntax_number = { fg = "code.number" }
syntax_comment = { fg = "code.comment", italic = true }
syntax_type = { fg = "code.type" }
```

Familiar syntax highlighting patterns.

## Troubleshooting

### Style Not Applying

**Check style name:**

```rust
// Correct
theme.style("keyword")

// Wrong (typo)
theme.style("keywrod")
```

**Verify style exists:**

```rust
if theme.has_style("keyword") {
    let style = theme.style("keyword");
} else {
    // Fallback to default
    let style = ThemeStyle::default();
}
```

### Gradient Not Rendering

**Verify gradient exists:**

```rust
if let Some(gradient) = theme.get_gradient("primary") {
    let color = gradient.at(0.5);
} else {
    // Fallback color
    let color = theme.color("accent.primary");
}
```

**Check position range:**

```rust
// Positions outside 0.0-1.0 are clamped automatically
let color = gradient.at(1.5);  // Clamped to 1.0
```

### Terminal Rendering Issues

**Modifiers not showing:**

- Some terminals don't support all modifiers
- Test in iTerm2, Alacritty, or WezTerm
- Check `$TERM` environment variable

**Colors look different:**

- Ensure true color support: `export COLORTERM=truecolor`
- Some terminals apply color profiles
- Test in different terminals to compare

## Examples

### Progress Bar Gradient

```toml
[gradients]
progress = ["#ff6363", "#f1fa8c", "#50fa7b"]

# Red (0%) -> Yellow (50%) -> Green (100%)
```

```rust
fn render_progress(percent: f32) {
    let color = theme.gradient("progress", percent / 100.0);
    // Render bar with dynamic color
}
```

### Diff Line Styles

```toml
[styles]
diff_added_line = { fg = "diff.added" }
diff_removed_line = { fg = "diff.removed" }
diff_hunk_header = { fg = "diff.hunk", bold = true }
diff_context_line = { fg = "diff.context" }
```

### Interactive States

```toml
[styles]
button_normal = { fg = "text.primary" }
button_hover = { fg = "accent.secondary", underline = true }
button_active = { fg = "accent.primary", bg = "bg.active", bold = true }
button_disabled = { fg = "text.dim", dim = true }
```

---

**Next Steps:**

- [View complete token reference](./tokens.md)
- [Create your custom theme](./creating.md)
- [Explore theme gallery](./gallery.md)
