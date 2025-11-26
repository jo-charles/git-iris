//! PR mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::studio::components::render_diff_view;
use crate::studio::state::{PanelId, StudioState};
use crate::studio::theme;

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
            // Render diff view for selected commit or all changes
            let title = if state.modes.pr.commits.is_empty() {
                "Changes".to_string()
            } else {
                format!("Changes ({} → {})", state.modes.pr.base_branch, "HEAD")
            };
            render_diff_view(frame, area, &state.modes.pr.diff_view, &title, is_focused);
        }
        PanelId::Right => {
            // Render PR description
            let block = Block::default()
                .title(" PR Description ")
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    theme::focused_border()
                } else {
                    theme::unfocused_border()
                });
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if state.modes.pr.pr_content.is_empty() {
                // Show generating state or hint
                if state.modes.pr.generating {
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
                            "Iris is crafting your PR description",
                            Style::default().fg(theme::TEXT_DIM),
                        )),
                    ]);
                    frame.render_widget(text, inner);
                } else {
                    let text = Paragraph::new("Press 'r' to generate a PR description")
                        .style(Style::default().fg(theme::TEXT_DIM));
                    frame.render_widget(text, inner);
                }
            } else {
                // Render PR content with scroll
                let lines: Vec<Line> = state
                    .modes
                    .pr
                    .pr_content
                    .lines()
                    .skip(state.modes.pr.pr_scroll)
                    .take(inner.height as usize)
                    .map(|line| Line::from(line.to_string()))
                    .collect();
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, inner);
            }
        }
    }
}
