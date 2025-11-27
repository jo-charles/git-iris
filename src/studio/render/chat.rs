//! Chat rendering and markdown formatting for Iris Studio

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
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

/// Format markdown content into styled lines using pulldown-cmark
pub fn format_markdown(content: &str, max_width: usize, base_style: Style) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();

    // Parser state
    let mut style_stack: Vec<Style> = vec![base_style];
    let mut in_code_block = false;
    let mut code_block_lines: Vec<String> = Vec::new();
    let mut code_lang = String::new();
    let mut list_depth: usize = 0;
    let mut ordered_list_num: Option<u64> = None;

    // Enable all markdown options
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(content, options);

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    let color = match level {
                        pulldown_cmark::HeadingLevel::H1 => theme::ELECTRIC_PURPLE,
                        pulldown_cmark::HeadingLevel::H2 => theme::NEON_CYAN,
                        pulldown_cmark::HeadingLevel::H3 => theme::CORAL,
                        _ => theme::TEXT_SECONDARY,
                    };
                    style_stack.push(Style::default().fg(color).add_modifier(Modifier::BOLD));
                    current_spans.push(Span::styled("  ", base_style));
                }
                Tag::Paragraph => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                }
                Tag::CodeBlock(kind) => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    in_code_block = true;
                    code_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                }
                Tag::List(first_num) => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    list_depth += 1;
                    ordered_list_num = first_num;
                }
                Tag::Item => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    let indent = "  ".repeat(list_depth);
                    if let Some(num) = ordered_list_num.as_mut() {
                        current_spans.push(Span::styled(
                            format!("{}{}. ", indent, num),
                            Style::default().fg(theme::ELECTRIC_PURPLE),
                        ));
                        *num += 1;
                    } else {
                        current_spans.push(Span::styled(
                            format!("{}• ", indent),
                            Style::default().fg(theme::ELECTRIC_PURPLE),
                        ));
                    }
                }
                Tag::Emphasis => {
                    let current = style_stack.last().copied().unwrap_or(base_style);
                    style_stack.push(current.add_modifier(Modifier::ITALIC));
                }
                Tag::Strong => {
                    let current = style_stack.last().copied().unwrap_or(base_style);
                    style_stack.push(current.add_modifier(Modifier::BOLD));
                }
                Tag::Strikethrough => {
                    let current = style_stack.last().copied().unwrap_or(base_style);
                    style_stack.push(current.add_modifier(Modifier::CROSSED_OUT));
                }
                Tag::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    current_spans.push(Span::styled("  │ ", Style::default().fg(theme::TEXT_DIM)));
                }
                Tag::Link { .. } | Tag::Image { .. } => {
                    style_stack.push(Style::default().fg(theme::NEON_CYAN));
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    style_stack.pop();
                    flush_line(&mut lines, &mut current_spans, list_depth);
                }
                TagEnd::Paragraph => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    lines.push(Line::from("")); // Add spacing after paragraphs
                }
                TagEnd::CodeBlock => {
                    render_code_block(&mut lines, &code_block_lines, &code_lang, max_width);
                    code_block_lines.clear();
                    code_lang.clear();
                    in_code_block = false;
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        ordered_list_num = None;
                    }
                }
                TagEnd::Item => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                    style_stack.pop();
                }
                TagEnd::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                }
                TagEnd::Link | TagEnd::Image => {
                    style_stack.pop();
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_lines.extend(text.lines().map(String::from));
                } else {
                    let style = style_stack.last().copied().unwrap_or(base_style);
                    // Word wrap the text
                    let effective_width = max_width.saturating_sub(4 + list_depth * 2);
                    for chunk in wrap_text(&text, effective_width) {
                        if !current_spans.is_empty() && !chunk.is_empty() {
                            current_spans.push(Span::styled(chunk, style));
                        } else if !chunk.is_empty() {
                            let indent = if list_depth > 0 { "" } else { "  " };
                            current_spans.push(Span::styled(format!("{}{}", indent, chunk), style));
                        }
                    }
                }
            }
            Event::Code(code) => {
                current_spans.push(Span::styled(code.to_string(), theme::inline_code()));
            }
            Event::SoftBreak => {
                // Treat as space
                let style = style_stack.last().copied().unwrap_or(base_style);
                current_spans.push(Span::styled(" ", style));
            }
            Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans, list_depth);
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_spans, list_depth);
                lines.push(Line::from(Span::styled(
                    "─".repeat(max_width.min(60)),
                    Style::default().fg(theme::TEXT_DIM),
                )));
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "☑ " } else { "☐ " };
                current_spans.push(Span::styled(
                    marker.to_string(),
                    Style::default().fg(theme::ELECTRIC_PURPLE),
                ));
            }
            _ => {}
        }
    }

    // Flush any remaining content
    flush_line(&mut lines, &mut current_spans, list_depth);

    // Remove trailing empty lines
    while lines.last().is_some_and(|l| l.spans.is_empty()) {
        lines.pop();
    }

    lines
}

/// Flush current spans into a line
fn flush_line(lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>, _list_depth: usize) {
    if !spans.is_empty() {
        lines.push(Line::from(std::mem::take(spans)));
    }
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
