//! Commit count picker modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::CommitCountTarget;
use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect, input: &str, target: CommitCountTarget) {
    let title = match target {
        CommitCountTarget::Pr => " PR: Last N Commits ",
        CommitCountTarget::Review => " Review: Last N Commits ",
        CommitCountTarget::Changelog => " Changelog: Last N Commits ",
        CommitCountTarget::ReleaseNotes => " Release Notes: Last N Commits ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_secondary()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build content
    let preview = if input.is_empty() {
        "HEAD~_".to_string()
    } else {
        format!("HEAD~{input}")
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Number of commits: ", theme::dimmed()),
            Span::styled(
                if input.is_empty() { "_" } else { input },
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(theme::accent_secondary())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Will set from ref to: ", theme::dimmed()),
            Span::styled(
                preview,
                Style::default()
                    .fg(theme::success_color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(theme::accent_secondary())),
            Span::styled(" confirm  ", theme::dimmed()),
            Span::styled("Esc", Style::default().fg(theme::accent_secondary())),
            Span::styled(" cancel", theme::dimmed()),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
