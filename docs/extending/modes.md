# Adding Studio Modes

Studio modes are interactive TUI interfaces for specific workflows. Each mode combines state management, input handling, and rendering to create a focused user experience. This guide shows you how to add a new mode to Iris Studio.

## What is a Studio Mode?

A mode is a complete user interface for a specific task:

- **Explore Mode**: Navigate codebase with semantic understanding
- **Commit Mode**: Generate and edit commit messages
- **Review Mode**: AI-powered code reviews
- **PR Mode**: Pull request descriptions
- **Changelog Mode**: Structured changelog generation

Each mode has:

1. **State** — Data specific to this mode
2. **Handler** — Input processing logic
3. **Renderer** — UI drawing code

## Architecture: Pure Reducer Pattern

Studio uses a predictable state management pattern:

```
Input Event
    ↓
Handler (maps input → StudioEvent)
    ↓
Reducer (pure function: state + event → new state + side effects)
    ↓
Side Effects (spawn agent, load data, etc.)
    ↓
State Updated
    ↓
Renderer (draw UI from state)
```

**Key principle**: State transitions are pure functions. Side effects are returned as data, not executed directly.

## Step-by-Step: Adding a New Mode

### Example: Feature Summary Mode

::: tip Teaching Example
This section walks through creating a hypothetical "Feature Summary" mode. **This mode does not exist in the current codebase** — it's a complete example to illustrate the pattern. Follow along to understand how modes work, then apply the same structure to your own mode.
:::

Let's create a mode that displays AI-generated feature summaries.

### Step 1: Add Mode Variant

Edit `src/studio/state/mod.rs`:

```rust
/// Available modes in Iris Studio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Explore,
    Commit,
    Review,
    PR,
    Changelog,
    ReleaseNotes,
    FeatureSummary,  // Add your mode
}

impl Mode {
    pub fn display_name(&self) -> &'static str {
        match self {
            // ... existing modes ...
            Mode::FeatureSummary => "Feature Summary",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            // ... existing modes ...
            Mode::FeatureSummary => 'F',
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(
            self,
            Mode::Explore
                | Mode::Commit
                | Mode::Review
                | Mode::PR
                | Mode::Changelog
                | Mode::ReleaseNotes
                | Mode::FeatureSummary  // Mark as available
        )
    }

    pub fn all() -> &'static [Mode] {
        &[
            Mode::Explore,
            Mode::Commit,
            Mode::Review,
            Mode::PR,
            Mode::Changelog,
            Mode::ReleaseNotes,
            Mode::FeatureSummary,  // Add to list
        ]
    }
}
```

### Step 2: Create State Struct

Edit `src/studio/state/modes.rs`:

```rust
/// Feature Summary mode state
#[derive(Debug, Clone, Default)]
pub struct FeatureSummaryMode {
    /// Base branch to compare against
    pub from_ref: String,
    /// Feature branch to summarize
    pub to_ref: String,
    /// Generated summary content
    pub summary_content: String,
    /// Whether we're currently generating
    pub generating: bool,
    /// Scroll offset for summary view
    pub scroll_offset: usize,
    /// Panel state for file list (if showing files)
    pub file_list: Vec<String>,
    pub file_list_selected: usize,
}

impl FeatureSummaryMode {
    pub fn new() -> Self {
        Self {
            from_ref: "main".to_string(),
            to_ref: "HEAD".to_string(),
            ..Default::default()
        }
    }

    /// Scroll summary down
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    /// Scroll summary up
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Select next file in list
    pub fn select_next_file(&mut self) {
        if !self.file_list.is_empty() {
            self.file_list_selected = (self.file_list_selected + 1) % self.file_list.len();
        }
    }

    /// Select previous file in list
    pub fn select_prev_file(&mut self) {
        if !self.file_list.is_empty() && self.file_list_selected > 0 {
            self.file_list_selected -= 1;
        } else if !self.file_list.is_empty() {
            self.file_list_selected = self.file_list.len() - 1;
        }
    }
}
```

