//! Changelog mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, PanelId, RefSelectorTarget, StudioState};

use super::{copy_to_clipboard, spawn_changelog_task};

/// Handle key events in Changelog mode
pub fn handle_changelog_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match state.focused_panel {
        PanelId::Left => handle_commits_key(state, key),
        PanelId::Center => handle_output_key(state, key),
        PanelId::Right => handle_diff_key(state, key),
    }
}

fn handle_commits_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
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
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.modes.changelog.selected_commit + 1 < state.modes.changelog.commits.len() {
                state.modes.changelog.selected_commit += 1;
                state.mark_dirty();
            }
            vec![]
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.changelog.selected_commit = 0;
            state.modes.changelog.commit_scroll = 0;
            state.mark_dirty();
            vec![]
        }
        KeyCode::End | KeyCode::Char('G') => {
            if !state.modes.changelog.commits.is_empty() {
                state.modes.changelog.selected_commit = state.modes.changelog.commits.len() - 1;
                state.mark_dirty();
            }
            vec![]
        }
        // Select from ref
        KeyCode::Char('f') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::ChangelogFrom,
            });
            state.mark_dirty();
            vec![]
        }
        // Select to ref
        KeyCode::Char('t') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::ChangelogTo,
            });
            state.mark_dirty();
            vec![]
        }
        // Generate changelog
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating changelog...");
            state.modes.changelog.generating = true;
            vec![spawn_changelog_task(state)]
        }
        _ => vec![],
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Scroll diff
        KeyCode::Up | KeyCode::Char('k') => {
            state.modes.changelog.diff_view.scroll_up(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.modes.changelog.diff_view.scroll_down(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.changelog.diff_view.scroll_up(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.changelog.diff_view.scroll_down(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.changelog.diff_view.scroll_to_top();
            state.mark_dirty();
            vec![]
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.modes.changelog.diff_view.scroll_to_bottom();
            state.mark_dirty();
            vec![]
        }
        // Hunk navigation
        KeyCode::Char(']') => {
            state.modes.changelog.diff_view.next_hunk();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('[') => {
            state.modes.changelog.diff_view.prev_hunk();
            state.mark_dirty();
            vec![]
        }
        // File navigation within diff
        KeyCode::Char('n') => {
            state.modes.changelog.diff_view.next_file();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('p') => {
            state.modes.changelog.diff_view.prev_file();
            state.mark_dirty();
            vec![]
        }
        // Generate changelog
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating changelog...");
            state.modes.changelog.generating = true;
            vec![spawn_changelog_task(state)]
        }
        _ => vec![],
    }
}

fn handle_output_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    let content_lines = state.modes.changelog.changelog_content.lines().count();

    match key.code {
        // Scroll changelog content
        KeyCode::Up | KeyCode::Char('k') => {
            state.modes.changelog.changelog_scroll =
                state.modes.changelog.changelog_scroll.saturating_sub(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.modes.changelog.changelog_scroll + 1 < content_lines {
                state.modes.changelog.changelog_scroll += 1;
                state.mark_dirty();
            }
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.changelog.changelog_scroll =
                state.modes.changelog.changelog_scroll.saturating_sub(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.changelog.changelog_scroll =
                (state.modes.changelog.changelog_scroll + 20).min(content_lines.saturating_sub(1));
            state.mark_dirty();
            vec![]
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.changelog.changelog_scroll = 0;
            state.mark_dirty();
            vec![]
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.modes.changelog.changelog_scroll = content_lines.saturating_sub(1);
            state.mark_dirty();
            vec![]
        }
        // Generate changelog
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating changelog...");
            state.modes.changelog.generating = true;
            vec![spawn_changelog_task(state)]
        }
        // Copy to clipboard
        KeyCode::Char('y') => {
            if !state.modes.changelog.changelog_content.is_empty() {
                let content = state.modes.changelog.changelog_content.clone();
                copy_to_clipboard(state, &content, "Changelog");
            }
            vec![]
        }
        // Reset
        KeyCode::Char('R') => {
            state.modes.changelog.changelog_content.clear();
            state.modes.changelog.changelog_scroll = 0;
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
