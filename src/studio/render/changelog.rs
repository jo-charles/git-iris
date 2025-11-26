//! Changelog mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::studio::components::render_diff_view;
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
                        .style(Style::default().fg(theme::TEXT_DIM));
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
            // Render diff view for changes between refs
            render_diff_view(
                frame,
                area,
                &state.modes.changelog.diff_view,
                "Changes",
                is_focused,
            );
        }
        PanelId::Right => {
            // Render changelog output (generated content)
            let block = Block::default()
                .title(" Changelog [y:copy] ")
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    theme::focused_border()
                } else {
                    theme::unfocused_border()
                });
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if state.modes.changelog.changelog_content.is_empty() {
                // Show generating state or hint
                if state.modes.changelog.generating {
                    let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                    let frame_idx = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                        / 100) as usize
                        % spinner_frames.len();
                    let spinner = spinner_frames[frame_idx];

                    let text = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(vec![
                            Span::styled(
                                format!("{} ", spinner),
                                Style::default().fg(theme::ELECTRIC_PURPLE),
                            ),
                            Span::styled(
                                "Analyzing commits...",
                                Style::default().fg(theme::TEXT_PRIMARY),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled(
                            "Iris is crafting your changelog",
                            Style::default().fg(theme::TEXT_DIM),
                        )),
                    ]);
                    frame.render_widget(text, inner);
                } else {
                    let text = Paragraph::new("Press 'r' to generate a changelog")
                        .style(Style::default().fg(theme::TEXT_DIM));
                    frame.render_widget(text, inner);
                }
            } else {
                // Render changelog content with scroll
                let lines: Vec<Line> = state
                    .modes
                    .changelog
                    .changelog_content
                    .lines()
                    .skip(state.modes.changelog.changelog_scroll)
                    .take(inner.height as usize)
                    .map(|line| Line::from(line.to_string()))
                    .collect();
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            }
        }
    }
}
