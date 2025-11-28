//! Changelog mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

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

/// Render a panel in Changelog mode
pub fn render_changelog_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;

    match panel_id {
        PanelId::Left => {
            // Render commits list with ref info
            let title = format!(
                " {} → {} ({}) [f/t] ",
                state.modes.changelog.from_ref,
                state.modes.changelog.to_ref,
                state.modes.changelog.commits.len()
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

            if state.modes.changelog.commits.is_empty() {
                let text =
                    Paragraph::new("No commits to show\n\nPress 'f' for from ref, 't' for to ref")
                        .style(theme::dimmed());
                frame.render_widget(text, inner);
            } else {
                let items: Vec<ListItem> = state
                    .modes
                    .changelog
                    .commits
                    .iter()
                    .enumerate()
                    .skip(state.modes.changelog.commit_scroll)
                    .take(inner.height as usize)
                    .map(|(idx, commit)| {
                        let is_selected = idx == state.modes.changelog.selected_commit;
                        let prefix = if is_selected { "▸ " } else { "  " };
                        let style = if is_selected {
                            Style::default()
                                .fg(theme::text_primary_color())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme::text_dim_color())
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(prefix, style),
                            Span::styled(&commit.hash, theme::commit_hash()),
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
            let content_to_display = state.modes.changelog.streaming_content.as_ref().or(
                if state.modes.changelog.changelog_content.is_empty() {
                    None
                } else {
                    Some(&state.modes.changelog.changelog_content)
                },
            );

            let total_lines = content_to_display.map_or(0, |c| c.lines().count());
            let title = scrollable_title(
                "Changelog [y:copy]",
                state.modes.changelog.changelog_scroll,
                total_lines,
                visible_height,
            );

            // Render changelog output (generated content)
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
                    .skip(state.modes.changelog.changelog_scroll)
                    .take(inner.height as usize)
                    .map(|line| Line::from(line.to_string()))
                    .collect();
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            } else {
                let hint = if state.modes.changelog.generating {
                    "Generating changelog..."
                } else {
                    "Press 'r' to generate a changelog"
                };
                let text = Paragraph::new(hint).style(theme::dimmed());
                frame.render_widget(text, inner);
            }
        }
        PanelId::Right => {
            // Render diff view for changes between refs
            render_diff_view(
                frame,
                area,
                &state.modes.changelog.diff_view,
                "Changes",
                is_focused,
            );
        }
    }
}
