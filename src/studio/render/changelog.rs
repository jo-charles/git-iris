//! Changelog mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

/// Render a panel in Changelog mode
pub fn render_changelog_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;

    // For now, all panels show "Coming Soon"
    let title = match panel_id {
        PanelId::Left => format!(
            " {} → {} [f/t] ",
            state.modes.changelog.from_version, state.modes.changelog.to_version
        ),
        PanelId::Center => " Changes ".to_string(),
        PanelId::Right => " Changelog ".to_string(),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if is_focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint = match panel_id {
        PanelId::Left => "Commits will appear here\n\nPress 'f' for from ref, 't' for to ref",
        PanelId::Center => "Changes between refs will appear here",
        PanelId::Right => "Generated changelog will appear here\n\nPress 'r' to generate",
    };

    let text = Paragraph::new(hint).style(Style::default().fg(theme::TEXT_DIM));
    frame.render_widget(text, inner);
}
