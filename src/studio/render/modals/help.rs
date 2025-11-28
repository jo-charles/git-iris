//! Help modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(theme::keyword());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let section_style = Style::default()
        .fg(theme::accent_secondary())
        .add_modifier(Modifier::BOLD);

    let help_text = vec![
        Line::from(Span::styled("Global", section_style)),
        Line::from("  q          Quit                 /   Chat with Iris"),
        Line::from("  ?          This help            Tab Next panel"),
        Line::from("  Shift+S    Settings             Shift+E  Explore mode"),
        Line::from("  Shift+C    Commit mode          Shift+R  Review mode"),
        Line::from("  Shift+P    PR mode              Shift+L  Changelog mode"),
        Line::from(""),
        Line::from(Span::styled("Navigation (all modes)", section_style)),
        Line::from("  j/k        Down/up              g/G  Top/bottom"),
        Line::from("  h/l        Collapse/expand      Enter Select"),
        Line::from(""),
        Line::from(Span::styled("Commit Mode", section_style)),
        Line::from("  r          Generate message     i   With instructions"),
        Line::from("  e          Edit message         n/p Cycle alternatives"),
        Line::from("  p          Select preset        g   Select emoji"),
        Line::from("  E          Toggle emoji         y   Copy message"),
        Line::from("  Enter      Commit changes"),
        Line::from(""),
        Line::from(Span::styled("Review / PR / Changelog", section_style)),
        Line::from("  f          Select from ref      t   Select to ref"),
        Line::from("  r          Generate             R   Reset"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", theme::dimmed())),
    ];
    let paragraph = Paragraph::new(help_text);
    frame.render_widget(paragraph, inner);
}
