//! Diff view component for Iris Studio
//!
//! Displays git diffs with syntax highlighting for added/removed lines.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use std::path::PathBuf;

use crate::studio::theme;

// ═══════════════════════════════════════════════════════════════════════════════
// Diff Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Type of diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Context line (unchanged)
    Context,
    /// Added line
    Added,
    /// Removed line
    Removed,
    /// Hunk header (@@ ... @@)
    HunkHeader,
    /// File header (--- or +++)
    FileHeader,
    /// Empty/separator
    Empty,
}

impl DiffLineType {
    /// Get style for this line type
    pub fn style(self) -> Style {
        match self {
            Self::Context => theme::diff_context(),
            Self::Added => theme::diff_added(),
            Self::Removed => theme::diff_removed(),
            Self::HunkHeader => theme::diff_hunk(),
            Self::FileHeader => Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
            Self::Empty => Style::default(),
        }
    }

    /// Get the line prefix character
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Context => " ",
            Self::Added => "+",
            Self::Removed => "-",
            Self::HunkHeader => "@",
            Self::FileHeader => "",
            Self::Empty => " ",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diff Line
// ═══════════════════════════════════════════════════════════════════════════════

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Type of line
    pub line_type: DiffLineType,
    /// Line content (without prefix)
    pub content: String,
    /// Old line number (for context and removed)
    pub old_line_num: Option<usize>,
    /// New line number (for context and added)
    pub new_line_num: Option<usize>,
}

