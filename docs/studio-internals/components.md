# UI Components

**Guide to reusable UI components in Iris Studio.**

## Philosophy

Studio components are **stateful widgets** that:

1. **Own their display state** (scroll position, selection, etc.)
2. **Provide pure render functions** (no side effects)
3. **Are reusable** across multiple modes
4. **Emit semantic updates** (no direct event handling)

## Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Component                              │
│                                                             │
│  ┌──────────────────┐                                       │
│  │ Component State  │ (scroll, selection, cursor, etc.)    │
│  └────────┬─────────┘                                       │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                       │
│  │  Update Methods  │ (scroll_down, select_next, etc.)     │
│  └────────┬─────────┘                                       │
│           │                                                 │
│           ▼                                                 │
│  ┌──────────────────┐                                       │
│  │ Render Function  │ (draw to Ratatui frame)              │
│  │  render_xxx()    │                                       │
│  └──────────────────┘                                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Key insight:** Components manage **display state**, not **business logic**.

## Core Components

### FileTreeState

**Purpose:** Hierarchical file browser with git status.

**Location:** `src/studio/components/file_tree.rs`

#### State Structure

```rust
pub struct FileTreeState {
    /// Root tree nodes
    pub root: Vec<TreeNode>,
    /// Currently selected file path
    pub selected: Option<PathBuf>,
    /// Expanded directories
    pub expanded: HashSet<PathBuf>,
    /// Scroll offset
    pub scroll_offset: usize,
    /// Viewport height
    pub viewport_height: usize,
}
```

#### Key Methods

```rust
impl FileTreeState {
    /// Create from file list with git status
    pub fn from_files(files: Vec<(PathBuf, FileGitStatus)>) -> Self

    /// Select next file
    pub fn select_next(&mut self)

    /// Select previous file
    pub fn select_prev(&mut self)

    /// Toggle expand/collapse directory
    pub fn toggle_expand(&mut self)

    /// Get currently selected file
    pub fn selected_file(&self) -> Option<&PathBuf>

    /// Scroll to ensure selection is visible
    pub fn scroll_to_selection(&mut self)

    /// Flatten tree for rendering
    fn flatten(&self) -> Vec<FlatEntry>
}
```

#### TreeNode Structure

```rust
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub git_status: FileGitStatus,
    pub depth: usize,
    pub children: Vec<TreeNode>,
}
```

Recursively builds directory tree from flat file list.

#### Git Status

```rust
pub enum FileGitStatus {
    Normal,
    Staged,      // ● (green)
    Modified,    // ○ (yellow)
    Untracked,   // ? (cyan)
    Deleted,     // ✕ (red)
    Renamed,     // → (green)
    Conflict,    // ! (red)
}
```

Each status has indicator character and color.

#### Rendering

```rust
pub fn render_file_tree(
    frame: &mut Frame,
    area: Rect,
    state: &FileTreeState,
    focused: bool,
)
```

**Visual:**

```
╭─ Files ────────────────────╮
│                            │
│ ● src/                     │  <- Staged directory
│   ● main.rs                │  <- Staged file (selected)
│   ○ lib.rs                 │  <- Modified file
│   ▸ components/            │  <- Collapsed directory
│ ? docs/                    │  <- Untracked directory
│   ? README.md              │
│                            │
╰────────────────────────────╯
```

**Features:**

- Tree structure with indentation
- Git status indicators
- Expand/collapse arrows (▾/▸)
- Selection highlight
- Scroll indicators
- Focus border styling

### CodeViewState

**Purpose:** Syntax-highlighted source code display.

**Location:** `src/studio/components/code_view.rs`

#### State Structure

```rust
pub struct CodeViewState {
    /// File path being displayed
    pub path: Option<PathBuf>,
    /// File content (lines)
    pub lines: Vec<String>,
    /// Scroll offset (line number)
    pub scroll_offset: usize,
    /// Viewport height
    pub viewport_height: usize,
    /// Highlighted line range (for blame, etc.)
    pub highlighted_range: Option<(usize, usize)>,
    /// Language for syntax highlighting
    pub language: Option<String>,
}
```

#### Key Methods