Add to `ModeStates`:

```rust
/// Container for all mode-specific states
#[derive(Debug, Default)]
pub struct ModeStates {
    pub explore: ExploreMode,
    pub commit: CommitMode,
    pub review: ReviewMode,
    pub pr: PRMode,
    pub changelog: ChangelogMode,
    pub release_notes: ReleaseNotesMode,
    pub feature_summary: FeatureSummaryMode,  // Add here
}
```

### Step 3: Create Input Handler

Create `src/studio/handlers/feature_summary.rs`:

```rust
//! Feature Summary mode key handling

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, PanelId, RefSelectorTarget, StudioState};

use super::copy_to_clipboard;

/// Handle key events in Feature Summary mode
pub fn handle_feature_summary_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match state.focused_panel {
        PanelId::Left => handle_files_key(state, key),
        PanelId::Center => handle_summary_key(state, key),
        PanelId::Right => handle_diff_key(state, key),
    }
}

fn handle_files_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.feature_summary.select_next_file();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.feature_summary.select_prev_file();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Enter => {
            // Switch to diff view for selected file
            state.focused_panel = PanelId::Right;
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}

fn handle_summary_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Scrolling
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.feature_summary.scroll_down(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.feature_summary.scroll_up(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.feature_summary.scroll_down(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.feature_summary.scroll_up(20);
            state.mark_dirty();
            vec![]
        }

        // Generate summary
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating feature summary...");
            state.modes.feature_summary.generating = true;
            vec![spawn_feature_summary_task(state)]
        }

        // Select base branch
        KeyCode::Char('b') => {
            let refs = state.get_branch_refs();
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs,
                selected: 0,
                target: RefSelectorTarget::FeatureSummaryFrom,
            });
            state.mark_dirty();
            vec![]
        }

        // Select feature branch
        KeyCode::Char('f') => {
            let refs = state.get_branch_refs();
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs,
                selected: 0,
                target: RefSelectorTarget::FeatureSummaryTo,
            });
            state.mark_dirty();
            vec![]
        }

        // Copy to clipboard
        KeyCode::Char('y') => {
            let content = &state.modes.feature_summary.summary_content;
            if !content.is_empty() {
                copy_to_clipboard(state, content, "Feature summary");
            }
            vec![]
        }

        _ => vec![],
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            // Scroll diff view
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}

/// Spawn task to generate feature summary
fn spawn_feature_summary_task(state: &StudioState) -> SideEffect {
    let from_ref = state.modes.feature_summary.from_ref.clone();
    let to_ref = state.modes.feature_summary.to_ref.clone();
    let config = state.config.clone();

    SideEffect::SpawnAgent {
        task: Box::pin(async move {
            use crate::agents::setup::IrisAgentService;

            let service = IrisAgentService::new(config)?;
            let response = service
                .execute_capability("feature_summary", &[
                    ("from_ref", &from_ref),
                    ("to_ref", &to_ref),
                ])
                .await?;

            // Return the summary content
            Ok(response.to_string())
        }),
    }
}
```

Add to `src/studio/handlers/mod.rs`:

```rust
pub mod feature_summary;
pub use feature_summary::handle_feature_summary_key;
```

Update main handler to dispatch to your mode:

```rust
// In src/studio/handlers/global.rs or main handler
match state.active_mode {
    // ... existing modes ...
    Mode::FeatureSummary => handle_feature_summary_key(state, key),
}
```

### Step 4: Create Renderer

Create `src/studio/render/feature_summary.rs`:

