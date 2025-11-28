//! PR mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
#[allow(unused_imports)]
use std::time::SystemTime;

use crate::studio::components::render_diff_view;
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

/// Render a panel in PR mode
pub fn render_pr_panel(state: &mut StudioState, frame: &mut Frame, area: Rect, panel_id: PanelId) {
    let is_focused = panel_id == state.focused_panel;

    match panel_id {
        PanelId::Left => {
            // Render commits list with ref info
            let title = format!(
                " {} → {} ({}) [f/t] ",
                state.modes.pr.base_branch,
                state.modes.pr.to_ref,
                state.modes.pr.commits.len()
            );
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

            if state.modes.pr.commits.is_empty() {
                let text = Paragraph::new("No commits to show")
                    .style(Style::default().fg(theme::TEXT_DIM));
                frame.render_widget(text, inner);
            } else {
                let items: Vec<ListItem> = state
                    .modes
                    .pr
                    .commits
                    .iter()
                    .enumerate()
                    .skip(state.modes.pr.commit_scroll)
                    .take(inner.height as usize)
                    .map(|(idx, commit)| {
                        let is_selected = idx == state.modes.pr.selected_commit;
                        let prefix = if is_selected { "▸ " } else { "  " };
                        let style = if is_selected {
                            Style::default()
                                .fg(theme::TEXT_PRIMARY)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme::TEXT_DIM)
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(prefix, style),
                            Span::styled(&commit.hash, Style::default().fg(theme::CORAL)),
                            Span::raw(" "),
                            Span::styled(&commit.message, style),
                        ]))
                    })
                    .collect();
                let list = List::new(items);
                frame.render_widget(list, inner);
            }
        }
        PanelId::Center => {
            // Calculate visible height for scroll indicator
            let visible_height = area.height.saturating_sub(2) as usize;

            // Prefer streaming content if available, then final content
            let content_to_display = state.modes.pr.streaming_content.as_ref().or(
                if state.modes.pr.pr_content.is_empty() {
                    None
                } else {
                    Some(&state.modes.pr.pr_content)
                },
            );

            let total_lines = content_to_display.map_or(0, |c| c.lines().count());
            let title = scrollable_title(
                "PR Description [y:copy]",
                state.modes.pr.pr_scroll,
                total_lines,
                visible_height,
            );

            // Render PR description (center panel - main content)
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
                    .skip(state.modes.pr.pr_scroll)
                    .take(inner.height as usize)
                    .map(|line| Line::from(line.to_string()))
                    .collect();
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            } else {
                let hint = if state.modes.pr.generating {
                    "Generating PR description..."
                } else {
                    "Press 'r' to generate a PR description"
                };
                let text = Paragraph::new(hint).style(Style::default().fg(theme::TEXT_DIM));
                frame.render_widget(text, inner);
            }
        }
        PanelId::Right => {
            // Render diff view for selected commit or all changes
            let title = if state.modes.pr.commits.is_empty() {
                "Changes".to_string()
            } else {
                format!("Changes ({} → {})", state.modes.pr.base_branch, "HEAD")
            };
            render_diff_view(frame, area, &state.modes.pr.diff_view, &title, is_focused);
        }
    }
}