```rust
impl CodeViewState {
    /// Load file content
    pub fn load_file(&mut self, path: PathBuf, content: String)

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize)

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize)

    /// Jump to line
    pub fn jump_to_line(&mut self, line: usize)

    /// Highlight a line range
    pub fn highlight_range(&mut self, start: usize, end: usize)

    /// Clear highlighting
    pub fn clear_highlight(&mut self)

    /// Get visible line range
    pub fn visible_range(&self) -> (usize, usize)
}
```

#### Rendering

```rust
pub fn render_code_view(
    frame: &mut Frame,
    area: Rect,
    state: &CodeViewState,
    focused: bool,
)
```

**Visual:**

```
╭─ src/main.rs ──────────────────────────────────────────────╮
│                                                            │
│   1  use std::io;                                          │
│   2                                                        │
│   3  fn main() {                                           │
│   4      println!("Hello, world!");                        │
│   5  }                                                     │
│                                                            │
│                                                     [1/142]│
╰────────────────────────────────────────────────────────────╯
```

**Features:**

- Line numbers
- Syntax highlighting (via tree-sitter or syntect)
- Scroll position indicator
- Highlighted line ranges
- Gutter for git blame info
- Focus border styling

### DiffViewState

**Purpose:** Unified/split diff rendering with hunks.

**Location:** `src/studio/components/diff_view.rs`

#### State Structure

```rust
pub struct DiffViewState {
    /// List of file diffs
    pub diffs: Vec<FileDiff>,
    /// Currently selected file index
    pub selected_file: usize,
    /// Scroll offset
    pub scroll_offset: usize,
    /// Viewport height
    pub viewport_height: usize,
    /// Show/hide context lines
    pub show_context: bool,
}

pub struct FileDiff {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,  // For renames
    pub status: DiffStatus,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
    NoNewline,
}
```

#### Key Methods

```rust
impl DiffViewState {
    /// Load diffs from git output
    pub fn load_diffs(&mut self, diff_output: String)

    /// Select next file
    pub fn select_next_file(&mut self)

    /// Select previous file
    pub fn select_prev_file(&mut self)

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize)

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize)

    /// Jump to next hunk
    pub fn next_hunk(&mut self)

    /// Jump to previous hunk
    pub fn prev_hunk(&mut self)

    /// Get total line count
    pub fn total_lines(&self) -> usize

    /// Get statistics
    pub fn stats_summary(&self) -> DiffStats
}
```

#### Rendering

```rust
pub fn render_diff_view(
    frame: &mut Frame,
    area: Rect,
    state: &DiffViewState,
    focused: bool,
)
```

**Visual:**

```
╭─ Diff (3 files, +42/-18) ──────────────────────────────────╮
│                                                            │
│ ● src/main.rs (+12/-5)                                     │
│ @@ -10,7 +10,8 @@ fn main() {                               │
│    use std::io;                                            │
│                                                            │
│ -  fn old_function() {                                     │
│ +  fn new_function() {                                     │
│ +      // Added feature                                    │
│        println!("Hello");                                  │
│    }                                                       │
│                                                            │
│ ○ src/lib.rs (+18/-8)                                      │
│ ? tests/test.rs (+12/-5)                                   │
│                                                            │
╰────────────────────────────────────────────────────────────╯
```

**Features:**

- File-level navigation
- Hunk headers with line numbers
- Color-coded additions/deletions
- Diff statistics
- Context line control
- Git status indicators

### MessageEditorState

**Purpose:** Text editor for commit messages.

**Location:** `src/studio/components/message_editor.rs`

#### State Structure

```rust
pub struct MessageEditorState {
    /// Text area (from tui-textarea crate)
    textarea: TextArea<'static>,
    /// Generated messages from Iris
    generated_messages: Vec<GeneratedMessage>,
    /// Currently selected message index
    selected_message: usize,
    /// Edit mode (view vs edit)
    edit_mode: bool,
    /// Original message (for reset)
    original_message: String,
}
```

#### Key Methods

