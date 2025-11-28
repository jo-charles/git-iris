//! Ref selector modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::RefSelectorTarget;
use crate::studio::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    refs: &[String],
    selected: usize,
    target: RefSelectorTarget,
) {
    let title = match target {
        RefSelectorTarget::ReviewFrom => " Select Review From Ref ",
        RefSelectorTarget::ReviewTo => " Select Review To Ref ",
        RefSelectorTarget::PrFrom => " Select PR Base (From) ",
        RefSelectorTarget::PrTo => " Select PR Target (To) ",
        RefSelectorTarget::ChangelogFrom => " Select Changelog From ",
        RefSelectorTarget::ChangelogTo => " Select Changelog To ",
        RefSelectorTarget::ReleaseNotesFrom => " Select Release Notes From ",
        RefSelectorTarget::ReleaseNotesTo => " Select Release Notes To ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::NEON_CYAN));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter refs based on input
    let filtered: Vec<_> = refs
        .iter()
        .filter(|r| input.is_empty() || r.to_lowercase().contains(&input.to_lowercase()))
        .collect();

    // Build lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
    ];

    // Show filtered refs
    for (idx, branch) in filtered.iter().take(inner.height as usize - 4).enumerate() {
        let is_selected = idx == selected;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled((*branch).clone(), style),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching refs",
            theme::dimmed(),
        )));
    }

    // Hint at bottom
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" select  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" cancel", theme::dimmed()),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
