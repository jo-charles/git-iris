//! Review mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::components::{render_diff_view, render_file_tree};
use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

/// Render a panel in Review mode
pub fn render_review_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;

    match panel_id {
        PanelId::Left => {
            // Render changed files using FileTree component
            render_file_tree(
                frame,
                area,
                &mut state.modes.review.file_tree,
                "Changed Files",
                is_focused,
            );
        }
        PanelId::Center => {
            // Render diff view for selected file
            let title = state.modes.review.file_tree.selected_path().map_or_else(
                || "Diff".to_string(),
                |p| format!("◈ {}", p.file_name().unwrap_or_default().to_string_lossy()),
            );
            render_diff_view(
                frame,
                area,
                &state.modes.review.diff_view,
                &title,
                is_focused,
            );
        }
        PanelId::Right => {
            // Render review output
            let block = Block::default()
                .title(" Review ")
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    theme::focused_border()
                } else {
                    theme::unfocused_border()
                });
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if state.modes.review.review_content.is_empty() {
                let hint = if state.modes.review.generating {
                    "Generating review..."
                } else {
                    "Press 'r' to generate a code review"
                };
                let text = Paragraph::new(hint).style(Style::default().fg(theme::TEXT_DIM));
                frame.render_widget(text, inner);
            } else {
                // Render review content with scroll
                let lines: Vec<Line> = state
                    .modes
                    .review
                    .review_content
                    .lines()
                    .skip(state.modes.review.review_scroll)
                    .take(inner.height as usize)
                    .map(|line| Line::from(line.to_string()))
                    .collect();
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            }
        }
    }
}
