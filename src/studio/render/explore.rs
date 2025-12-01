//! Explore mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

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
            // Right panel: semantic blame if active, otherwise file log
            if state.modes.explore.blame_loading {
                render_blame_loading(frame, area, is_focused);
            } else if let Some(ref blame) = state.modes.explore.semantic_blame {
                render_semantic_blame_panel(frame, area, blame, is_focused);
            } else {
                render_file_log_panel(frame, area, state, is_focused);
            }
        }
    }
}

/// Render the file log panel (git history for selected file)
fn render_file_log_panel(frame: &mut Frame, area: Rect, state: &mut StudioState, is_focused: bool) {
    let show_global = state.modes.explore.show_global_log;

    let title = if show_global {
        " Commit Log (L) ".to_string()
    } else {
        let file_name = state
            .modes
            .explore
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        if file_name.is_empty() {
            " History (L) ".to_string()
        } else {
            format!(" {} (L) ", file_name)
        }
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

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Use either global log or file log based on toggle
    let file_log = if show_global {
        &state.modes.explore.global_log
    } else {
        &state.modes.explore.file_log
    };
    let selected = state.modes.explore.file_log_selected;
    let scroll = state.modes.explore.file_log_scroll;
    let visible_height = inner.height as usize;

    let is_loading = if show_global {
        state.modes.explore.global_log_loading
    } else {
        state.modes.explore.file_log_loading
    };

    if is_loading {
        let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 100) as usize
            % frames.len();
        let spinner = frames[idx];
        let loading = Paragraph::new(format!("{} Loading history...", spinner))
            .style(Style::default().fg(theme::accent_secondary()));
        frame.render_widget(loading, inner);
        return;
    }

    if file_log.is_empty() {
        let hint = if show_global {
            "Press L to load commit log"
        } else if state.modes.explore.current_file.is_none() {
            "Select a file to view history"
        } else {
            "No history for this file"
        };
        let empty = Paragraph::new(hint).style(Style::default().fg(theme::text_dim_color()));
        frame.render_widget(empty, inner);
        return;
    }

    // Render file log entries (3 lines per entry: message, meta, separator)
    let entry_height = 3;
    let visible_entries = visible_height / entry_height;

    let mut lines: Vec<Line> = Vec::new();
    let panel_width = inner.width as usize;

    for (i, entry) in file_log
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_entries)
    {
        let is_selected = i == selected;
        let bg = if is_selected {
            Some(theme::bg_highlight_color())
        } else {
            None
        };

        // Line 1: marker + hash + message (truncated)
        let marker = if is_selected { "› " } else { "  " };
        let marker_style = if is_selected {
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let hash_style = theme::commit_hash().bg(bg.unwrap_or(Color::Reset));
        let msg_style = Style::default()
            .fg(theme::text_primary_color())
            .bg(bg.unwrap_or(Color::Reset));

        // Calculate message width: panel - marker(2) - hash(7) - space(1)
        let msg_width = panel_width.saturating_sub(10);
        let truncated_msg = truncate_str(&entry.message, msg_width);

        lines.push(Line::from(vec![
            Span::styled(marker, marker_style.bg(bg.unwrap_or(Color::Reset))),
            Span::styled(&entry.short_hash, hash_style),
            Span::raw(" "),
            Span::styled(truncated_msg, msg_style),
        ]));

        // Line 2: author · time · stats (indented)
        let meta_style = Style::default()
            .fg(theme::text_dim_color())
            .bg(bg.unwrap_or(Color::Reset));
        let author_style = Style::default()
            .fg(theme::text_muted_color())
            .bg(bg.unwrap_or(Color::Reset));
        let time_style = Style::default()
            .fg(theme::text_dim_color())
            .bg(bg.unwrap_or(Color::Reset));

        let mut meta_spans = vec![
            Span::styled("  ", meta_style), // indent to align with message
            Span::styled(&entry.author, author_style),
            Span::styled(" · ", meta_style),
            Span::styled(&entry.relative_time, time_style),
        ];

        // Add +/- stats if available
        if let (Some(adds), Some(dels)) = (entry.additions, entry.deletions)
            && (adds > 0 || dels > 0)
        {
            meta_spans.push(Span::styled(" ", meta_style));
            if adds > 0 {
                meta_spans.push(Span::styled(
                    format!("+{adds}"),
                    Style::default()
                        .fg(theme::success_color())
                        .bg(bg.unwrap_or(Color::Reset)),
                ));
            }
            if dels > 0 {
                meta_spans.push(Span::styled(
                    format!("-{dels}"),
                    Style::default()
                        .fg(theme::error_color())
                        .bg(bg.unwrap_or(Color::Reset)),
                ));
            }
        }

        lines.push(Line::from(meta_spans));

        // Line 3: subtle separator
        let sep_char = if i + 1 < file_log.len() { "─" } else { " " };
        let sep = sep_char.repeat(panel_width.saturating_sub(2));
        lines.push(Line::from(Span::styled(
            format!(" {sep}"),
            Style::default().fg(theme::bg_highlight_color()),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    // Scrollbar
    if file_log.len() > visible_entries {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state = ScrollbarState::new(file_log.len()).position(scroll);
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

/// Render blame loading state
fn render_blame_loading(frame: &mut Frame, area: Rect, is_focused: bool) {
    let block = Block::default()
        .title(" Analyzing... ")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 100) as usize
        % frames.len();
    let spinner = frames[idx];

    let loading_text = Paragraph::new(format!("{} Iris is analyzing the code history...", spinner))
        .style(Style::default().fg(theme::accent_secondary()));
    frame.render_widget(loading_text, inner);
}

/// Render semantic blame panel with full content
fn render_semantic_blame_panel(
    frame: &mut Frame,
    area: Rect,
    blame: &crate::studio::events::SemanticBlameResult,
    is_focused: bool,
) {
    let block = Block::default()
        .title(" Why This Code? ")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    render_semantic_blame(frame, inner, blame);
}

/// Render the companion status bar (compact, at bottom of explore mode)
pub fn render_companion_status_bar(frame: &mut Frame, area: Rect, state: &StudioState) {
    let display = &state.companion_display;

    // Format: ⎇ branch ↑1↓2 | ●3 staged ○5 unstaged | ◷ 2h15m | [w] why [/] chat
    let mut spans = vec![
        Span::styled("⎇ ", Style::default().fg(theme::text_dim_color())),
        Span::styled(
            &display.branch,
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD),
        ),
    ];

    // Ahead/behind
    if display.ahead > 0 || display.behind > 0 {
        spans.push(Span::raw(" "));
        if display.ahead > 0 {
            spans.push(Span::styled(
                format!("↑{}", display.ahead),
                Style::default().fg(theme::success_color()),
            ));
        }
        if display.behind > 0 {
            if display.ahead > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                format!("↓{}", display.behind),
                Style::default().fg(theme::warning_color()),
            ));
        }
    }

    spans.push(Span::styled(
        " │ ",
        Style::default().fg(theme::text_dim_color()),
    ));

    // Staged/unstaged counts
    if display.staged_count > 0 {
        spans.push(Span::styled(
            format!("●{}", display.staged_count),
            Style::default().fg(theme::success_color()),
        ));
        spans.push(Span::raw(" "));
    }
    if display.unstaged_count > 0 {
        spans.push(Span::styled(
            format!("○{}", display.unstaged_count),
            Style::default().fg(theme::warning_color()),
        ));
    }
    if display.staged_count == 0 && display.unstaged_count == 0 {
        spans.push(Span::styled(
            "clean",
            Style::default().fg(theme::text_dim_color()),
        ));
    }

    spans.push(Span::styled(
        " │ ",
        Style::default().fg(theme::text_dim_color()),
    ));

    // Session duration
    spans.push(Span::styled(
        format!("◷ {}", display.duration),
        Style::default().fg(theme::text_muted_color()),
    ));

    // Welcome message (if any)
    if let Some(ref welcome) = display.welcome_message {
        spans.push(Span::styled(
            " │ ",
            Style::default().fg(theme::text_dim_color()),
        ));
        spans.push(Span::styled(
            welcome.clone(),
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::ITALIC),
        ));
    }

    // Keyboard hints (right-aligned would require calculating width)
    spans.push(Span::styled(
        " │ ",
        Style::default().fg(theme::text_dim_color()),
    ));
    spans.push(Span::styled(
        "[w]",
        Style::default().fg(theme::accent_secondary()),
    ));
    spans.push(Span::styled(
        "hy ",
        Style::default().fg(theme::text_muted_color()),
    ));
    spans.push(Span::styled(
        "[/]",
        Style::default().fg(theme::accent_secondary()),
    ));
    spans.push(Span::styled(
        "chat",
        Style::default().fg(theme::text_muted_color()),
    ));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Truncate a string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
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
