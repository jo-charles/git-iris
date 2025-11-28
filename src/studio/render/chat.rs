//! Chat rendering and markdown formatting for Iris Studio

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::prelude::Stylize;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::studio::components::syntax::SyntaxHighlighter;
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
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(theme::text_primary_color()),
            ),
            ChatRole::Iris => (
                "◇ Iris",
                Style::default()
                    .fg(theme::accent_primary())
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(theme::text_secondary_color()),
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
                Style::default().fg(theme::text_dim_color()),
            )));
            lines.push(Line::from(""));
        }
    }

    // Show streaming response or typing indicator if Iris is responding
    if chat_state.is_responding {
        if !chat_state.messages.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─".repeat(content_width.min(40)),
                Style::default().fg(theme::text_dim_color()),
            )));
            lines.push(Line::from(""));
        }

        // Show streaming content if available
        if let Some(ref streaming) = chat_state.streaming_response {
            // Iris header for streaming response
            lines.push(Line::from(Span::styled(
                "◇ Iris",
                Style::default()
                    .fg(theme::accent_primary())
                    .add_modifier(Modifier::BOLD),
            )));

            // Render the streaming content with markdown formatting
            let content_style = Style::default().fg(theme::text_secondary_color());
            let formatted_lines = format_markdown(streaming, content_width, content_style);
            lines.extend(formatted_lines);

            // Show tool activity history
            for tool in &chat_state.tool_history {
                lines.push(Line::from(vec![
                    Span::styled("  ⚙ ", Style::default().fg(theme::accent_secondary())),
                    Span::styled(
                        tool.clone(),
                        Style::default()
                            .fg(theme::text_muted_color())
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }

            // Add streaming cursor
            let spinner =
                theme::SPINNER_BRAILLE[last_render_ms as usize / 80 % theme::SPINNER_BRAILLE.len()];
            lines.push(Line::from(Span::styled(
                format!(" {}", spinner),
                Style::default().fg(theme::accent_primary()),
            )));
        } else {
            // Just show thinking indicator when no streaming content yet
            let spinner =
                theme::SPINNER_BRAILLE[last_render_ms as usize / 80 % theme::SPINNER_BRAILLE.len()];

            // Show tool history when thinking
            for tool in &chat_state.tool_history {
                lines.push(Line::from(vec![
                    Span::styled("  ⚙ ", Style::default().fg(theme::accent_secondary())),
                    Span::styled(
                        tool.clone(),
                        Style::default()
                            .fg(theme::text_muted_color())
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }

            // Show current tool activity prominently
            if let Some(ref tool) = chat_state.current_tool {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} ⚙ ", spinner),
                        Style::default().fg(theme::accent_secondary()),
                    ),
                    Span::styled(
                        tool.clone(),
                        Style::default()
                            .fg(theme::text_secondary_color())
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            } else if chat_state.tool_history.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} ", spinner),
                        Style::default().fg(theme::accent_primary()),
                    ),
                    Span::styled(
                        "Iris is thinking",
                        Style::default()
                            .fg(theme::accent_primary())
                            .add_modifier(Modifier::ITALIC),
                    ),
                    Span::styled("...", Style::default().fg(theme::text_dim_color())),
                ]));
            } else {
                // Show spinner after tool history
                lines.push(Line::from(Span::styled(
                    format!("  {}", spinner),
                    Style::default().fg(theme::accent_primary()),
                )));
            }
        }
    }

    // Show error message if present
    if let Some(ref error) = chat_state.error {
        if !chat_state.messages.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─".repeat(content_width.min(40)),
                Style::default().fg(theme::text_dim_color()),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "  ⚠ Error: ",
                Style::default()
                    .fg(theme::error_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(error.clone(), Style::default().fg(theme::error_color())),
        ]));
        lines.push(Line::from(""));
    }

    // Empty state with helpful message
    if chat_state.messages.is_empty() && !chat_state.is_responding && chat_state.error.is_none() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "✨ Ask Iris anything about your changes",
            Style::default()
                .fg(theme::text_secondary_color())
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme::accent_primary())),
            Span::styled("\"Make the title shorter\"", theme::dimmed()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme::accent_primary())),
            Span::styled("\"Add more context to the body\"", theme::dimmed()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme::accent_primary())),
            Span::styled("\"Explain what this change does\"", theme::dimmed()),
        ]));
    }

    // Add bottom padding for scrolling - ensures last message can scroll into view
    if !chat_state.messages.is_empty() || chat_state.is_responding {
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
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
    let mut _in_link = false;
    let mut in_table = false;
    let mut table_row: Vec<String> = Vec::new();

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
                        pulldown_cmark::HeadingLevel::H1 => theme::accent_primary(),
                        pulldown_cmark::HeadingLevel::H2 => theme::accent_secondary(),
                        pulldown_cmark::HeadingLevel::H3 => theme::accent_tertiary(),
                        _ => theme::text_secondary_color(),
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
                            Style::default().fg(theme::accent_primary()),
                        ));
                        *num += 1;
                    } else {
                        current_spans.push(Span::styled(
                            format!("{}• ", indent),
                            Style::default().fg(theme::accent_primary()),
                        ));
                    }
                }
                Tag::Emphasis => {
                    // Add space before if needed
                    if needs_space_before(&current_spans) {
                        current_spans.push(Span::styled(" ", base_style));
                    }
                    let current = style_stack.last().copied().unwrap_or(base_style);
                    style_stack.push(current.add_modifier(Modifier::ITALIC));
                }
                Tag::Strong => {
                    // Add space before if needed
                    if needs_space_before(&current_spans) {
                        current_spans.push(Span::styled(" ", base_style));
                    }
                    let current = style_stack.last().copied().unwrap_or(base_style);
                    style_stack.push(current.add_modifier(Modifier::BOLD));
                }
                Tag::Strikethrough => {
                    // Add space before if needed
                    if needs_space_before(&current_spans) {
                        current_spans.push(Span::styled(" ", base_style));
                    }
                    let current = style_stack.last().copied().unwrap_or(base_style);
                    style_stack.push(current.add_modifier(Modifier::CROSSED_OUT));
                }
                Tag::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    current_spans.push(Span::styled(
                        "  │ ",
                        Style::default().fg(theme::text_dim_color()),
                    ));
                }
                Tag::Link { .. } | Tag::Image { .. } => {
                    // Add space before link if needed
                    if needs_space_before(&current_spans) {
                        current_spans.push(Span::styled(" ", base_style));
                    }
                    _in_link = true;
                    style_stack.push(
                        Style::default()
                            .fg(theme::accent_secondary())
                            .add_modifier(Modifier::UNDERLINED),
                    );
                }
                Tag::Table(_) | Tag::TableHead | Tag::TableRow => {
                    in_table = true;
                }
                Tag::TableCell => {
                    // Cell content handled in text
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
                    lines.push(Line::from("")); // Add spacing after code blocks
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        ordered_list_num = None;
                        lines.push(Line::from("")); // Add spacing after top-level lists
                    }
                }
                TagEnd::Item => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                    style_stack.pop();
                    // Add space after if next text might need it
                    // (will be trimmed if next char is punctuation)
                }
                TagEnd::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_spans, list_depth);
                }
                TagEnd::Link | TagEnd::Image => {
                    style_stack.pop();
                    _in_link = false;
                    // Add space after link
                    current_spans.push(Span::styled(" ", base_style));
                }
                TagEnd::Table => {
                    in_table = false;
                    flush_line(&mut lines, &mut current_spans, list_depth);
                    lines.push(Line::from("")); // Space after table
                }
                TagEnd::TableHead | TagEnd::TableRow => {
                    // Render the row
                    if !table_row.is_empty() {
                        let row_text = table_row.join(" │ ");
                        flush_line(&mut lines, &mut current_spans, list_depth);
                        current_spans.push(Span::styled(
                            format!("  │ {} │", row_text),
                            Style::default().fg(theme::text_secondary_color()),
                        ));
                        flush_line(&mut lines, &mut current_spans, list_depth);
                        table_row.clear();
                    }
                }
                TagEnd::TableCell => {
                    // Cell completed - handled in text
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_lines.extend(text.lines().map(String::from));
                } else if in_table {
                    table_row.push(text.to_string());
                } else {
                    let style = style_stack.last().copied().unwrap_or(base_style);
                    process_text(&text, style, &mut current_spans, list_depth, max_width);
                }
            }
            Event::Code(code) => {
                // Add space before inline code if needed (prevents "text`code`" from merging)
                if let Some(last_span) = current_spans.last() {
                    let last_content = last_span.content.as_ref();
                    if !last_content.is_empty()
                        && !last_content.ends_with(' ')
                        && !last_content.ends_with('\n')
                        && !last_content.ends_with('(')
                        && !last_content.ends_with('[')
                    {
                        let style = style_stack.last().copied().unwrap_or(base_style);
                        current_spans.push(Span::styled(" ", style));
                    }
                }
                // Render without backticks - just styled text
                current_spans.push(Span::styled(code.to_string(), theme::inline_code()));
                // Add trailing space after inline code
                let style = style_stack.last().copied().unwrap_or(base_style);
                current_spans.push(Span::styled(" ", style));
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
                    Style::default().fg(theme::text_dim_color()),
                )));
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "☑ " } else { "☐ " };
                current_spans.push(Span::styled(
                    marker.to_string(),
                    Style::default().fg(theme::accent_primary()),
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

/// Check if we need to add a space before a new styled element
fn needs_space_before(spans: &[Span<'static>]) -> bool {
    if let Some(last_span) = spans.last() {
        let last_content = last_span.content.as_ref();
        !last_content.is_empty()
            && !last_content.ends_with(' ')
            && !last_content.ends_with('\n')
            && !last_content.ends_with('(')
            && !last_content.ends_with('[')
            && !last_content.ends_with('"')
            && !last_content.ends_with('\'')
    } else {
        false
    }
}

/// Process text content with proper spacing and word wrapping
fn process_text(
    text: &str,
    style: Style,
    current_spans: &mut Vec<Span<'static>>,
    list_depth: usize,
    max_width: usize,
) {
    // Add space after inline code if text doesn't start with punctuation/space
    if let Some(last_span) = current_spans.last() {
        let last_content = last_span.content.as_ref();
        if last_content.ends_with('`') {
            let first_char = text.chars().next().unwrap_or(' ');
            if !first_char.is_whitespace()
                && !matches!(
                    first_char,
                    '.' | ',' | ':' | ';' | ')' | ']' | '-' | '!' | '?'
                )
            {
                current_spans.push(Span::styled(" ", style));
            }
        }
    }

    // Word wrap the text
    let effective_width = max_width.saturating_sub(4 + list_depth * 2);
    for chunk in wrap_text(text, effective_width) {
        if !current_spans.is_empty() && !chunk.is_empty() {
            current_spans.push(Span::styled(chunk, style));
        } else if !chunk.is_empty() {
            // Base indent + list depth indent (aligns with bullet text)
            let indent = if list_depth > 0 {
                "  ".repeat(list_depth) + "  "
            } else {
                "  ".to_string()
            };
            current_spans.push(Span::styled(format!("{}{}", indent, chunk), style));
        }
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
        Style::default().fg(theme::text_dim_color()),
    )));

    // Create syntax highlighter for the language
    let highlighter = SyntaxHighlighter::for_extension(lang);

    for code_line in code_lines {
        let truncated = if code_line.len() > max_width.saturating_sub(4) {
            format!("{}…", &code_line[..max_width.saturating_sub(5)])
        } else {
            code_line.clone()
        };

        // Build spans for this line
        let mut line_spans = vec![Span::styled(
            "  │ ",
            Style::default().fg(theme::text_dim_color()),
        )];

        if highlighter.is_available() {
            // Syntax highlighted
            for (style, text) in highlighter.highlight_line(&truncated) {
                line_spans.push(Span::styled(text, style));
            }
        } else {
            // Fallback to plain green
            line_spans.push(Span::styled(
                truncated,
                Style::default().fg(theme::success_color()),
            ));
        }

        lines.push(Line::from(line_spans));
    }

    lines.push(Line::from(Span::styled(
        "  └─",
        Style::default().fg(theme::text_dim_color()),
    )));
}

/// Wrap text to fit within `max_width`
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    // Preserve leading/trailing whitespace
    let has_leading_space = text.starts_with(char::is_whitespace);
    let has_trailing_space = text.ends_with(char::is_whitespace);

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut first_word = true;

    for word in text.split_whitespace() {
        // Add leading space to first word if original had it
        let word_to_add = if first_word && has_leading_space {
            first_word = false;
            format!(" {}", word)
        } else {
            first_word = false;
            word.to_string()
        };
        // Handle words longer than max_width by breaking them
        if word_to_add.len() > max_width {
            // Push current line if not empty
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            // Break the long word into chunks
            let mut remaining = word_to_add.as_str();
            while remaining.len() > max_width {
                let (chunk, rest) = remaining.split_at(max_width);
                lines.push(chunk.to_string());
                remaining = rest;
            }
            if !remaining.is_empty() {
                current_line = remaining.to_string();
            }
        } else if current_line.is_empty() {
            current_line = word_to_add;
        } else if current_line.len() + 1 + word_to_add.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(&word_to_add);
        } else {
            lines.push(current_line);
            current_line = word_to_add;
        }
    }

    // Add trailing space if original had it
    if has_trailing_space && !current_line.is_empty() {
        current_line.push(' ');
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
        Span::styled("❯ ", Style::default().fg(theme::accent_primary())),
        Span::styled(
            input.to_string(),
            Style::default().fg(theme::text_primary_color()),
        ),
        Span::styled(
            cursor_char.to_string(),
            Style::default().fg(theme::accent_secondary()),
        ),
    ])
}

/// Render the help footer
pub fn help_footer() -> Line<'static> {
    Line::from(" [Enter] send · [Esc] close · [↑↓] scroll ").fg(theme::text_dim_color())
}
