//! Chat rendering and markdown formatting for Iris Studio

use ratatui::prelude::Stylize;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::studio::state::{ChatRole, ChatState};
use crate::studio::theme;

/// Render chat messages into formatted lines
pub fn render_messages(
    chat_state: &ChatState,
    content_width: usize,
    last_render_ms: u128,
) -> Vec<Line<'_>> {
    let mut lines: Vec<Line> = Vec::new();

    for (msg_idx, msg) in chat_state.messages.iter().enumerate() {
        // Message header with role indicator
        let (prefix, prefix_style, content_style) = match msg.role {
            ChatRole::User => (
                "◆ You",
                Style::default()
                    .fg(theme::NEON_CYAN)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(theme::TEXT_PRIMARY),
            ),
            ChatRole::Iris => (
                "◇ Iris",
                Style::default()
                    .fg(theme::ELECTRIC_PURPLE)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        };

        lines.push(Line::from(Span::styled(prefix, prefix_style)));

        // Parse and render message content with markdown-like formatting
        let formatted_lines = format_markdown(&msg.content, content_width, content_style);
        lines.extend(formatted_lines);

        // Add separator between messages (except last)
        if msg_idx < chat_state.messages.len() - 1 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─".repeat(content_width.min(40)),
                Style::default().fg(theme::TEXT_DIM),
            )));
            lines.push(Line::from(""));
        }
    }

    // Show typing indicator if Iris is responding
    if chat_state.is_responding {
        if !chat_state.messages.is_empty() {
            lines.push(Line::from(""));
        }
        let spinner =
            theme::SPINNER_BRAILLE[last_render_ms as usize / 80 % theme::SPINNER_BRAILLE.len()];
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default().fg(theme::ELECTRIC_PURPLE),
            ),
            Span::styled(
                "Iris is thinking",
                Style::default()
                    .fg(theme::ELECTRIC_PURPLE)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled("...", Style::default().fg(theme::TEXT_DIM)),
        ]));
    }

    // Empty state with helpful message
    if chat_state.messages.is_empty() && !chat_state.is_responding {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "✨ Ask Iris anything about your changes",
            Style::default()
                .fg(theme::TEXT_SECONDARY)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled("\"Make the title shorter\"", theme::dimmed()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled("\"Add more context to the body\"", theme::dimmed()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled("\"Explain what this change does\"", theme::dimmed()),
        ]));
    }

    lines
}

/// Format markdown content into styled lines
pub fn format_markdown(content: &str, max_width: usize, base_style: Style) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lines: Vec<String> = Vec::new();
    let mut code_lang = String::new();

    for line in content.lines() {
        // Check for code block start/end
        if line.starts_with("```") {
            if in_code_block {
                // End of code block - render accumulated code
                render_code_block(&mut lines, &code_block_lines, &code_lang, max_width);
                code_block_lines.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
                code_lang = line.trim_start_matches('`').to_string();
            }
            continue;
        }

        if in_code_block {
            code_block_lines.push(line.to_string());
            continue;
        }

        // Check for tool call patterns
        if line.contains("tool:") || line.starts_with("🔧") || line.starts_with("[Tool") {
            lines.push(render_tool_call(line));
            continue;
        }

        // Check for headers
        if let Some(header_line) = render_header(line) {
            lines.push(header_line);
            continue;
        }

        // Check for bullet points
        if let Some(bullet_line) = render_bullet(line, base_style) {
            lines.push(bullet_line);
            continue;
        }

        // Check for numbered lists
        if let Some(numbered_line) = render_numbered_list(line, base_style) {
            lines.push(numbered_line);
            continue;
        }

        // Regular line with inline formatting and word wrapping
        if line.is_empty() {
            lines.push(Line::from(""));
        } else {
            // Always wrap to ensure proper display
            let effective_width = max_width.saturating_sub(4); // Account for indent
            for chunk in wrap_text(line, effective_width) {
                let formatted = format_inline(&chunk, base_style);
                lines.push(Line::from(formatted));
            }
        }
    }

    // Handle unclosed code block
    if in_code_block && !code_block_lines.is_empty() {
        render_code_block(&mut lines, &code_block_lines, &code_lang, max_width);
    }

    lines
}

fn render_code_block(
    lines: &mut Vec<Line<'static>>,
    code_lines: &[String],
    lang: &str,
    max_width: usize,
) {
    let lang_label = if lang.is_empty() { "code" } else { lang };
    lines.push(Line::from(Span::styled(
        format!("  ┌─ {} ", lang_label),
        Style::default().fg(theme::TEXT_DIM),
    )));

    for code_line in code_lines {
        let truncated = if code_line.len() > max_width.saturating_sub(4) {
            format!("{}…", &code_line[..max_width.saturating_sub(5)])
        } else {
            code_line.clone()
        };
        lines.push(Line::from(vec![
            Span::styled("  │ ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(truncated, Style::default().fg(theme::SUCCESS_GREEN)),
        ]));
    }

    lines.push(Line::from(Span::styled(
        "  └─",
        Style::default().fg(theme::TEXT_DIM),
    )));
}

fn render_tool_call(line: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ⚙ ", Style::default().fg(theme::CORAL)),
        Span::styled(
            line.trim_start_matches("🔧").trim().to_string(),
            Style::default()
                .fg(theme::CORAL)
                .add_modifier(Modifier::ITALIC),
        ),
    ])
}

