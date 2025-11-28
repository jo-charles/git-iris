//! Settings modal rendering with sectioned layout

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::studio::state::{SettingsField, SettingsSection, SettingsState};
use crate::theme;
use crate::theme::adapters::ratatui::{ThemeColorExt, ToRatatuiColor};

/// Unicode box drawing characters for visual polish
const BOX_HORIZONTAL: &str = "─";

pub fn render(frame: &mut Frame, area: Rect, state: &SettingsState) {
    frame.render_widget(Clear, area);

    let t = theme::current();

    // Title with modification indicator
    let title = if state.modified {
        " Settings * "
    } else {
        " Settings "
    };

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(t.ratatui_color("text.primary"))
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.ratatui_color("border.focused")));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: settings fields, theme preview strip, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Settings fields
            Constraint::Length(2), // Theme preview strip
            Constraint::Length(3), // Footer
        ])
        .split(inner);

    render_settings_fields(frame, chunks[0], state);
    render_theme_strip(frame, chunks[1]);
    render_footer(frame, chunks[2], state);
}

fn render_settings_fields(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let t = theme::current();
    let mut lines = Vec::new();
    let mut current_section: Option<SettingsSection> = None;

    for (idx, field) in SettingsField::all().iter().enumerate() {
        let section = field.section();

        // Section header when section changes
        if current_section != Some(section) {
            if current_section.is_some() {
                lines.push(Line::from("")); // Spacing between sections
            }

            let section_name = section.display_name();
            lines.push(Line::from(Span::styled(
                section_name,
                Style::default()
                    .fg(t.ratatui_color("accent.primary"))
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                BOX_HORIZONTAL.repeat(section_name.len()),
                Style::default().fg(t.ratatui_color("text.dim")),
            )));

            current_section = Some(section);
        }

        let is_selected = idx == state.selected_field;
        let value = state.get_field_value(*field);

        // Styles based on selection
        let (label_style, value_style, row_style) = if is_selected {
            (
                Style::default()
                    .fg(t.ratatui_color("accent.secondary"))
                    .add_modifier(Modifier::BOLD)
                    .bg(t.ratatui_color("bg.highlight")),
                Style::default()
                    .fg(t.ratatui_color("text.primary"))
                    .bg(t.ratatui_color("bg.highlight")),
                Style::default().bg(t.ratatui_color("bg.highlight")),
            )
        } else {
            (
                Style::default().fg(t.ratatui_color("text.secondary")),
                Style::default().fg(t.ratatui_color("text.muted")),
                Style::default(),
            )
        };

        // Show input buffer when editing
        let display_value = if state.editing && is_selected {
            match field {
                SettingsField::ApiKey => format!("{}█", "*".repeat(state.input_buffer.len())),
                _ => format!("{}█", state.input_buffer),
            }
        } else {
            value
        };

        // Light theme indicator
        let suffix = if *field == SettingsField::Theme {
            state
                .current_theme_info()
                .map_or("", |info| if info.variant == "light" { " ☀" } else { "" })
        } else {
            ""
        };

        // Build row with fixed-width label
        let label_width = 14;
        let label = format!("  {:width$}", field.display_name(), width = label_width);
        let value_text = format!("{}{}", display_value, suffix);

        // Pad to fill background highlight
        let padding_len = area
            .width
            .saturating_sub(label.len() as u16 + value_text.len() as u16 + 1);
        let padding = " ".repeat(padding_len as usize);

        lines.push(Line::from(vec![
            Span::styled(label, label_style),
            Span::styled(value_text, value_style),
            Span::styled(padding, row_style),
        ]));
    }

    // Error message
    if let Some(error) = &state.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", error),
            Style::default().fg(t.ratatui_color("error")),
        )));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

#[allow(clippy::cast_precision_loss)]
fn render_theme_strip(frame: &mut Frame, area: Rect) {
    let t = theme::current();

    // Compact preview: palette swatches + gradient on one line
    let mut spans = vec![
        Span::styled("  ", Style::default()),
        Span::styled("██", Style::default().fg(t.ratatui_color("accent.primary"))),
        Span::styled(" ", Style::default()),
        Span::styled(
            "██",
            Style::default().fg(t.ratatui_color("accent.secondary")),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            "██",
            Style::default().fg(t.ratatui_color("accent.tertiary")),
        ),
        Span::styled(" ", Style::default()),
        Span::styled("██", Style::default().fg(t.ratatui_color("success"))),
        Span::styled(" ", Style::default()),
        Span::styled("██", Style::default().fg(t.ratatui_color("warning"))),
        Span::styled(" ", Style::default()),
        Span::styled("██", Style::default().fg(t.ratatui_color("error"))),
        Span::styled("  │  ", Style::default().fg(t.ratatui_color("text.dim"))),
    ];

    // Add gradient
    let gradient_width = 24;
    for i in 0..gradient_width {
        let t_pos = i as f32 / (gradient_width - 1) as f32;
        let color = t.gradient("primary", t_pos).to_ratatui();
        spans.push(Span::styled("▀", Style::default().fg(color)));
    }

    let lines = vec![
        Line::from(Span::styled(
            BOX_HORIZONTAL.repeat(area.width as usize),
            Style::default().fg(t.ratatui_color("text.dim")),
        )),
        Line::from(spans),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let t = theme::current();

    let separator = Line::from(Span::styled(
        BOX_HORIZONTAL.repeat(area.width as usize),
        Style::default().fg(t.ratatui_color("text.dim")),
    ));

    let hints = if state.editing {
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(t.ratatui_color("success"))),
            Span::styled(
                " confirm  ",
                Style::default().fg(t.ratatui_color("text.muted")),
            ),
            Span::styled("Esc", Style::default().fg(t.ratatui_color("warning"))),
            Span::styled(
                " cancel",
                Style::default().fg(t.ratatui_color("text.muted")),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "  ↑↓",
                Style::default().fg(t.ratatui_color("accent.primary")),
            ),
            Span::styled(" nav  ", Style::default().fg(t.ratatui_color("text.muted"))),
            Span::styled("←→", Style::default().fg(t.ratatui_color("accent.primary"))),
            Span::styled(
                " cycle  ",
                Style::default().fg(t.ratatui_color("text.muted")),
            ),
            Span::styled(
                "Enter",
                Style::default().fg(t.ratatui_color("accent.primary")),
            ),
            Span::styled(
                " edit  ",
                Style::default().fg(t.ratatui_color("text.muted")),
            ),
            Span::styled(
                "s",
                Style::default()
                    .fg(t.ratatui_color("success"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " save  ",
                Style::default().fg(t.ratatui_color("text.muted")),
            ),
            Span::styled("Esc", Style::default().fg(t.ratatui_color("warning"))),
            Span::styled(" close", Style::default().fg(t.ratatui_color("text.muted"))),
        ])
    };

    frame.render_widget(Paragraph::new(vec![separator, Line::from(""), hints]), area);
}
