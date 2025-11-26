//! Commit mode rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::studio::components::{render_diff_view, render_file_tree, render_message_editor};
use crate::studio::state::{EmojiMode, PanelId, StudioState};

/// Render a panel in Commit mode
pub fn render_commit_panel(
    state: &mut StudioState,
    frame: &mut Frame,
    area: Rect,
    panel_id: PanelId,
) {
    let is_focused = panel_id == state.focused_panel;

    match panel_id {
        PanelId::Left => {
            // Build title with file counts
            let staged = state.git_status.staged_count;
            let modified = state.git_status.modified_count;
            let title = if modified > 0 {
                format!("Changes ✓{} ○{}", staged, modified)
            } else if staged > 0 {
                format!("Changes ✓{}", staged)
            } else {
                "Changes".to_string()
            };

            render_file_tree(
                frame,
                area,
                &mut state.modes.commit.file_tree,
                &title,
                is_focused,
            );
        }
        PanelId::Center => {
            // Build title with current style settings
            let emoji_indicator = match &state.modes.commit.emoji_mode {
                EmojiMode::None => "—",
                EmojiMode::Auto => "✨",
                EmojiMode::Custom(e) => e,
            };
            let preset_name = &state.modes.commit.preset;
            let title = format!("Message · {} {}", preset_name, emoji_indicator);

            // Render message editor
            render_message_editor(
                frame,
                area,
                &state.modes.commit.message_editor,
                &title,
                is_focused,
                state.modes.commit.generating,
            );
        }
        PanelId::Right => {
            // Render diff view for selected file
            let title = state.modes.commit.file_tree.selected_path().map_or_else(
                || "Changes".to_string(),
                |p| format!("◈ {}", p.file_name().unwrap_or_default().to_string_lossy()),
            );
            render_diff_view(
                frame,
                area,
                &state.modes.commit.diff_view,
                &title,
                is_focused,
            );
        }
    }
}