fn render_header(line: &str) -> Option<Line<'static>> {
    // Count the number of # characters
    let hash_count = line.chars().take_while(|c| *c == '#').count();
    if hash_count > 0 && line.chars().nth(hash_count) == Some(' ') {
        let header_text = line[hash_count..].trim();
        let (color, prefix) = match hash_count {
            1 => (theme::ELECTRIC_PURPLE, "# "),
            2 => (theme::NEON_CYAN, "## "),
            3 => (theme::CORAL, "### "),
            _ => (theme::TEXT_SECONDARY, ""),
        };
        Some(Line::from(vec![
            Span::styled(
                format!("  {}", prefix),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                header_text.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]))
    } else {
        None
    }
}

fn render_bullet(line: &str, base_style: Style) -> Option<Line<'static>> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        let indent = line.len() - trimmed.len();
        let bullet_content = trimmed.trim_start_matches(['*', '-']).trim();
        Some(Line::from(vec![
            Span::styled(
                format!("  {}• ", " ".repeat(indent)),
                Style::default().fg(theme::ELECTRIC_PURPLE),
            ),
            Span::styled(bullet_content.to_string(), base_style),
        ]))
    } else {
        None
    }
}

#[allow(clippy::unwrap_used)] // Safe: we verified char exists via strip_prefix
fn render_numbered_list(line: &str, base_style: Style) -> Option<Line<'static>> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit())
        && rest.starts_with(". ")
    {
        let indent = line.len() - trimmed.len();
        let num = trimmed.chars().next().unwrap();
        let list_content = rest.trim_start_matches(". ");
        return Some(Line::from(vec![
            Span::styled(
                format!("  {}{}", " ".repeat(indent), num),
                Style::default().fg(theme::ELECTRIC_PURPLE),
            ),
            Span::styled(". ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(list_content.to_string(), base_style),
        ]));
    }
    None
}

/// Format inline markdown (bold, italic, code)
#[allow(clippy::unwrap_used)] // Safe: unwraps inside peek() guards
fn format_inline(text: &str, base_style: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '`' => {
                // Inline code
                if !current.is_empty() {
                    spans.push(Span::styled(format!("  {}", current), base_style));
                    current.clear();
                }
                let mut code = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '`' {
                        chars.next();
                        break;
                    }
                    code.push(chars.next().unwrap());
                }
                spans.push(Span::styled(code, theme::inline_code()));
            }
            '*' if chars.peek() == Some(&'*') => {
                // Bold
                chars.next();
                if !current.is_empty() {
                    spans.push(Span::styled(current.clone(), base_style));
                    current.clear();
                }
                let mut bold_text = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '*' {
                        chars.next();
                        if chars.peek() == Some(&'*') {
                            chars.next();
                            break;
                        }
                        bold_text.push('*');
                    } else {
                        bold_text.push(chars.next().unwrap());
                    }
                }
                spans.push(Span::styled(
                    bold_text,
                    base_style.add_modifier(Modifier::BOLD),
                ));
            }
            '*' | '_' => {
                // Italic
                if !current.is_empty() {
                    spans.push(Span::styled(current.clone(), base_style));
                    current.clear();
                }
                let delimiter = c;
                let mut italic_text = String::new();
                while let Some(&next) = chars.peek() {
                    if next == delimiter {
                        chars.next();
                        break;
                    }
                    italic_text.push(chars.next().unwrap());
                }
                spans.push(Span::styled(
                    italic_text,
                    base_style.add_modifier(Modifier::ITALIC),
                ));
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        if spans.is_empty() {
            spans.push(Span::styled(format!("  {}", current), base_style));
        } else {
            spans.push(Span::styled(current, base_style));
        }
    }

    if spans.is_empty() {
        spans.push(Span::styled("  ".to_string(), base_style));
    }

    spans
}

/// Wrap text to fit within `max_width`
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        // Handle words longer than max_width by breaking them
        if word.len() > max_width {
            // Push current line if not empty
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            // Break the long word into chunks
            let mut remaining = word;
            while remaining.len() > max_width {
                let (chunk, rest) = remaining.split_at(max_width);
                lines.push(chunk.to_string());
                remaining = rest;
            }
            if !remaining.is_empty() {
                current_line = remaining.to_string();
            }
        } else if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Render the input line with cursor
pub fn render_input_line(input: &str, cursor_visible: bool) -> Line<'static> {
    let cursor_char = if cursor_visible { "▌" } else { " " };

    Line::from(vec![
        Span::styled("❯ ", Style::default().fg(theme::ELECTRIC_PURPLE)),
        Span::styled(input.to_string(), Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled(
            cursor_char.to_string(),
            Style::default().fg(theme::NEON_CYAN),
        ),
    ])
}

/// Render the help footer
pub fn help_footer() -> Line<'static> {
    Line::from(" [Enter] send · [Esc] close · [↑↓] scroll ").fg(theme::TEXT_DIM)
}