```rust
impl MessageEditorState {
    /// Set generated messages
    pub fn set_messages(&mut self, messages: Vec<GeneratedMessage>)

    /// Select next message variant
    pub fn next_message(&mut self)

    /// Select previous message variant
    pub fn prev_message(&mut self)

    /// Enter edit mode
    pub fn enter_edit_mode(&mut self)

    /// Exit edit mode
    pub fn exit_edit_mode(&mut self)

    /// Is in edit mode?
    pub fn is_editing(&self) -> bool

    /// Reset to original
    pub fn reset(&mut self)

    /// Get current message text
    pub fn get_message(&self) -> String

    /// Handle key input (when editing)
    pub fn input(&mut self, key: KeyEvent)
}
```

#### Rendering

```rust
pub fn render_message_editor(
    frame: &mut Frame,
    area: Rect,
    state: &MessageEditorState,
    focused: bool,
)
```

**Visual (View Mode):**

```
╭─ Message (1/3) ────────────────────────────────────────────╮
│                                                            │
│ ✨ feat: Add user authentication                           │
│                                                            │
│ Implement JWT-based authentication with:                  │
│ - Login/logout endpoints                                   │
│ - Token refresh mechanism                                  │
│ - Role-based access control                                │
│                                                            │
│                                         [VIEW] [←/→ cycle] │
╰────────────────────────────────────────────────────────────╯
```

**Visual (Edit Mode):**

```
╭─ Message (1/3) [EDIT] ─────────────────────────────────────╮
│                                                            │
│ ✨ feat: Add user authentication█                          │
│                                                            │
│ Implement JWT-based authentication with:                  │
│ - Login/logout endpoints                                   │
│ - Token refresh mechanism                                  │
│ - Role-based access control                                │
│                                                            │
│                                  [ESC cancel] [Enter save] │
╰────────────────────────────────────────────────────────────╯
```

**Features:**

- Multi-line text editing
- Cursor positioning
- Message variant cycling
- View/edit mode toggle
- Reset to original
- Character count

## Component Patterns

### Pattern 1: Stateful Widget

Component owns display state:

```rust
pub struct MyComponentState {
    pub items: Vec<Item>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl MyComponentState {
    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
        self.scroll_to_selection();
    }
}
```

### Pattern 2: Pure Render

Render function has no side effects:

```rust
pub fn render_my_component(
    frame: &mut Frame,
    area: Rect,
    state: &MyComponentState,
    focused: bool,
) {
    // Only draws to frame, no state mutation
    let items: Vec<_> = state.items.iter()
        .map(|item| Line::from(item.name.clone()))
        .collect();

    let list = List::new(items)
        .highlight_style(theme::highlight());

    frame.render_stateful_widget(list, area, &mut state.selected);
}
```

### Pattern 3: Builder Pattern

```rust
impl FileTreeState {
    pub fn new() -> Self { ... }

    pub fn with_files(files: Vec<(PathBuf, FileGitStatus)>) -> Self {
        let mut state = Self::new();
        state.load_files(files);
        state
    }

    pub fn with_selection(mut self, path: PathBuf) -> Self {
        self.selected = Some(path);
        self
    }
}
```

### Pattern 4: Event Emission

Components don't handle events directly, they return what changed:

```rust
// BAD: Component handles events
impl MyComponent {
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Enter {
            self.spawn_agent();  // Side effect!
        }
    }
}

// GOOD: Component updates state, caller decides action
impl MyComponent {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Enter => Some(Action::Confirm),
            KeyCode::Up => { self.select_prev(); None }
            _ => None
        }
    }
}
```

## Component Composition

Components compose into mode layouts:

```
┌────────────────────────────────────────────────────────────┐
│                     Commit Mode                            │
├──────────────────┬─────────────────────┬───────────────────┤
│                  │                     │                   │
│  FileTreeState   │  MessageEditorState │   DiffViewState   │
│                  │                     │                   │
│  ╭─ Files ────╮  │  ╭─ Message ─────╮  │  ╭─ Diff ──────╮ │
│  │ ● main.rs  │  │  │ ✨ feat: ...   │  │  │ +  added    │ │
│  │ ○ lib.rs   │  │  │               │  │  │ -  removed  │ │
│  │ ? test.rs  │  │  │ Description   │  │  │    context  │ │
│  ╰────────────╯  │  ╰───────────────╯  │  ╰─────────────╯ │
│                  │                     │                   │
└──────────────────┴─────────────────────┴───────────────────┘
```

