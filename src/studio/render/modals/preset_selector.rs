//! Preset selector modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::PresetInfo;
use crate::studio::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    presets: &[PresetInfo],
    selected: usize,
    scroll: usize,
) {
    let block = Block::default()
        .title(" Select Commit Style Preset ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_primary()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter presets based on input
    let filtered: Vec<_> = presets
        .iter()
        .filter(|p| {
            input.is_empty()
                || p.name.to_lowercase().contains(&input.to_lowercase())
                || p.key.to_lowercase().contains(&input.to_lowercase())
        })
        .collect();

    // Calculate visible area for presets (height minus header and footer)
    let visible_height = inner.height.saturating_sub(5) as usize;

    // Build lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(input, Style::default().fg(theme::text_primary_color())),
            Span::styled("█", Style::default().fg(theme::accent_secondary())),
        ]),
        Line::from(""),
    ];

    // Show filtered presets with scroll offset
    for (idx, preset) in filtered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
    {
        let is_selected = idx == selected;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::text_primary_color())
        };
        let desc_style = if is_selected {
            Style::default().fg(theme::text_dim_color())
        } else {
            theme::dimmed()
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(&preset.emoji, style),
            Span::raw(" "),
            Span::styled(&preset.name, style),
            Span::styled(" - ", desc_style),
            Span::styled(&preset.description, desc_style),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching presets",
            theme::dimmed(),
        )));
    }

    // Hint at bottom with scroll indicator
    lines.push(Line::from(""));
    let scroll_hint = if filtered.len() > visible_height {
        format!(" ({}/{})", selected + 1, filtered.len())
    } else {
        String::new()
    };
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::accent_primary())),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::accent_primary())),
        Span::styled(" select  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::accent_primary())),
        Span::styled(" cancel", theme::dimmed()),
        Span::styled(scroll_hint, Style::default().fg(theme::text_dim_color())),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