```rust
//! Feature Summary mode rendering

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

/// Render a panel in Feature Summary mode
pub fn render_feature_summary_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;
    let theme = theme::current();

    match panel_id {
        PanelId::Left => {
            // File list panel
            render_file_list(state, frame, area, is_focused, &theme);
        }
        PanelId::Center => {
            // Summary content panel
            render_summary(state, frame, area, is_focused, &theme);
        }
        PanelId::Right => {
            // Diff view panel (optional)
            render_diff(state, frame, area, is_focused, &theme);
        }
    }
}

fn render_file_list(
    state: &StudioState,
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    theme: &theme::Theme,
) {
    let files = &state.modes.feature_summary.file_list;
    let selected = state.modes.feature_summary.file_list_selected;

    let items: Vec<ListItem> = files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.colors.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.colors.text)
            };

            let marker = if i == selected { "▸" } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::raw(" "),
                Span::styled(file, style),
            ]))
        })
        .collect();

    let border_style = if is_focused {
        Style::default().fg(theme.colors.accent)
    } else {
        Style::default().fg(theme.colors.border)
    };

    let title = format!("Files · {}", files.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_summary(
    state: &StudioState,
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    theme: &theme::Theme,
) {
    let content = &state.modes.feature_summary.summary_content;
    let scroll = state.modes.feature_summary.scroll_offset;

    let border_style = if is_focused {
        Style::default().fg(theme.colors.accent)
    } else {
        Style::default().fg(theme.colors.border)
    };

    // Build title with refs
    let from = &state.modes.feature_summary.from_ref;
    let to = &state.modes.feature_summary.to_ref;
    let title = format!("Summary · {} → {}", from, to);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render content with scrolling
    if content.is_empty() {
        let placeholder = if state.modes.feature_summary.generating {
            "Generating feature summary..."
        } else {
            "Press 'r' to generate feature summary\n\
             Press 'b' to select base branch\n\
             Press 'f' to select feature branch"
        };

        let para = Paragraph::new(placeholder)
            .style(Style::default().fg(theme.colors.text_dim))
            .wrap(Wrap { trim: false });

        frame.render_widget(para, inner);
    } else {
        // Render markdown content (simplified - use proper markdown rendering in real impl)
        let lines: Vec<Line> = content
            .lines()
            .skip(scroll)
            .take(inner.height as usize)
            .map(|line| Line::from(line))
            .collect();

        let para = Paragraph::new(lines)
            .style(Style::default().fg(theme.colors.text))
            .wrap(Wrap { trim: false });

        frame.render_widget(para, inner);
    }
}

fn render_diff(
    state: &StudioState,
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    theme: &theme::Theme,
) {
    let border_style = if is_focused {
        Style::default().fg(theme.colors.accent)
    } else {
        Style::default().fg(theme.colors.border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Diff");

    // Render diff for selected file (simplified)
    let placeholder = Paragraph::new("Diff view")
        .block(block)
        .style(Style::default().fg(theme.colors.text_dim));

    frame.render_widget(placeholder, area);
}
```

Add to `src/studio/render/mod.rs`:

```rust
pub mod feature_summary;
pub use feature_summary::render_feature_summary_panel;
```

Update main renderer to use your mode:

```rust
// In src/studio/render/mod.rs or main render function
match state.active_mode {
    // ... existing modes ...
    Mode::FeatureSummary => {
        render_feature_summary_panel(state, frame, left_area, PanelId::Left);
        render_feature_summary_panel(state, frame, center_area, PanelId::Center);
        render_feature_summary_panel(state, frame, right_area, PanelId::Right);
    }
}
```

### Step 5: Add Side Effects

Edit `src/studio/events.rs` to add any new side effects:

```rust
#[derive(Debug)]
pub enum SideEffect {
    // ... existing effects ...

    /// Generate feature summary
    GenerateFeatureSummary {
        from_ref: String,
        to_ref: String,
    },
}
```

Handle in reducer (if using specialized effects instead of generic `SpawnAgent`):

```rust
// In src/studio/reducer.rs
match effect {
    // ... existing effects ...
    SideEffect::GenerateFeatureSummary { from_ref, to_ref } => {
        // Spawn agent task
    }
}
```

