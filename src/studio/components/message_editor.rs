//! Message editor component for Iris Studio
//!
//! Text editor for commit messages using tui-textarea.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use tui_textarea::TextArea;
use unicode_width::UnicodeWidthStr;

use crate::studio::theme;
use crate::types::GeneratedMessage;

// ═══════════════════════════════════════════════════════════════════════════════
// Message Editor State
// ═══════════════════════════════════════════════════════════════════════════════

/// State for the message editor component
pub struct MessageEditorState {
    /// Text area for editing
    textarea: TextArea<'static>,
    /// Generated messages from Iris
    generated_messages: Vec<GeneratedMessage>,
    /// Currently selected generated message index
    selected_message: usize,
    /// Is the editor in edit mode (vs view mode)
    edit_mode: bool,
    /// Original message (for reset)
    original_message: String,
}

impl Default for MessageEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageEditorState {
    /// Create a new message editor state
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default().bg(theme::BG_HIGHLIGHT));
        textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

        Self {
            textarea,
            generated_messages: Vec::new(),
            selected_message: 0,
            edit_mode: false,
            original_message: String::new(),
        }
    }

    /// Set generated messages
    pub fn set_messages(&mut self, messages: Vec<GeneratedMessage>) {
        self.generated_messages = messages;
        self.selected_message = 0;
        let first_msg = self.generated_messages.first().cloned();
        if let Some(msg) = first_msg {
            self.load_message(&msg);
        }
    }

    /// Load a message into the editor
    fn load_message(&mut self, msg: &GeneratedMessage) {
        let full_message = format_message(msg);
        self.original_message.clone_from(&full_message);

        // Clear and set new content
        self.textarea = TextArea::from(full_message.lines().map(String::from).collect::<Vec<_>>());
        self.textarea
            .set_cursor_line_style(Style::default().bg(theme::BG_HIGHLIGHT));
        self.textarea
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    /// Get current message count
    pub fn message_count(&self) -> usize {
        self.generated_messages.len()
    }

    /// Get currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_message
    }

    /// Select next message
    pub fn next_message(&mut self) {
        if !self.generated_messages.is_empty() {
            self.selected_message = (self.selected_message + 1) % self.generated_messages.len();
            if let Some(msg) = self.generated_messages.get(self.selected_message) {
                self.load_message(&msg.clone());
            }
            self.edit_mode = false;
        }
    }

    /// Select previous message
    pub fn prev_message(&mut self) {
        if !self.generated_messages.is_empty() {
            self.selected_message = if self.selected_message == 0 {
                self.generated_messages.len() - 1
            } else {
                self.selected_message - 1
            };
            if let Some(msg) = self.generated_messages.get(self.selected_message) {
                self.load_message(&msg.clone());
            }
            self.edit_mode = false;
        }
    }

    /// Enter edit mode
    pub fn enter_edit_mode(&mut self) {
        self.edit_mode = true;
    }

    /// Exit edit mode
    pub fn exit_edit_mode(&mut self) {
        self.edit_mode = false;
    }

    /// Is in edit mode?
    pub fn is_editing(&self) -> bool {
        self.edit_mode
    }

    /// Reset to original message
    pub fn reset(&mut self) {
        self.textarea = TextArea::from(
            self.original_message
                .lines()
                .map(String::from)
                .collect::<Vec<_>>(),
        );
        self.textarea
            .set_cursor_line_style(Style::default().bg(theme::BG_HIGHLIGHT));
        self.textarea
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        self.edit_mode = false;
    }

    /// Get current message text
    pub fn get_message(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Get the current generated message (if any)
    pub fn current_generated(&self) -> Option<&GeneratedMessage> {
        self.generated_messages.get(self.selected_message)
    }

    /// Handle key input (when in edit mode)
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.edit_mode {
            return false;
        }

        // Handle special keys
        if let (KeyCode::Esc, _) = (key.code, key.modifiers) {
            self.exit_edit_mode();
            true
        } else {
            // Forward to textarea
            self.textarea.input(key);
            true
        }
    }

    /// Check if message was modified
    pub fn is_modified(&self) -> bool {
        self.get_message() != self.original_message
    }

    /// Get textarea reference for rendering
    pub fn textarea(&self) -> &TextArea<'static> {
        &self.textarea
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Format a generated message for display
pub fn format_message(msg: &GeneratedMessage) -> String {
    let emoji = msg.emoji.as_deref().unwrap_or("");
    let title = if emoji.is_empty() {
        msg.title.clone()
    } else {
        format!("{} {}", emoji, msg.title)
    };

    if msg.message.is_empty() {
        title
    } else {
        format!("{}\n\n{}", title, msg.message)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the message editor widget
pub fn render_message_editor(
    frame: &mut Frame,
    area: Rect,
    state: &MessageEditorState,
    title: &str,
    focused: bool,
    generating: bool,
) {
    // Build title with message count indicator
    let count_indicator = if state.message_count() > 1 {
        format!(
            " ({}/{})",
            state.selected_index() + 1,
            state.message_count()
        )
    } else {
        String::new()
    };

    let mode_indicator = if state.is_editing() { " [EDITING]" } else { "" };

    let full_title = format!(" {}{}{} ", title, count_indicator, mode_indicator);

    let block = Block::default()
        .title(full_title)
        .borders(Borders::ALL)
        .border_style(if focused {
            if state.is_editing() {
                Style::default().fg(theme::ELECTRIC_PURPLE)
            } else {
                theme::focused_border()
            }
        } else {
            theme::unfocused_border()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    if state.message_count() == 0 {
        // No messages - show placeholder or generating state
        let placeholder = if generating {
            // Show generating spinner with braille pattern
            let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let frame_idx = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                / 100) as usize
                % spinner_frames.len();
            let spinner = spinner_frames[frame_idx];

            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("{} ", spinner),
                        Style::default().fg(theme::ELECTRIC_PURPLE),
                    ),
                    Span::styled(
                        "Analyzing staged changes...",
                        Style::default().fg(theme::TEXT_PRIMARY),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Iris is crafting your commit message",
                    theme::dimmed(),
                )),
            ])
        } else {
            Paragraph::new(vec![
                Line::from(Span::styled("No commit message generated", theme::dimmed())),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'r' to regenerate",
                    Style::default().fg(theme::NEON_CYAN),
                )),
            ])
        };
        frame.render_widget(placeholder, inner);
    } else if state.is_editing() {
        // Render textarea in edit mode
        frame.render_widget(state.textarea(), inner);
    } else {
        // Render as read-only view
        render_message_view(frame, inner, state);
    }
}

