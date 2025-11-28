//! PR mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, PanelId, RefSelectorTarget, StudioState};

use super::{copy_to_clipboard, spawn_pr_task};

/// Handle key events in PR mode
pub fn handle_pr_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
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
            if state.modes.pr.selected_commit > 0 {
                state.modes.pr.selected_commit -= 1;
                // Adjust scroll if needed
                if state.modes.pr.selected_commit < state.modes.pr.commit_scroll {
                    state.modes.pr.commit_scroll = state.modes.pr.selected_commit;
                }
                state.mark_dirty();
            }
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.modes.pr.selected_commit + 1 < state.modes.pr.commits.len() {
                state.modes.pr.selected_commit += 1;
                state.mark_dirty();
            }
            vec![]
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.pr.selected_commit = 0;
            state.modes.pr.commit_scroll = 0;
            state.mark_dirty();
            vec![]
        }
        KeyCode::End | KeyCode::Char('G') => {
            if !state.modes.pr.commits.is_empty() {
                state.modes.pr.selected_commit = state.modes.pr.commits.len() - 1;
                state.mark_dirty();
            }
            vec![]
        }
        // Select from ref (base branch)
        KeyCode::Char('f') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::PrFrom,
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
                target: RefSelectorTarget::PrTo,
            });
            state.mark_dirty();
            vec![]
        }
        // Generate PR
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating PR description...");
            state.modes.pr.generating = true;
            vec![spawn_pr_task(state)]
        }
        _ => vec![],
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        // Scroll diff
        KeyCode::Up | KeyCode::Char('k') => {
            state.modes.pr.diff_view.scroll_up(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.modes.pr.diff_view.scroll_down(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.pr.diff_view.scroll_up(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.pr.diff_view.scroll_down(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.pr.diff_view.scroll_to_top();
            state.mark_dirty();
            vec![]
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.modes.pr.diff_view.scroll_to_bottom();
            state.mark_dirty();
            vec![]
        }
        // Hunk navigation
        KeyCode::Char(']') => {
            state.modes.pr.diff_view.next_hunk();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('[') => {
            state.modes.pr.diff_view.prev_hunk();
            state.mark_dirty();
            vec![]
        }
        // File navigation within diff
        KeyCode::Char('n') => {
            state.modes.pr.diff_view.next_file();
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char('p') => {
            state.modes.pr.diff_view.prev_file();
            state.mark_dirty();
            vec![]
        }
        // Generate PR
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating PR description...");
            state.modes.pr.generating = true;
            vec![spawn_pr_task(state)]
        }
        _ => vec![],
    }
}

fn handle_output_key(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    let content_lines = state.modes.pr.pr_content.lines().count();

    match key.code {
        // Scroll PR content
        KeyCode::Up | KeyCode::Char('k') => {
            state.modes.pr.pr_scroll = state.modes.pr.pr_scroll.saturating_sub(1);
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.modes.pr.pr_scroll + 1 < content_lines {
                state.modes.pr.pr_scroll += 1;
                state.mark_dirty();
            }
            vec![]
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.pr.pr_scroll = state.modes.pr.pr_scroll.saturating_sub(20);
            state.mark_dirty();
            vec![]
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.pr.pr_scroll =
                (state.modes.pr.pr_scroll + 20).min(content_lines.saturating_sub(1));
            state.mark_dirty();
            vec![]
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.modes.pr.pr_scroll = 0;
            state.mark_dirty();
            vec![]
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.modes.pr.pr_scroll = content_lines.saturating_sub(1);
            state.mark_dirty();
            vec![]
        }
        // Generate PR
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating PR description...");
            state.modes.pr.generating = true;
            vec![spawn_pr_task(state)]
        }
        // Copy to clipboard
        KeyCode::Char('y') => {
            if !state.modes.pr.pr_content.is_empty() {
                let content = state.modes.pr.pr_content.clone();
                copy_to_clipboard(state, &content, "PR description");
            }
            vec![]
        }
        // Reset
        KeyCode::Char('R') => {
            state.modes.pr.pr_content.clear();
            state.modes.pr.pr_scroll = 0;
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