### Step 6: Update Focus Defaults

Edit `src/studio/state/mod.rs` in the `switch_mode` method:

```rust
pub fn switch_mode(&mut self, new_mode: Mode) {
    // ... existing code ...

    // Set default focus based on mode
    self.focused_panel = match new_mode {
        Mode::Commit => PanelId::Center,
        Mode::Review | Mode::PR | Mode::Changelog | Mode::ReleaseNotes => PanelId::Center,
        Mode::FeatureSummary => PanelId::Center,  // Add here
        Mode::Explore => PanelId::Left,
    };
    self.dirty = true;
}
```

### Step 7: Test Your Mode

```bash
cargo build
cargo run -- studio
```

In Studio:

- Press `Shift+F` to switch to Feature Summary mode
- Test navigation with `j`/`k`, `Tab`
- Test generating with `r`
- Test branch selection with `b` and `f`

## Component Reuse

Studio provides reusable components in `src/studio/components/`:

### File Tree

```rust
use crate::studio::components::render_file_tree;

render_file_tree(
    frame,
    area,
    &mut state.modes.my_mode.file_tree,
    "Files",
    is_focused,
);
```

### Diff View

```rust
use crate::studio::components::render_diff_view;

render_diff_view(
    frame,
    area,
    &state.modes.my_mode.diff_view,
    "Changes",
    is_focused,
);
```

### Message Editor

```rust
use crate::studio::components::render_message_editor;

render_message_editor(
    frame,
    area,
    &state.modes.my_mode.message_editor,
    "Message",
    is_focused,
    generating,
);
```

### Code View

```rust
use crate::studio::components::render_code_view;

render_code_view(
    frame,
    area,
    &content,
    Some(&language),
    scroll_offset,
    is_focused,
);
```

## Best Practices

### State Design

**Keep state minimal:**

```rust
pub struct MyMode {
    pub essential_data: String,
    pub scroll_offset: usize,
    // Don't store derived data - compute on render
}
```

**Use clear field names:**

```rust
pub struct MyMode {
    pub from_ref: String,      // Good - clear purpose
    pub to_ref: String,         // Good
    pub data: String,           // Bad - vague
    pub temp: Vec<String>,      // Bad - unclear
}
```

### Handler Design

**Return side effects, don't execute:**

```rust
// Good
KeyCode::Char('r') => {
    state.modes.my_mode.generating = true;
    vec![SideEffect::SpawnAgent { task: ... }]
}

// Bad - executes directly
KeyCode::Char('r') => {
    tokio::spawn(async { ... });  // Don't do this!
    vec![]
}
```

**Keep handlers focused:**

```rust
// Good - separate concerns
fn handle_file_list_key(...) -> Vec<SideEffect> { ... }
fn handle_content_key(...) -> Vec<SideEffect> { ... }

pub fn handle_my_mode_key(...) -> Vec<SideEffect> {
    match state.focused_panel {
        PanelId::Left => handle_file_list_key(state, key),
        PanelId::Center => handle_content_key(state, key),
        PanelId::Right => handle_diff_key(state, key),
    }
}
```

### Renderer Design

**Compute dimensions from available space:**

```rust
fn render_my_panel(frame: &mut Frame, area: Rect, ...) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(10),        // Content
            Constraint::Length(1),      // Footer
        ])
        .split(area);

    render_header(frame, chunks[0], ...);
    render_content(frame, chunks[1], ...);
    render_footer(frame, chunks[2], ...);
}
```

**Use theme colors:**

```rust
use crate::studio::theme;

let theme = theme::current();

let text_style = Style::default().fg(theme.colors.text);
let accent_style = Style::default().fg(theme.colors.accent);
let dim_style = Style::default().fg(theme.colors.text_dim);
```

**Handle empty states:**

