//! Changelog mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::state::{Modal, PanelId, RefSelectorTarget, StudioState};

use super::{Action, IrisQueryRequest, copy_to_clipboard};

/// Handle key events in Changelog mode
pub fn handle_changelog_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match state.focused_panel {
        PanelId::Left => handle_commits_key(state, key),
        PanelId::Center => handle_diff_key(state, key),
        PanelId::Right => handle_output_key(state, key),
    }
}

fn handle_commits_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if state.modes.changelog.selected_commit > 0 {
                state.modes.changelog.selected_commit -= 1;
                // Adjust scroll if needed
                if state.modes.changelog.selected_commit < state.modes.changelog.commit_scroll {
                    state.modes.changelog.commit_scroll = state.modes.changelog.selected_commit;
                }
                state.mark_dirty();
            }
            Action::Redraw
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.modes.changelog.selected_commit + 1 < state.modes.changelog.commits.len() {
                state.modes.changelog.selected_commit += 1;
                state.mark_dirty();
            }
            Action::Redraw
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.changelog.selected_commit = 0;
            state.modes.changelog.commit_scroll = 0;
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::End | KeyCode::Char('G') => {
            if !state.modes.changelog.commits.is_empty() {
                state.modes.changelog.selected_commit = state.modes.changelog.commits.len() - 1;
                state.mark_dirty();
            }
            Action::Redraw
        }
        // Select from ref
        KeyCode::Char('f') => open_from_ref_selector(state),
        // Select to ref
        KeyCode::Char('t') => open_to_ref_selector(state),
        // Generate changelog
        KeyCode::Char('r') => generate_changelog(state),
        _ => Action::None,
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Scroll diff
        KeyCode::Up | KeyCode::Char('k') => {
            state.modes.changelog.diff_view.scroll_up(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.modes.changelog.diff_view.scroll_down(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp => {
            state.modes.changelog.diff_view.scroll_up(20);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown => {
            state.modes.changelog.diff_view.scroll_down(20);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.changelog.diff_view.scroll_to_top();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.modes.changelog.diff_view.scroll_to_bottom();
            state.mark_dirty();
            Action::Redraw
        }
        // Generate changelog
        KeyCode::Char('r') => generate_changelog(state),
        _ => Action::None,
    }
}

fn handle_output_key(state: &mut StudioState, key: KeyEvent) -> Action {
    let content_lines = state.modes.changelog.changelog_content.lines().count();

    match key.code {
        // Scroll changelog content
        KeyCode::Up | KeyCode::Char('k') => {
            state.modes.changelog.changelog_scroll =
                state.modes.changelog.changelog_scroll.saturating_sub(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.modes.changelog.changelog_scroll + 1 < content_lines {
                state.modes.changelog.changelog_scroll += 1;
                state.mark_dirty();
            }
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.changelog.changelog_scroll =
                state.modes.changelog.changelog_scroll.saturating_sub(20);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.changelog.changelog_scroll =
                (state.modes.changelog.changelog_scroll + 20).min(content_lines.saturating_sub(1));
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.changelog.changelog_scroll = 0;
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.modes.changelog.changelog_scroll = content_lines.saturating_sub(1);
            state.mark_dirty();
            Action::Redraw
        }
        // Generate changelog
        KeyCode::Char('r') => generate_changelog(state),
        // Copy to clipboard
        KeyCode::Char('y') => {
            if state.modes.changelog.changelog_content.is_empty() {
                Action::None
            } else {
                let content = state.modes.changelog.changelog_content.clone();
                copy_to_clipboard(state, &content, "Changelog")
            }
        }
        // Reset
        KeyCode::Char('R') => {
            state.modes.changelog.changelog_content.clear();
            state.modes.changelog.changelog_scroll = 0;
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn open_from_ref_selector(state: &mut StudioState) -> Action {
    state.modal = Some(Modal::RefSelector {
        input: String::new(),
        refs: state.get_branch_refs(),
        selected: 0,
        target: RefSelectorTarget::ChangelogFrom,
    });
    state.mark_dirty();
    Action::Redraw
}

fn open_to_ref_selector(state: &mut StudioState) -> Action {
    state.modal = Some(Modal::RefSelector {
        input: String::new(),
        refs: state.get_branch_refs(),
        selected: 0,
        target: RefSelectorTarget::ChangelogTo,
    });
    state.mark_dirty();
    Action::Redraw
}

fn generate_changelog(state: &mut StudioState) -> Action {
    state.set_iris_thinking("Generating changelog...");
    state.modes.changelog.generating = true;
    let from_ref = state.modes.changelog.from_ref.clone();
    let to_ref = state.modes.changelog.to_ref.clone();
    Action::IrisQuery(IrisQueryRequest::GenerateChangelog { from_ref, to_ref })
}