Each component is independent, mode state owns all component states.

## Component State Management

### Where State Lives

**Component state** (scroll, selection) lives in component struct.

**Business state** (generated messages, file content) lives in mode state.

**Example:**

```rust
pub struct CommitMode {
    // Business state
    pub messages: Vec<GeneratedMessage>,
    pub generating: bool,

    // Component states
    pub message_editor: MessageEditorState,
    pub diff_view: DiffViewState,
    pub file_tree: FileTreeState,
}
```

### State Updates

**Component state** updated directly:

```rust
state.modes.commit.file_tree.select_next();
state.modes.commit.diff_view.scroll_down(5);
```

**Business state** updated via reducer:

```rust
StudioEvent::AgentComplete { result } => {
    state.modes.commit.messages = result;
    state.modes.commit.message_editor.set_messages(result);
}
```

## Advanced Components

### Syntax Highlighting

**Location:** `src/studio/components/syntax.rs`

```rust
pub struct SyntaxHighlighter {
    highlighter: syntect::highlighting::Highlighter,
    theme: syntect::highlighting::Theme,
}

impl SyntaxHighlighter {
    pub fn highlight_line(
        &self,
        line: &str,
        language: &str,
    ) -> Vec<(Style, String)>
}
```

Used by `CodeViewState` to colorize code.

### Scrollbar

**Ratatui built-in:**

```rust
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

let scrollbar = Scrollbar::default()
    .orientation(ScrollbarOrientation::VerticalRight)
    .begin_symbol(Some("↑"))
    .end_symbol(Some("↓"));

let mut scrollbar_state = ScrollbarState::new(total_lines)
    .position(scroll_offset);

frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
```

### Modal Overlays

**Pattern:** Center modal over background.

```rust
pub fn render_modal_overlay(
    frame: &mut Frame,
    background_render: impl Fn(&mut Frame),
) {
    // 1. Render background (dimmed)
    background_render(frame);

    // 2. Calculate centered area
    let area = centered_rect(60, 20, frame.size());

    // 3. Clear modal area
    frame.render_widget(Clear, area);

    // 4. Render modal content
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Modal Title");
    frame.render_widget(block, area);
}
```

## Component Testing

Components are easy to test:

```rust
#[test]
fn test_file_tree_selection() {
    let files = vec![
        (PathBuf::from("a.rs"), FileGitStatus::Normal),
        (PathBuf::from("b.rs"), FileGitStatus::Modified),
        (PathBuf::from("c.rs"), FileGitStatus::Staged),
    ];

    let mut tree = FileTreeState::from_files(files);

    // Initial selection
    assert_eq!(tree.selected_file(), Some(&PathBuf::from("a.rs")));

    // Navigate
    tree.select_next();
    assert_eq!(tree.selected_file(), Some(&PathBuf::from("b.rs")));

    tree.select_next();
    assert_eq!(tree.selected_file(), Some(&PathBuf::from("c.rs")));

    // Wrap around
    tree.select_next();
    assert_eq!(tree.selected_file(), Some(&PathBuf::from("a.rs")));
}

#[test]
fn test_diff_view_stats() {
    let mut diff_view = DiffViewState::new();
    diff_view.load_diffs(SAMPLE_DIFF);

    let stats = diff_view.stats_summary();
    assert_eq!(stats.files_changed, 3);
    assert_eq!(stats.insertions, 42);
    assert_eq!(stats.deletions, 18);
}
```

## Performance Considerations

**Lazy rendering:** Only render visible lines.

```rust
let visible_start = self.scroll_offset;
let visible_end = visible_start + self.viewport_height;

let visible_lines: Vec<_> = self.lines[visible_start..visible_end]
    .iter()
    .map(|line| Line::from(line.clone()))
    .collect();
```

**Caching:** Pre-compute expensive operations.

```rust
pub struct FileTreeState {
    root: Vec<TreeNode>,
    flattened_cache: Option<Vec<FlatEntry>>,  // Cache
}

impl FileTreeState {
    fn flatten(&mut self) -> &[FlatEntry] {
        if self.flattened_cache.is_none() {
            self.flattened_cache = Some(self.compute_flattened());
        }
        self.flattened_cache.as_ref().unwrap()
    }

    pub fn invalidate_cache(&mut self) {
        self.flattened_cache = None;
    }
}
```

