//! Confirm modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect, message: &str) {
    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_YELLOW));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(message),
        Line::from(""),
        Line::from("Press y/n to confirm"),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
