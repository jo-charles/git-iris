//! Explore mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::components::{render_code_view, render_file_tree};
use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

/// Render a panel in Explore mode
pub fn render_explore_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;

    match panel_id {
        PanelId::Left => {
            // File tree
            render_file_tree(
                frame,
                area,
                &mut state.modes.explore.file_tree,
                "Files",
                is_focused,
            );
        }
        PanelId::Center => {
            // Code view - display actual file content
            let title = state.modes.explore.code_view.current_file().map_or_else(
                || "Code".to_string(),
                |p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                },
            );

            render_code_view(
                frame,
                area,
                &state.modes.explore.code_view,
                &title,
                is_focused,
            );
        }
        PanelId::Right => {
            // Context panel
            let block = Block::default()
                .title(" Context ")
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    theme::focused_border()
                } else {
                    theme::unfocused_border()
                });
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let text = Paragraph::new("Semantic context\n\nSelect code and press 'w' to ask why")
                .style(Style::default().fg(theme::TEXT_DIM));
            frame.render_widget(text, inner);
        }
    }
}