**Batching:** Update once, render once.

```rust
// BAD: Multiple renders
state.diff_view.scroll_down(1);
frame.render(...);  // Render
state.diff_view.scroll_down(1);
frame.render(...);  // Render again

// GOOD: Batch updates
state.diff_view.scroll_down(2);
frame.render(...);  // Render once
```

## Styling with SilkCircuit Theme

All components use theme functions:

```rust
use crate::studio::theme;

// Text colors
let text = Span::styled("Hello", theme::text());
let dimmed = Span::styled("(optional)", theme::dimmed());
let keyword = Span::styled("fn", theme::keyword());

// Git status
let staged = Span::styled("●", theme::git_staged());
let modified = Span::styled("○", theme::git_modified());

// Highlights
let selected = Span::styled("Item", theme::highlight());
let focused = Block::default()
    .border_style(theme::focus_border());

// Notifications
let success = Span::styled("✓", theme::success());
let error = Span::styled("✗", theme::error());
let warning = Span::styled("⚠", theme::warning());
```

**Consistency:** All components use same color palette.

## Creating a New Component

### 1. Define State

```rust
pub struct MyComponentState {
    pub items: Vec<String>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub viewport_height: usize,
}
```

### 2. Implement Methods

```rust
impl MyComponentState {
    pub fn new() -> Self { ... }

    pub fn select_next(&mut self) { ... }
    pub fn select_prev(&mut self) { ... }

    pub fn scroll_down(&mut self) { ... }
    pub fn scroll_up(&mut self) { ... }
}
```

### 3. Create Render Function

```rust
pub fn render_my_component(
    frame: &mut Frame,
    area: Rect,
    state: &MyComponentState,
    focused: bool,
) {
    let items: Vec<Line> = state.items.iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.selected {
                theme::highlight()
            } else {
                theme::text()
            };
            Line::from(Span::styled(item, style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if focused {
                theme::focus_border()
            } else {
                theme::dimmed()
            }));

    frame.render_widget(list, area);
}
```

### 4. Add to Mode State

```rust
pub struct MyMode {
    pub my_component: MyComponentState,
}
```

### 5. Use in Render

```rust
pub fn render_my_mode_panel(
    frame: &mut Frame,
    areas: &LayoutAreas,
    state: &StudioState,
) {
    render_my_component(
        frame,
        areas.center,
        &state.modes.my_mode.my_component,
        state.focused_panel == PanelId::Center,
    );
}
```

## Common Component Utilities

### Centering

```rust
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### Truncation

```rust
pub fn truncate_width(text: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthStr;

    if text.width() <= max_width {
        return text.to_string();
    }

    let mut width = 0;
    let mut result = String::new();

    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width + 3 > max_width {  // Reserve space for "..."
            result.push_str("...");
            break;
        }
        result.push(ch);
        width += ch_width;
    }

    result
}
```

### Scroll Indicators

```rust
pub fn render_scroll_indicators(
    frame: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    total_lines: usize,
    viewport_height: usize,
) {
    let can_scroll_up = scroll_offset > 0;
    let can_scroll_down = scroll_offset + viewport_height < total_lines;

    if can_scroll_up {
        let indicator = Span::styled("▲", theme::dimmed());
        frame.render_widget(
            Paragraph::new(Line::from(indicator)),
            Rect { x: area.x + area.width - 1, y: area.y, width: 1, height: 1 },
        );
    }

    if can_scroll_down {
        let indicator = Span::styled("▼", theme::dimmed());
        frame.render_widget(
            Paragraph::new(Line::from(indicator)),
            Rect { x: area.x + area.width - 1, y: area.y + area.height - 1, width: 1, height: 1 },
        );
    }
}
```

## Summary

**Components are stateful widgets:**

- Own display state (scroll, selection)
- Provide pure render functions
- Reusable across modes
- No business logic, no side effects

**Key patterns:**

- State + Methods + Render
- Builder pattern for construction
- Event emission via return values
- Theme-consistent styling

**When creating components:**

- Keep state minimal (only display concerns)
- Make render pure (no mutations)
- Test state updates independently
- Use theme functions for consistency
