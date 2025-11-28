//! Search modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect, query: &str, results: &[String], selected: usize) {
    let block = Block::default()
        .title(" Search Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_secondary()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter results by query
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| query.is_empty() || r.to_lowercase().contains(&query.to_lowercase()))
        .collect();

    let visible_height = inner.height.saturating_sub(4) as usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(query, Style::default().fg(theme::text_primary_color())),
            Span::styled("█", Style::default().fg(theme::accent_secondary())),
        ]),
        Line::from(""),
    ];

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            if results.is_empty() {
                "No files in current mode"
            } else {
                "No matching files"
            },
            theme::dimmed(),
        )));
    } else {
        // Calculate scroll offset to keep selection visible
        let scroll = if selected >= visible_height {
            selected - visible_height + 1
        } else {
            0
        };

        for (i, file) in filtered
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
        {
            let is_selected = i == selected;
            let prefix = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::text_primary_color())
            };
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, file),
                style,
            )));
        }
    }

    // Add footer
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::accent_secondary())),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::accent_secondary())),
        Span::styled(" jump  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::accent_secondary())),
        Span::styled(" cancel", theme::dimmed()),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