impl DiffLine {
    /// Create a context line
    pub fn context(content: impl Into<String>, old_num: usize, new_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Context,
            content: content.into(),
            old_line_num: Some(old_num),
            new_line_num: Some(new_num),
        }
    }

    /// Create an added line
    pub fn added(content: impl Into<String>, new_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Added,
            content: content.into(),
            old_line_num: None,
            new_line_num: Some(new_num),
        }
    }

    /// Create a removed line
    pub fn removed(content: impl Into<String>, old_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Removed,
            content: content.into(),
            old_line_num: Some(old_num),
            new_line_num: None,
        }
    }

    /// Create a hunk header line
    pub fn hunk_header(content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::HunkHeader,
            content: content.into(),
            old_line_num: None,
            new_line_num: None,
        }
    }

    /// Create a file header line
    pub fn file_header(content: impl Into<String>) -> Self {
        Self {
            line_type: DiffLineType::FileHeader,
            content: content.into(),
            old_line_num: None,
            new_line_num: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diff Hunk
// ═══════════════════════════════════════════════════════════════════════════════

/// A single hunk in a diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Header line (@@...@@)
    pub header: String,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
    /// Starting line in old file
    pub old_start: usize,
    /// Number of lines in old file
    pub old_count: usize,
    /// Starting line in new file
    pub new_start: usize,
    /// Number of lines in new file
    pub new_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// File Diff
// ═══════════════════════════════════════════════════════════════════════════════

/// Diff for a single file
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// File path
    pub path: PathBuf,
    /// Old path (for renames)
    pub old_path: Option<PathBuf>,
    /// Is this a new file?
    pub is_new: bool,
    /// Is this a deleted file?
    pub is_deleted: bool,
    /// Is this a binary file?
    pub is_binary: bool,
    /// Hunks in this diff
    pub hunks: Vec<DiffHunk>,
}

impl FileDiff {
    /// Create a new file diff
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            old_path: None,
            is_new: false,
            is_deleted: false,
            is_binary: false,
            hunks: Vec::new(),
        }
    }

    /// Get total lines changed (added + removed)
    pub fn lines_changed(&self) -> (usize, usize) {
        let mut added = 0;
        let mut removed = 0;
        for hunk in &self.hunks {
            for line in &hunk.lines {
                match line.line_type {
                    DiffLineType::Added => added += 1,
                    DiffLineType::Removed => removed += 1,
                    _ => {}
                }
            }
        }
        (added, removed)
    }

    /// Get all lines for display
    pub fn all_lines(&self) -> Vec<DiffLine> {
        let mut lines = Vec::new();

        // File header
        let status = if self.is_new {
            " (new)"
        } else if self.is_deleted {
            " (deleted)"
        } else {
            ""
        };
        lines.push(DiffLine::file_header(format!(
            "{}{}",
            self.path.display(),
            status
        )));

        if self.is_binary {
            lines.push(DiffLine {
                line_type: DiffLineType::Empty,
                content: "Binary file".to_string(),
                old_line_num: None,
                new_line_num: None,
            });
            return lines;
        }

        for hunk in &self.hunks {
            lines.push(DiffLine::hunk_header(&hunk.header));
            lines.extend(hunk.lines.clone());
        }

        lines
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diff View State
// ═══════════════════════════════════════════════════════════════════════════════

/// State for the diff view widget
#[derive(Debug, Clone)]
pub struct DiffViewState {
    /// All file diffs
    diffs: Vec<FileDiff>,
    /// Currently selected file index
    selected_file: usize,
    /// Scroll offset (line)
    scroll_offset: usize,
    /// Selected line index within current file
    selected_line: usize,
    /// Cached all lines for current file
    cached_lines: Vec<DiffLine>,
}

impl Default for DiffViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffViewState {
    /// Create new diff view state
    pub fn new() -> Self {
        Self {
            diffs: Vec::new(),
            selected_file: 0,
            scroll_offset: 0,
            selected_line: 0,
            cached_lines: Vec::new(),
        }
    }

    /// Set diffs to display
    pub fn set_diffs(&mut self, diffs: Vec<FileDiff>) {
        self.diffs = diffs;
        self.selected_file = 0;
        self.scroll_offset = 0;
        self.selected_line = 0;
        self.update_cache();
    }

    /// Update cached lines
    fn update_cache(&mut self) {
        self.cached_lines = if let Some(diff) = self.diffs.get(self.selected_file) {
            diff.all_lines()
        } else {
            Vec::new()
        };
    }

    /// Get current file diff
    pub fn current_diff(&self) -> Option<&FileDiff> {
        self.diffs.get(self.selected_file)
    }

    /// Get number of files
    pub fn file_count(&self) -> usize {
        self.diffs.len()
    }

    /// Select next file
    pub fn next_file(&mut self) {
        if self.selected_file + 1 < self.diffs.len() {
            self.selected_file += 1;
            self.scroll_offset = 0;
            self.selected_line = 0;
            self.update_cache();
        }
    }

    /// Select previous file
    pub fn prev_file(&mut self) {
        if self.selected_file > 0 {
            self.selected_file -= 1;
            self.scroll_offset = 0;
            self.selected_line = 0;
            self.update_cache();
        }
    }

    /// Select file by index
    pub fn select_file(&mut self, index: usize) {
        if index < self.diffs.len() {
            self.selected_file = index;
            self.scroll_offset = 0;
            self.selected_line = 0;
            self.update_cache();
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.cached_lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    /// Go to next hunk
    pub fn next_hunk(&mut self) {
        let lines = &self.cached_lines;
        for (i, line) in lines.iter().enumerate().skip(self.scroll_offset + 1) {
            if line.line_type == DiffLineType::HunkHeader {
                self.scroll_offset = i;
                return;
            }
        }
    }

    /// Go to previous hunk
    pub fn prev_hunk(&mut self) {
        let lines = &self.cached_lines;
        for i in (0..self.scroll_offset).rev() {
            if lines
                .get(i)
                .is_some_and(|l| l.line_type == DiffLineType::HunkHeader)
            {
                self.scroll_offset = i;
                return;
            }
        }
    }

    /// Get cached lines
    pub fn lines(&self) -> &[DiffLine] {
        &self.cached_lines
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get selected file index
    pub fn selected_file_index(&self) -> usize {
        self.selected_file
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Parsing
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse a unified diff string into `FileDiff` structs
pub fn parse_diff(diff_text: &str) -> Vec<FileDiff> {
    let mut diffs = Vec::new();
    let mut current_diff: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line = 0;
    let mut new_line = 0;

    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            // Save previous diff
            if let Some(mut diff) = current_diff.take() {
                if let Some(hunk) = current_hunk.take() {
                    diff.hunks.push(hunk);
                }
                diffs.push(diff);
            }

            // Parse file path from "diff --git a/path b/path"
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() >= 4 {
                let path = parts[3].strip_prefix("b/").unwrap_or(parts[3]);
                current_diff = Some(FileDiff::new(path));
            }
        } else if line.starts_with("new file") {
            if let Some(ref mut diff) = current_diff {
                diff.is_new = true;
            }
        } else if line.starts_with("deleted file") {
            if let Some(ref mut diff) = current_diff {
                diff.is_deleted = true;
            }
        } else if line.starts_with("Binary files") {
            if let Some(ref mut diff) = current_diff {
                diff.is_binary = true;
            }
        } else if line.starts_with("@@") {
            // Save previous hunk
            if let Some(ref mut diff) = current_diff
                && let Some(hunk) = current_hunk.take()
            {
                diff.hunks.push(hunk);
            }

            // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            let mut hunk = DiffHunk {
                header: line.to_string(),
                lines: Vec::new(),
                old_start: 0,
                old_count: 0,
                new_start: 0,
                new_count: 0,
            };

            // Simple parsing of line numbers
            if let Some(at_section) = line.strip_prefix("@@ ")
                && let Some(end) = at_section.find(" @@")
            {
                let range_part = &at_section[..end];
                let parts: Vec<&str> = range_part.split(' ').collect();

                for part in parts {
                    if let Some(old) = part.strip_prefix('-') {
                        let nums: Vec<&str> = old.split(',').collect();
                        hunk.old_start = nums.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                        hunk.old_count = nums.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                    } else if let Some(new) = part.strip_prefix('+') {
                        let nums: Vec<&str> = new.split(',').collect();
                        hunk.new_start = nums.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                        hunk.new_count = nums.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                    }
                }
            }

            old_line = hunk.old_start;
            new_line = hunk.new_start;
            current_hunk = Some(hunk);
        } else if let Some(ref mut hunk) = current_hunk {
            let diff_line = if let Some(content) = line.strip_prefix('+') {
                let dl = DiffLine::added(content, new_line);
                new_line += 1;
                dl
            } else if let Some(content) = line.strip_prefix('-') {
                let dl = DiffLine::removed(content, old_line);
                old_line += 1;
                dl
            } else if let Some(content) = line.strip_prefix(' ') {
                let dl = DiffLine::context(content, old_line, new_line);
                old_line += 1;
                new_line += 1;
                dl
            } else {
                // Treat as context (handles lines without prefix)
                let dl = DiffLine::context(line, old_line, new_line);
                old_line += 1;
                new_line += 1;
                dl
            };
            hunk.lines.push(diff_line);
        }
    }

    // Save final diff/hunk
    if let Some(mut diff) = current_diff {
        if let Some(hunk) = current_hunk {
            diff.hunks.push(hunk);
        }
        diffs.push(diff);
    }

    diffs
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the diff view widget
pub fn render_diff_view(
    frame: &mut Frame,
    area: Rect,
    state: &DiffViewState,
    title: &str,
    focused: bool,
) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let visible_height = inner.height as usize;
    let lines = state.lines();
    let scroll_offset = state.scroll_offset();
    let line_num_width = 4; // Width for line numbers

    let display_lines: Vec<Line> = lines
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|line| render_diff_line(line, line_num_width, inner.width as usize))
        .collect();

    let paragraph = Paragraph::new(display_lines);
    frame.render_widget(paragraph, inner);

    // Render scrollbar if needed
    if lines.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);

        let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Render a single diff line
fn render_diff_line(line: &DiffLine, line_num_width: usize, _width: usize) -> Line<'static> {
    let style = line.line_type.style();

    match line.line_type {
        DiffLineType::FileHeader => {
            Line::from(vec![Span::styled(format!("━━━ {} ", line.content), style)])
        }
        DiffLineType::HunkHeader => Line::from(vec![
            Span::styled(
                format!("{:>width$} ", "", width = line_num_width * 2 + 3),
                Style::default(),
            ),
            Span::styled(line.content.clone(), style),
        ]),
        DiffLineType::Added | DiffLineType::Removed | DiffLineType::Context => {
            let old_num = line.old_line_num.map_or_else(
                || " ".repeat(line_num_width),
                |n| format!("{:>width$}", n, width = line_num_width),
            );

            let new_num = line.new_line_num.map_or_else(
                || " ".repeat(line_num_width),
                |n| format!("{:>width$}", n, width = line_num_width),
            );

            let prefix = line.line_type.prefix();
            let prefix_style = match line.line_type {
                DiffLineType::Added => Style::default()
                    .fg(theme::SUCCESS_GREEN)
                    .add_modifier(Modifier::BOLD),
                DiffLineType::Removed => Style::default()
                    .fg(theme::ERROR_RED)
                    .add_modifier(Modifier::BOLD),
                _ => theme::dimmed(),
            };

            Line::from(vec![
                Span::styled(old_num, theme::dimmed()),
                Span::styled(" │ ", theme::dimmed()),
                Span::styled(new_num, theme::dimmed()),
                Span::raw(" "),
                Span::styled(prefix, prefix_style),
                Span::styled(line.content.clone(), style),
            ])
        }
        DiffLineType::Empty => Line::from(""),
    }
}

/// Render a compact summary of changes
pub fn render_diff_summary(diff: &FileDiff) -> Line<'static> {
    let (added, removed) = diff.lines_changed();
    let path = diff.path.display().to_string();

    let status = if diff.is_new {
        Span::styled(" new ", Style::default().fg(theme::SUCCESS_GREEN))
    } else if diff.is_deleted {
        Span::styled(" del ", Style::default().fg(theme::ERROR_RED))
    } else {
        Span::raw("")
    };

    Line::from(vec![
        Span::styled(path, theme::file_path()),
        status,
        Span::styled(
            format!("+{added}"),
            Style::default().fg(theme::SUCCESS_GREEN),
        ),
        Span::raw(" "),
        Span::styled(format!("-{removed}"), Style::default().fg(theme::ERROR_RED)),
    ])
}
