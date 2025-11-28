//! Settings modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::{SettingsField, SettingsState};
use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let title = if state.modified {
        " Settings * "
    } else {
        " Settings "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_PURPLE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        "Configure Iris settings",
        theme::dimmed(),
    )));
    lines.push(Line::from(""));

    // Render each field
    for (idx, field) in SettingsField::all().iter().enumerate() {
        let is_selected = idx == state.selected_field;
        let value = state.get_field_value(*field);

        let prefix = if is_selected { "> " } else { "  " };
        let label_style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };

        let value_style = if is_selected {
            Style::default().fg(theme::TEXT_PRIMARY)
        } else {
            theme::dimmed()
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

        // Add indicator for cyclable fields
        let suffix = match field {
            SettingsField::Provider
            | SettingsField::UseGitmoji
            | SettingsField::InstructionPreset => " [←→]",
            SettingsField::Model | SettingsField::ApiKey => " [edit]",
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled(format!("{:12}", field.display_name()), label_style),
            Span::styled(display_value, value_style),
            Span::styled(suffix, theme::dimmed()),
        ]));
    }

    // Error message if any
    if let Some(error) = &state.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            error,
            Style::default().fg(theme::ERROR_RED),
        )));
    }

    // Footer hints
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    if state.editing {
        lines.push(Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme::NEON_CYAN)),
            Span::styled(" confirm  ", theme::dimmed()),
            Span::styled("Esc", Style::default().fg(theme::NEON_CYAN)),
            Span::styled(" cancel", theme::dimmed()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(" navigate  ", theme::dimmed()),
            Span::styled("Enter/Space", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(" edit  ", theme::dimmed()),
            Span::styled("s", Style::default().fg(theme::SUCCESS_GREEN)),
            Span::styled(" save  ", theme::dimmed()),
            Span::styled("Esc", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(" close", theme::dimmed()),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
