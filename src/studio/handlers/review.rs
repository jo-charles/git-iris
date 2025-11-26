//! Review mode key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::studio::state::{Modal, PanelId, RefSelectorTarget, StudioState};

use super::{Action, IrisQueryRequest};

/// Handle key events in Review mode
pub fn handle_review_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match state.focused_panel {
        PanelId::Left => handle_files_key(state, key),
        PanelId::Center => handle_diff_key(state, key),
        PanelId::Right => handle_output_key(state, key),
    }
}

/// Sync file tree selection with diff view in review mode
fn sync_file_selection(state: &mut StudioState) {
    if let Some(path) = state.modes.review.file_tree.selected_path() {
        state.modes.review.diff_view.select_file_by_path(&path);
    }
}

fn handle_files_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.review.file_tree.select_next();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.review.file_tree.select_prev();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('h') | KeyCode::Left => {
            state.modes.review.file_tree.collapse();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.modes.review.file_tree.expand();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('g') => {
            state.modes.review.file_tree.select_first();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('G') => {
            state.modes.review.file_tree.select_last();
            sync_file_selection(state);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Enter => {
            if let Some(entry) = state.modes.review.file_tree.selected_entry() {
                if entry.is_dir {
                    state.modes.review.file_tree.toggle_expand();
                } else {
                    sync_file_selection(state);
                    state.focused_panel = PanelId::Center;
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        // Select from ref
        KeyCode::Char('f') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::ReviewFrom,
            });
            state.mark_dirty();
            Action::Redraw
        }
        // Select to ref
        KeyCode::Char('t') => {
            state.modal = Some(Modal::RefSelector {
                input: String::new(),
                refs: state.get_branch_refs(),
                selected: 0,
                target: RefSelectorTarget::ReviewTo,
            });
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_diff_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            state.modes.review.diff_view.scroll_down(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.review.diff_view.scroll_up(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.review.diff_view.scroll_down(20);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.review.diff_view.scroll_up(20);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char(']') => {
            state.modes.review.diff_view.next_hunk();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('[') => {
            state.modes.review.diff_view.prev_hunk();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('n') => {
            state.modes.review.diff_view.next_file();
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('p') => {
            state.modes.review.diff_view.prev_file();
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_output_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        // Scroll review output
        KeyCode::Char('j') | KeyCode::Down => {
            let max_scroll = state
                .modes
                .review
                .review_content
                .lines()
                .count()
                .saturating_sub(10);
            state.modes.review.review_scroll =
                (state.modes.review.review_scroll + 1).min(max_scroll);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.modes.review.review_scroll = state.modes.review.review_scroll.saturating_sub(1);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let max_scroll = state
                .modes
                .review
                .review_content
                .lines()
                .count()
                .saturating_sub(10);
            state.modes.review.review_scroll =
                (state.modes.review.review_scroll + 20).min(max_scroll);
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.modes.review.review_scroll = state.modes.review.review_scroll.saturating_sub(20);
            state.mark_dirty();
            Action::Redraw
        }
        // Generate review
        KeyCode::Char('r') => {
            state.set_iris_thinking("Generating code review...");
            state.modes.review.generating = true;
            Action::IrisQuery(IrisQueryRequest::GenerateReview)
        }
        // Reset review
        KeyCode::Char('R') => {
            state.modes.review.review_content.clear();
            state.modes.review.review_scroll = 0;
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}
