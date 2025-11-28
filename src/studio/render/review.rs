//! Review mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::studio::components::{render_diff_view, render_file_tree};
use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

/// Create a panel title with scroll position indicator
fn scrollable_title(base_title: &str, scroll: usize, total_lines: usize, visible: usize) -> String {
    if total_lines <= visible {
        format!(" {} ", base_title)
    } else {
        let max_scroll = total_lines.saturating_sub(visible);
        let percent = if max_scroll == 0 {
            100
        } else {
            ((scroll.min(max_scroll)) * 100) / max_scroll
        };
        format!(
            " {} ({}/{}) {}% ",
            base_title,
            scroll + 1,
            total_lines,
            percent
        )
    }
}

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
            // Calculate visible height for scroll indicator
            let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders

            // Prefer streaming content if available, then final content
            let content_to_display = state.modes.review.streaming_content.as_ref().or(
                if state.modes.review.review_content.is_empty() {
                    None
                } else {
                    Some(&state.modes.review.review_content)
                },
            );

            let total_lines = content_to_display.map_or(0, |c| c.lines().count());
            let title = scrollable_title(
                "Review [y:copy]",
                state.modes.review.review_scroll,
                total_lines,
                visible_height,
            );

            // Render review output (center panel - main content)
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

            if let Some(content) = content_to_display {
                // Render content with scroll
                let lines: Vec<Line> = content
                    .lines()
                    .skip(state.modes.review.review_scroll)
                    .take(inner.height as usize)
                    .map(|line| Line::from(line.to_string()))
                    .collect();
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            } else {
                let hint = if state.modes.review.generating {
                    "Generating review..."
                } else {
                    "Press 'r' to generate a code review"
                };
                let text = Paragraph::new(hint).style(theme::dimmed());
                frame.render_widget(text, inner);
            }
        }
        PanelId::Right => {
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
    }
}
