//! Theme selector modal rendering

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::ThemeOptionInfo;
use crate::theme;
use crate::theme::adapters::ratatui::ThemeColorExt;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    themes: &[ThemeOptionInfo],
    selected: usize,
    scroll: usize,
) {
    let t = theme::current();

    let block = Block::default()
        .title(" Select Theme ")
        .title_style(
            Style::default()
                .fg(t.ratatui_color("text.primary"))
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.ratatui_color("accent.primary")));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into list and preview areas
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(inner);

    render_theme_list(frame, chunks[0], input, themes, selected, scroll);
    render_theme_preview(frame, chunks[1], themes, selected);
}

fn render_theme_list(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    themes: &[ThemeOptionInfo],
    selected: usize,
    scroll: usize,
) {
    let t = theme::current();

    // Filter themes based on input
    let filtered: Vec<(usize, &ThemeOptionInfo)> = themes
        .iter()
        .enumerate()
        .filter(|(_, theme)| {
            input.is_empty()
                || theme
                    .display_name
                    .to_lowercase()
                    .contains(&input.to_lowercase())
                || theme.author.to_lowercase().contains(&input.to_lowercase())
        })
        .collect();

    // Calculate visible area (height minus header and footer)
    let visible_height = area.height.saturating_sub(5) as usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "  Filter: ",
                Style::default().fg(t.ratatui_color("text.muted")),
            ),
            Span::styled(input, Style::default().fg(t.ratatui_color("text.primary"))),
            Span::styled(
                if input.is_empty() { "│" } else { "█" },
                Style::default().fg(t.ratatui_color("accent.secondary")),
            ),
        ]),
        Line::from(""),
    ];

    // Show filtered themes with scroll offset
    let mut current_variant: Option<&str> = None;
    for (display_idx, (original_idx, theme)) in filtered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
    {
        // Add variant separator
        if current_variant != Some(&theme.variant) {
            if current_variant.is_some() && display_idx > scroll {
                lines.push(Line::from(""));
            }
            let variant_label = if theme.variant == "dark" {
                "  Dark Themes"
            } else {
                "  Light Themes"
            };
            lines.push(Line::from(Span::styled(
                variant_label,
                Style::default()
                    .fg(t.ratatui_color("text.dim"))
                    .add_modifier(Modifier::ITALIC),
            )));
            current_variant = Some(&theme.variant);
        }

        let is_selected = *original_idx == selected;
        let prefix = if is_selected { "  > " } else { "    " };

        let name_style = if is_selected {
            Style::default()
                .fg(t.ratatui_color("accent.secondary"))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.ratatui_color("text.primary"))
        };

        let variant_icon = if theme.variant == "light" { " ☀" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(prefix, name_style),
            Span::styled(&theme.display_name, name_style),
            Span::styled(
                variant_icon,
                Style::default().fg(t.ratatui_color("warning")),
            ),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No matching themes",
            Style::default().fg(t.ratatui_color("text.muted")),
        )));
    }

    // Footer
    lines.push(Line::from(""));
    let scroll_hint = if filtered.len() > visible_height {
        format!(" ({}/{})", selected + 1, themes.len())
    } else {
        String::new()
    };
    lines.push(Line::from(vec![
        Span::styled(
            "  ↑↓",
            Style::default().fg(t.ratatui_color("accent.primary")),
        ),
        Span::styled(" nav  ", Style::default().fg(t.ratatui_color("text.muted"))),
        Span::styled(
            "Enter",
            Style::default().fg(t.ratatui_color("accent.primary")),
        ),
        Span::styled(
            " select  ",
            Style::default().fg(t.ratatui_color("text.muted")),
        ),
        Span::styled("Esc", Style::default().fg(t.ratatui_color("warning"))),
        Span::styled(
            " cancel",
            Style::default().fg(t.ratatui_color("text.muted")),
        ),
        Span::styled(
            scroll_hint,
            Style::default().fg(t.ratatui_color("text.dim")),
        ),
    ]));

    frame.render_widget(Paragraph::new(lines), area);
}

#[allow(clippy::cast_precision_loss)]
fn render_theme_preview(
    frame: &mut Frame,
    area: Rect,
    themes: &[ThemeOptionInfo],
    selected: usize,
) {
    let t = theme::current();

    let Some(theme_info) = themes.get(selected) else {
        return;
    };

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(t.ratatui_color("text.dim")));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        // Theme name
        Line::from(Span::styled(
            format!(" {}", theme_info.display_name),
            Style::default()
                .fg(t.ratatui_color("accent.primary"))
                .add_modifier(Modifier::BOLD),
        )),
        // Author
        Line::from(vec![
            Span::styled(" by ", Style::default().fg(t.ratatui_color("text.muted"))),
            Span::styled(
                &theme_info.author,
                Style::default().fg(t.ratatui_color("text.secondary")),
            ),
        ]),
        Line::from(""),
        // Description
        Line::from(Span::styled(
            format!(" {}", theme_info.description),
            Style::default().fg(t.ratatui_color("text.muted")),
        )),
        Line::from(""),
        // Variant
        Line::from(vec![
            Span::styled(
                " Variant: ",
                Style::default().fg(t.ratatui_color("text.dim")),
            ),
            Span::styled(
                if theme_info.variant == "light" {
                    "Light ☀"
                } else {
                    "Dark"
                },
                Style::default().fg(t.ratatui_color("text.secondary")),
            ),
        ]),
        Line::from(""),
        // Preview section header
        Line::from(Span::styled(
            " Preview",
            Style::default()
                .fg(t.ratatui_color("text.dim"))
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    // Color swatches (using current theme since we apply live preview)
    lines.push(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("██", Style::default().fg(t.ratatui_color("accent.primary"))),
        Span::raw(" "),
        Span::styled(
            "██",
            Style::default().fg(t.ratatui_color("accent.secondary")),
        ),
        Span::raw(" "),
        Span::styled(
            "██",
            Style::default().fg(t.ratatui_color("accent.tertiary")),
        ),
        Span::raw(" "),
        Span::styled("██", Style::default().fg(t.ratatui_color("success"))),
        Span::raw(" "),
        Span::styled("██", Style::default().fg(t.ratatui_color("warning"))),
        Span::raw(" "),
        Span::styled("██", Style::default().fg(t.ratatui_color("error"))),
    ]));

    lines.push(Line::from(""));

    // Gradient
    let gradient_width = 18;
    let mut gradient_spans = vec![Span::styled(" ", Style::default())];
    for i in 0..gradient_width {
        use crate::theme::adapters::ratatui::ToRatatuiColor;
        let t_pos = i as f32 / (gradient_width - 1) as f32;
        let color = t.gradient("primary", t_pos).to_ratatui();
        gradient_spans.push(Span::styled("▀", Style::default().fg(color)));
    }
    lines.push(Line::from(gradient_spans));

    frame.render_widget(Paragraph::new(lines), inner);
}
