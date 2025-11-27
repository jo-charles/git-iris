//! Explore mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

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
            // Context panel - show semantic blame results
            let title = if state.modes.explore.blame_loading {
                " Context (analyzing...) "
            } else if state.modes.explore.semantic_blame.is_some() {
                " Why This Code? "
            } else {
                " Context "
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

            if state.modes.explore.blame_loading {
                // Show loading spinner
                let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let idx = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    / 100) as usize
                    % frames.len();
                let spinner = frames[idx];

                let loading_text =
                    Paragraph::new(format!("{} Iris is analyzing the code history...", spinner))
                        .style(Style::default().fg(theme::NEON_CYAN));
                frame.render_widget(loading_text, inner);
            } else if let Some(ref blame) = state.modes.explore.semantic_blame {
                // Show semantic blame result
                render_semantic_blame(frame, inner, blame);
            } else {
                // Show placeholder
                let text = Paragraph::new("Select code and press 'w' to ask why")
                    .style(Style::default().fg(theme::TEXT_DIM));
                frame.render_widget(text, inner);
            }
        }
    }
}

/// Render semantic blame result in the context panel
fn render_semantic_blame(
    frame: &mut Frame,
    area: Rect,
    blame: &crate::studio::events::SemanticBlameResult,
) {
    use ratatui::layout::{Constraint, Layout};

    // Split area: header (commit info) and body (explanation)
    let chunks = Layout::vertical([
        Constraint::Length(5), // Header with commit info
        Constraint::Min(1),    // Explanation
    ])
    .split(area);

    // Header: commit info
    let file_name = blame.file.file_name().map_or_else(
        || "Unknown file".to_string(),
        |f| f.to_string_lossy().to_string(),
    );

    let header_lines = vec![
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(
                file_name,
                Style::default()
                    .fg(theme::NEON_CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" (L{}-{})", blame.start_line, blame.end_line),
                Style::default().fg(theme::TEXT_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("Commit: ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(
                &blame.commit_hash[..8.min(blame.commit_hash.len())],
                Style::default().fg(theme::CORAL),
            ),
            Span::styled(" by ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(&blame.author, Style::default().fg(theme::ELECTRIC_PURPLE)),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(
                &blame.commit_date,
                Style::default().fg(theme::ELECTRIC_YELLOW),
            ),
        ]),
        Line::from(vec![
            Span::styled("Message: ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled(
                &blame.commit_message,
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]),
    ];

    let header = Paragraph::new(header_lines);
    frame.render_widget(header, chunks[0]);

    // Body: explanation with word wrap
    let explanation = Paragraph::new(blame.explanation.clone())
        .style(Style::default().fg(theme::TEXT_PRIMARY))
        .wrap(Wrap { trim: true });
    frame.render_widget(explanation, chunks[1]);
}
