//! Instructions modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect, input: &str) {
    let block = Block::default()
        .title(" Instructions for Iris ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::NEON_CYAN));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "Enter instructions for commit message generation:",
            theme::dimmed(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to generate, Esc to cancel",
            theme::dimmed(),
        )),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
