//! Emoji selector modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::EmojiInfo;
use crate::studio::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    emojis: &[EmojiInfo],
    selected: usize,
    scroll: usize,
) {
    let block = Block::default()
        .title(" Select Emoji ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_YELLOW));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter emojis based on input
    let filtered: Vec<_> = emojis
        .iter()
        .filter(|e| {
            input.is_empty()
                || e.key.to_lowercase().contains(&input.to_lowercase())
                || e.description.to_lowercase().contains(&input.to_lowercase())
                || e.emoji.contains(input)
        })
        .collect();

    // Calculate visible area for emojis (height minus header and footer)
    let visible_height = inner.height.saturating_sub(5) as usize;

    // Build lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
    ];

    // Show filtered emojis with scroll offset
    for (idx, emoji_info) in filtered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
    {
        let is_selected = idx == selected;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };
        let desc_style = if is_selected {
            Style::default().fg(theme::TEXT_DIM)
        } else {
            theme::dimmed()
        };

        // Special styling for None and Auto options
        let emoji_style = if emoji_info.key == "none" || emoji_info.key == "auto" {
            Style::default()
                .fg(theme::ELECTRIC_PURPLE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::ELECTRIC_YELLOW)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(&emoji_info.emoji, emoji_style),
            Span::raw("  "),
            Span::styled(&emoji_info.key, style),
            Span::styled(" - ", desc_style),
            Span::styled(&emoji_info.description, desc_style),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching emojis",
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
        Span::styled("↑↓", Style::default().fg(theme::ELECTRIC_YELLOW)),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::ELECTRIC_YELLOW)),
        Span::styled(" select  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::ELECTRIC_YELLOW)),
        Span::styled(" cancel", theme::dimmed()),
        Span::styled(scroll_hint, Style::default().fg(theme::TEXT_DIM)),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