/// Truncate a string to fit within the given display width (accounting for unicode)
fn truncate_str(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let s_width = s.width();
    if s_width <= max_width {
        s.to_string()
    } else if max_width <= 1 {
        ".".to_string()
    } else {
        // Build string char by char until we hit width limit
        let mut result = String::new();
        let mut current_width = 0;
        let target_width = max_width - 1; // Leave room for ellipsis

        for c in s.chars() {
            let char_width = c.to_string().width();
            if current_width + char_width > target_width {
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        result.push('…');
        result
    }
}

/// Render the message in view mode (non-editing)
fn render_message_view(frame: &mut Frame, area: Rect, state: &MessageEditorState) {
    let Some(msg) = state.current_generated() else {
        return;
    };

    let width = area.width as usize;
    let mut lines = Vec::new();

    // Emoji and title (truncated to fit)
    let emoji = msg.emoji.as_deref().unwrap_or("");
    let title_width = if emoji.is_empty() {
        width
    } else {
        width.saturating_sub(emoji.chars().count() + 1)
    };
    let title = truncate_str(&msg.title, title_width);

    if emoji.is_empty() {
        lines.push(Line::from(Span::styled(
            title,
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled(emoji, Style::default()),
            Span::raw(" "),
            Span::styled(
                title,
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Empty line
    lines.push(Line::from(""));

    // Body (truncated lines)
    for body_line in msg.message.lines() {
        let truncated = truncate_str(body_line, width);
        lines.push(Line::from(Span::styled(
            truncated,
            Style::default().fg(theme::TEXT_PRIMARY),
        )));
    }

    // Help hints at bottom
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("e", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" edit  ", theme::dimmed()),
        Span::styled("n/p", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" cycle  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" commit", theme::dimmed()),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Render a compact message preview (for lists)
pub fn render_message_preview(msg: &GeneratedMessage, width: usize) -> Line<'static> {
    let emoji = msg.emoji.as_deref().unwrap_or("");
    let title_width = if emoji.is_empty() {
        width
    } else {
        width.saturating_sub(emoji.chars().count() + 1)
    };
    let title = truncate_str(&msg.title, title_width);

    if emoji.is_empty() {
        Line::from(Span::styled(title, theme::dimmed()))
    } else {
        Line::from(vec![
            Span::raw(emoji.to_string()),
            Span::raw(" "),
            Span::styled(title, theme::dimmed()),
        ])
    }
}
