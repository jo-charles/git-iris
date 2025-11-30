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
            // Context panel - show semantic blame results or companion session
            let title = if state.modes.explore.blame_loading {
                " Context (analyzing...) "
            } else if state.modes.explore.semantic_blame.is_some() {
                " Why This Code? "
            } else {
                " Session "
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
                // Show companion session info
                render_companion_session(frame, inner, state);
            }
        }
    }
}

/// Render companion session info in the context panel
fn render_companion_session(frame: &mut Frame, area: Rect, state: &StudioState) {
    use ratatui::layout::{Constraint, Layout};

    let display = &state.companion_display;

    // Calculate dynamic layout based on content
    let has_welcome = display.welcome_message.is_some();
    let commits_count = display.recent_commits.len();

    let mut constraints = vec![];
    if has_welcome {
        constraints.push(Constraint::Length(2)); // Welcome
    }
    constraints.push(Constraint::Length(4)); // Branch + HEAD
    constraints.push(Constraint::Length((commits_count as u16).saturating_add(2))); // Recent log
    constraints.push(Constraint::Length(5)); // Session stats
    constraints.push(Constraint::Min(1)); // Hints

    let chunks = Layout::vertical(constraints).split(area);
    let mut chunk_idx = 0;

    // Welcome message (if any)
    if has_welcome {
        if let Some(ref welcome) = display.welcome_message {
            let welcome_lines = vec![Line::from(vec![
                Span::styled("◆ ", Style::default().fg(theme::accent_tertiary())),
                Span::styled(
                    welcome.clone(),
                    Style::default()
                        .fg(theme::accent_primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ])];
            frame.render_widget(Paragraph::new(welcome_lines), chunks[chunk_idx]);
        }
        chunk_idx += 1;
    }

    // Branch + HEAD section
    let mut branch_lines = Vec::new();

    // Branch name with ahead/behind
    let mut branch_spans = vec![
        Span::styled("⎇ ", Style::default().fg(theme::accent_tertiary())),
        Span::styled(
            &display.branch,
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if display.ahead > 0 || display.behind > 0 {
        branch_spans.push(Span::styled(" ", Style::default()));
        if display.ahead > 0 {
            branch_spans.push(Span::styled(
                format!("↑{}", display.ahead),
                Style::default().fg(theme::success_color()),
            ));
        }
        if display.behind > 0 {
            if display.ahead > 0 {
                branch_spans.push(Span::styled(" ", Style::default()));
            }
            branch_spans.push(Span::styled(
                format!("↓{}", display.behind),
                Style::default().fg(theme::warning_color()),
            ));
        }
    }
    branch_lines.push(Line::from(branch_spans));

    // HEAD commit
    if let Some(ref head) = display.head_commit {
        branch_lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(&head.short_hash, theme::commit_hash()),
            Span::styled(" ", Style::default()),
            Span::styled(
                truncate_str(&head.message, 25),
                Style::default().fg(theme::text_primary_color()),
            ),
        ]));
        branch_lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(&head.author, theme::author()),
            Span::styled(" · ", Style::default().fg(theme::text_dim_color())),
            Span::styled(&head.relative_time, theme::timestamp()),
        ]));
    }
    frame.render_widget(Paragraph::new(branch_lines), chunks[chunk_idx]);
    chunk_idx += 1;

    // Recent commits log
    let mut log_lines = vec![Line::from(Span::styled(
        "─── History ───",
        Style::default().fg(theme::text_dim_color()),
    ))];
    for commit in &display.recent_commits {
        log_lines.push(Line::from(vec![
            Span::styled(&commit.short_hash, theme::commit_hash()),
            Span::styled(" ", Style::default()),
            Span::styled(
                truncate_str(&commit.message, 22),
                Style::default().fg(theme::text_secondary_color()),
            ),
        ]));
    }
    if display.recent_commits.is_empty() {
        log_lines.push(Line::from(Span::styled(
            "  (no history)",
            Style::default().fg(theme::text_dim_color()),
        )));
    }
    frame.render_widget(Paragraph::new(log_lines), chunks[chunk_idx]);
    chunk_idx += 1;

    // Session stats
    let mut stats_lines = vec![Line::from(Span::styled(
        "─── Session ───",
        Style::default().fg(theme::text_dim_color()),
    ))];

    // Staged / unstaged counts
    let mut status_spans = vec![Span::styled("  ", Style::default())];
    if display.staged_count > 0 {
        status_spans.push(Span::styled(
            format!("●{} staged", display.staged_count),
            Style::default().fg(theme::success_color()),
        ));
    }
    if display.unstaged_count > 0 {
        if display.staged_count > 0 {
            status_spans.push(Span::styled("  ", Style::default()));
        }
        status_spans.push(Span::styled(
            format!("○{} unstaged", display.unstaged_count),
            Style::default().fg(theme::warning_color()),
        ));
    }
    if display.staged_count == 0 && display.unstaged_count == 0 {
        status_spans.push(Span::styled(
            "clean",
            Style::default().fg(theme::text_dim_color()),
        ));
    }
    stats_lines.push(Line::from(status_spans));

    // Duration + files touched
    stats_lines.push(Line::from(vec![
        Span::styled("  ◷ ", Style::default().fg(theme::text_dim_color())),
        Span::styled(&display.duration, Style::default().fg(theme::text_primary_color())),
        Span::styled("  ◇ ", Style::default().fg(theme::text_dim_color())),
        Span::styled(
            format!("{} files", display.files_touched),
            Style::default().fg(theme::text_primary_color()),
        ),
    ]));

    // Commits made this session
    if display.commits_made > 0 {
        stats_lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{} commits this session", display.commits_made),
                Style::default().fg(theme::success_color()),
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(stats_lines), chunks[chunk_idx]);
    chunk_idx += 1;

    // Hints at bottom
    let hint_lines = vec![
        Line::from(vec![
            Span::styled("[w]", Style::default().fg(theme::accent_secondary())),
            Span::styled(" why  ", Style::default().fg(theme::text_muted_color())),
            Span::styled("[/]", Style::default().fg(theme::accent_secondary())),
            Span::styled(" chat", Style::default().fg(theme::text_muted_color())),
        ]),
    ];
    frame.render_widget(Paragraph::new(hint_lines), chunks[chunk_idx]);
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