```rust
if content.is_empty() {
    let placeholder = if generating {
        "Generating..."
    } else {
        "Press 'r' to generate"
    };

    render_placeholder(frame, area, placeholder);
} else {
    render_content(frame, area, content);
}
```

## Panel Layout Patterns

### Three-Panel Layout (Files | Content | Detail)

Used by Commit, Review modes:

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(20),  // File list
        Constraint::Percentage(50),  // Main content
        Constraint::Percentage(30),  // Details/diff
    ])
    .split(area);
```

### Two-Panel Layout (List | Content)

Used by Explore mode:

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(30),  // Navigation
        Constraint::Percentage(70),  // Content
    ])
    .split(area);
```

### Single-Panel Layout (Full Content)

For focused workflows:

```rust
// Use entire area for content
render_content(frame, area, ...);
```

## Keyboard Navigation Standards

Follow Studio conventions:

| Key       | Action                |
| --------- | --------------------- |
| `j`/`k`   | Navigate up/down      |
| `h`/`l`   | Navigate left/right   |
| `g`/`G`   | Jump to top/bottom    |
| `Tab`     | Cycle panels          |
| `Ctrl+D`  | Page down             |
| `Ctrl+U`  | Page up               |
| `r`       | Regenerate/refresh    |
| `e`       | Edit                  |
| `y`       | Copy to clipboard     |
| `/`       | Open chat             |
| `?`       | Show help             |
| `Esc`     | Close modal/cancel    |
| `Shift+C` | Switch to Commit mode |
| `Shift+R` | Switch to Review mode |

**Mode-specific keys** (like `b` for "select base branch") are fine, but document them in help.

## Event Flow Example

**User presses `r` to regenerate:**

1. Handler receives `KeyCode::Char('r')`
2. Handler updates state: `state.modes.my_mode.generating = true`
3. Handler returns `SideEffect::SpawnAgent { task }`
4. Reducer processes effect, spawns async task
5. Task completes, sends result via channel
6. App loop receives result, dispatches `StudioEvent::AgentComplete`
7. Reducer updates state: `state.modes.my_mode.content = result`
8. Next render cycle draws updated content

## Real-World Examples

Study these complete mode implementations:

### Commit Mode

- **State**: `src/studio/state/modes.rs` → `CommitMode`
- **Handler**: `src/studio/handlers/commit.rs`
- **Renderer**: `src/studio/render/commit.rs`

**Learn from**: Message editing, emoji selection, staged file handling

### Review Mode

- **State**: `src/studio/state/modes.rs` → `ReviewMode`
- **Handler**: `src/studio/handlers/review.rs`
- **Renderer**: `src/studio/render/review.rs`

**Learn from**: Ref selection, markdown rendering, scrolling

### PR Mode

- **State**: `src/studio/state/modes.rs` → `PRMode`
- **Handler**: `src/studio/handlers/pr.rs`
- **Renderer**: `src/studio/render/pr.rs`

**Learn from**: Branch comparison, commit history display

## Testing Your Mode

### Manual Testing Checklist

- [ ] Mode switches correctly from other modes
- [ ] Default panel focus is correct
- [ ] All keybindings work as expected
- [ ] Panel navigation with Tab works
- [ ] Scrolling works (if applicable)
- [ ] Content generates correctly
- [ ] Copy to clipboard works
- [ ] Modal interactions work (ref selector, etc.)
- [ ] Theme colors apply correctly
- [ ] Empty states display properly
- [ ] Error states handled gracefully

### Debug Your Mode

```bash
# Run with verbose logging
RUST_LOG=debug cargo run -- studio

# Check for panics
cargo run -- studio 2> errors.log
```

## Next Steps

- **Add capabilities** that your mode uses → [Adding Capabilities](./capabilities.md)
- **Create tools** to gather mode-specific data → [Adding Tools](./tools.md)
- **Contribute** your mode back → [Contributing](./contributing.md)
