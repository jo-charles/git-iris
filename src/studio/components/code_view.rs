//! Code view component for Iris Studio
//!
//! Displays file content with line numbers and scrolling support.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use std::fs;
use std::path::{Path, PathBuf};

use crate::studio::theme;

// ═══════════════════════════════════════════════════════════════════════════════
// Code View State
// ═══════════════════════════════════════════════════════════════════════════════

/// State for the code view widget
#[derive(Debug, Clone, Default)]
pub struct CodeViewState {
    /// Path to the currently loaded file
    current_file: Option<PathBuf>,
    /// File content as lines
    lines: Vec<String>,
    /// Scroll offset (line)
    scroll_offset: usize,
    /// Currently selected/highlighted line (1-indexed, 0 = none)
    selected_line: usize,
    /// Selection range for multi-line selection (start, end) 1-indexed
    selection: Option<(usize, usize)>,
}

impl CodeViewState {
    /// Create new code view state
    pub fn new() -> Self {
        Self::default()
    }

    /// Load file content from path
    pub fn load_file(&mut self, path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.lines = content.lines().map(String::from).collect();
        self.current_file = Some(path.to_path_buf());
        self.scroll_offset = 0;
        self.selected_line = 1;
        self.selection = None;
        Ok(())
    }

    /// Get current file path
    pub fn current_file(&self) -> Option<&Path> {
        self.current_file.as_deref()
    }

    /// Get all lines
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get selected line (1-indexed)
    pub fn selected_line(&self) -> usize {
        self.selected_line
    }

    /// Set selected line (1-indexed)
    pub fn set_selected_line(&mut self, line: usize) {
        if line > 0 && line <= self.lines.len() {
            self.selected_line = line;
        }
    }

    /// Get selection range
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    /// Set selection range (start, end) 1-indexed
    pub fn set_selection(&mut self, start: usize, end: usize) {
        if start > 0 && end >= start && end <= self.lines.len() {
            self.selection = Some((start, end));
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Scroll up by amount
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down by amount
    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    /// Scroll to make a specific line visible (1-indexed)
    pub fn scroll_to_line(&mut self, line: usize, visible_height: usize) {
        if line == 0 || self.lines.is_empty() {
            return;
        }
        let line_idx = line.saturating_sub(1);

        // If line is above visible area, scroll up
        if line_idx < self.scroll_offset {
            self.scroll_offset = line_idx;
        }
        // If line is below visible area, scroll down
        else if line_idx >= self.scroll_offset + visible_height {
            self.scroll_offset = line_idx.saturating_sub(visible_height.saturating_sub(1));
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, amount: usize, visible_height: usize) {
        if self.selected_line > 1 {
            self.selected_line = self.selected_line.saturating_sub(amount).max(1);
            self.scroll_to_line(self.selected_line, visible_height);
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, amount: usize, visible_height: usize) {
        if self.selected_line < self.lines.len() {
            self.selected_line = (self.selected_line + amount).min(self.lines.len());
            self.scroll_to_line(self.selected_line, visible_height);
        }
    }

    /// Go to first line
    pub fn goto_first(&mut self) {
        self.selected_line = 1;
        self.scroll_offset = 0;
    }

    /// Go to last line
    pub fn goto_last(&mut self, visible_height: usize) {
        self.selected_line = self.lines.len().max(1);
        self.scroll_to_line(self.selected_line, visible_height);
    }

    /// Check if file is loaded
    pub fn is_loaded(&self) -> bool {
        self.current_file.is_some()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the code view widget
pub fn render_code_view(
    frame: &mut Frame,
    area: Rect,
    state: &CodeViewState,
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

    // Show placeholder if no file loaded
    if !state.is_loaded() {
        let placeholder = Paragraph::new("Select a file from the tree")
            .style(Style::default().fg(theme::TEXT_DIM));
        frame.render_widget(placeholder, inner);
        return;
    }

    let visible_height = inner.height as usize;
    let lines = state.lines();
    let scroll_offset = state.scroll_offset();
    let line_num_width = lines.len().to_string().len().max(3);

    let display_lines: Vec<Line> = lines
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(idx, content)| {
            render_code_line(
                idx + 1, // 1-indexed line number
                content,
                line_num_width,
                inner.width as usize,
                state.selected_line,
                state.selection(),
            )
        })
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

/// Render a single code line with line number
fn render_code_line(
    line_num: usize,
    content: &str,
    line_num_width: usize,
    max_width: usize,
    selected_line: usize,
    selection: Option<(usize, usize)>,
) -> Line<'static> {
    let is_selected = line_num == selected_line;
    let is_in_selection =
        selection.is_some_and(|(start, end)| line_num >= start && line_num <= end);

    // Line number style
    let line_num_style = if is_selected {
        Style::default()
            .fg(theme::NEON_CYAN)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    // Content style
    let content_style = if is_in_selection {
        Style::default()
            .fg(theme::TEXT_PRIMARY)
            .bg(theme::SELECTION_BG)
    } else if is_selected {
        Style::default().fg(theme::TEXT_PRIMARY)
    } else {
        Style::default().fg(theme::TEXT_SECONDARY)
    };

    // Truncate content if too long (use char count, not byte count for UTF-8 safety)
    let available_width = max_width.saturating_sub(line_num_width + 3); // 3 = " | "
    let char_count = content.chars().count();
    let display_content = if char_count > available_width {
        let truncated: String = content
            .chars()
            .take(available_width.saturating_sub(3))
            .collect();
        format!("{}...", truncated)
    } else {
        content.to_string()
    };

    // Selection indicator
    let indicator = if is_selected { ">" } else { " " };
    let indicator_style = if is_selected {
        Style::default()
            .fg(theme::ELECTRIC_PURPLE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    Line::from(vec![
        Span::styled(indicator.to_string(), indicator_style),
        Span::styled(
            format!("{:>width$}", line_num, width = line_num_width),
            line_num_style,
        ),
        Span::styled(" │ ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(display_content, content_style),
    ])
}
