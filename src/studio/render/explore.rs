//! Explore mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::studio::components::{render_code_view, render_file_tree};
use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

/// Render a panel in Explore mode
pub fn render_explore_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;

    match panel_id {
        PanelId::Left => {
            // File tree
            render_file_tree(
                frame,
                area,
                &mut state.modes.explore.file_tree,
                "Files",
                is_focused,
            );
        }
        PanelId::Center => {
            // Code view - display actual file content
            let title = state.modes.explore.code_view.current_file().map_or_else(
                || "Code".to_string(),
                |p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                },
            );

            render_code_view(
                frame,
                area,
                &state.modes.explore.code_view,
                &title,
                is_focused,
            );
        }
        PanelId::Right => {
            // Context panel - show semantic blame results
            let title = if state.modes.explore.blame_loading {
                " Context (analyzing...) "
            } else if state.modes.explore.semantic_blame.is_some() {
                " Why This Code? "
            } else {
                " Context "
            };

            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    theme::focused_border()
                } else {
                    theme::unfocused_border()
                });
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if state.modes.explore.blame_loading {
                // Show loading spinner
                let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let idx = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    / 100) as usize
                    % frames.len();
                let spinner = frames[idx];

                let loading_text =
                    Paragraph::new(format!("{} Iris is analyzing the code history...", spinner))
                        .style(Style::default().fg(theme::accent_secondary()));
                frame.render_widget(loading_text, inner);
            } else if let Some(ref blame) = state.modes.explore.semantic_blame {
                // Show semantic blame result
                render_semantic_blame(frame, inner, blame);
            } else {
                // Show placeholder
                let text = Paragraph::new("Select code and press 'w' to ask why")
                    .style(Style::default().fg(theme::text_dim_color()));
                frame.render_widget(text, inner);
            }
        }
    }
}

/// Render semantic blame result in the context panel
fn render_semantic_blame(
    frame: &mut Frame,
    area: Rect,
    blame: &crate::studio::events::SemanticBlameResult,
) {
    use ratatui::layout::{Constraint, Layout};

    // Split area: header (commit info) and body (explanation)
    let chunks = Layout::vertical([
        Constraint::Length(5), // Header with commit info
        Constraint::Min(1),    // Explanation
    ])
    .split(area);

    // Header: commit info
    let file_name = blame.file.file_name().map_or_else(
        || "Unknown file".to_string(),
        |f| f.to_string_lossy().to_string(),
    );

    let header_lines = vec![
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(theme::text_dim_color())),
            Span::styled(
                file_name,
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" (L{}-{})", blame.start_line, blame.end_line),
                Style::default().fg(theme::text_dim_color()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Commit: ", Style::default().fg(theme::text_dim_color())),
            Span::styled(
                &blame.commit_hash[..8.min(blame.commit_hash.len())],
                theme::commit_hash(),
            ),
            Span::styled(" by ", Style::default().fg(theme::text_dim_color())),
            Span::styled(&blame.author, theme::author()),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().fg(theme::text_dim_color())),
            Span::styled(&blame.commit_date, theme::timestamp()),
        ]),
        Line::from(vec![
            Span::styled("Message: ", Style::default().fg(theme::text_dim_color())),
            Span::styled(
                &blame.commit_message,
                Style::default().fg(theme::text_secondary_color()),
            ),
        ]),
    ];

    let header = Paragraph::new(header_lines);
    frame.render_widget(header, chunks[0]);

    // Body: explanation with markdown rendering
    let lines = render_markdown_lines(&blame.explanation);
    let explanation = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(explanation, chunks[1]);
}

/// Render markdown text into styled Lines
fn render_markdown_lines(text: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for paragraph in text.split("\n\n") {
        if paragraph.trim().is_empty() {
            continue;
        }

        for line in paragraph.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                lines.push(Line::from(""));
                continue;
            }

            // Handle headers
            if let Some(header) = trimmed.strip_prefix("### ") {
                lines.push(Line::from(Span::styled(
                    header.to_string(),
                    Style::default()
                        .fg(theme::accent_secondary())
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }
            if let Some(header) = trimmed.strip_prefix("## ") {
                lines.push(Line::from(Span::styled(
                    header.to_string(),
                    Style::default()
                        .fg(theme::accent_primary())
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }
            if let Some(header) = trimmed.strip_prefix("# ") {
                lines.push(Line::from(Span::styled(
                    header.to_string(),
                    Style::default()
                        .fg(theme::accent_primary())
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }

            // Handle bullet points
            if let Some(bullet_text) = trimmed.strip_prefix("- ") {
                let mut spans = vec![Span::styled(
                    "  • ",
                    Style::default().fg(theme::accent_tertiary()),
                )];
                spans.extend(parse_inline_markdown(bullet_text));
                lines.push(Line::from(spans));
                continue;
            }
            if let Some(bullet_text) = trimmed.strip_prefix("* ") {
                let mut spans = vec![Span::styled(
                    "  • ",
                    Style::default().fg(theme::accent_tertiary()),
                )];
                spans.extend(parse_inline_markdown(bullet_text));
                lines.push(Line::from(spans));
                continue;
            }

            // Handle numbered lists
            if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                && let Some(dot_pos) = trimmed.find(". ")
            {
                let num = &trimmed[..dot_pos];
                if num.chars().all(|c| c.is_ascii_digit()) {
                    let rest = &trimmed[dot_pos + 2..];
                    let mut spans = vec![Span::styled(
                        format!("  {}. ", num),
                        Style::default().fg(theme::accent_tertiary()),
                    )];
                    spans.extend(parse_inline_markdown(rest));
                    lines.push(Line::from(spans));
                    continue;
                }
            }

            // Regular paragraph text with inline formatting
            let spans = parse_inline_markdown(trimmed);
            lines.push(Line::from(spans));
        }

        // Add spacing between paragraphs
        lines.push(Line::from(""));
    }

    // Remove trailing empty line
    if lines.last().is_some_and(|l| l.spans.is_empty()) {
        lines.pop();
    }

    lines
}

/// Parse inline markdown (bold, italic, code) into spans
fn parse_inline_markdown(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                // Bold: **text**
                chars.next(); // consume second *
                if !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        Style::default().fg(theme::text_primary_color()),
                    ));
                }
                // Collect bold text
                let mut bold_text = String::new();
                while let Some(bc) = chars.next() {
                    if bc == '*' && chars.peek() == Some(&'*') {
                        chars.next(); // consume closing **
                        break;
                    }
                    bold_text.push(bc);
                }
                if !bold_text.is_empty() {
                    spans.push(Span::styled(
                        bold_text,
                        Style::default()
                            .fg(theme::warning_color())
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }
            '`' => {
                // Inline code: `code`
                if !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        Style::default().fg(theme::text_primary_color()),
                    ));
                }
                let mut code_text = String::new();
                for cc in chars.by_ref() {
                    if cc == '`' {
                        break;
                    }
                    code_text.push(cc);
                }
                if !code_text.is_empty() {
                    spans.push(Span::styled(
                        code_text,
                        Style::default().fg(theme::accent_secondary()),
                    ));
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        spans.push(Span::styled(
            current,
            Style::default().fg(theme::text_primary_color()),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            text.to_string(),
            Style::default().fg(theme::text_primary_color()),
        ));
    }

    spans
}
