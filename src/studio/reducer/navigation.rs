//! Navigation-related reducer helper functions
//!
//! Provides the `apply_scroll` helper for scroll handling.
//! The main reducer in mod.rs handles the actual event dispatch.

use super::super::events::ScrollDirection;
use super::super::state::{Mode, PanelId, StudioState};

/// Apply scroll to the current focused panel
#[allow(clippy::cognitive_complexity)]
pub fn apply_scroll(state: &mut StudioState, direction: ScrollDirection, amount: usize) {
    match state.active_mode {
        Mode::Explore => match state.focused_panel {
            PanelId::Left => {
                // File tree navigation
                match direction {
                    ScrollDirection::Up => {
                        for _ in 0..amount {
                            state.modes.explore.file_tree.select_prev();
                        }
                    }
                    ScrollDirection::Down => {
                        for _ in 0..amount {
                            state.modes.explore.file_tree.select_next();
                        }
                    }
                    ScrollDirection::PageUp => {
                        state.modes.explore.file_tree.page_up(amount);
                    }
                    ScrollDirection::PageDown => {
                        state.modes.explore.file_tree.page_down(amount);
                    }
                    ScrollDirection::Top => {
                        state.modes.explore.file_tree.select_first();
                    }
                    ScrollDirection::Bottom => {
                        state.modes.explore.file_tree.select_last();
                    }
                }
            }
            PanelId::Center => {
                // Code view scroll
                match direction {
                    ScrollDirection::Up => {
                        state.modes.explore.code_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.explore.code_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
            PanelId::Right => {
                // Context/blame panel scroll - no scroll state yet
            }
        },
        Mode::Commit => match state.focused_panel {
            PanelId::Left => {
                // File tree navigation
                match direction {
                    ScrollDirection::Up => {
                        for _ in 0..amount {
                            state.modes.commit.file_tree.select_prev();
                        }
                    }
                    ScrollDirection::Down => {
                        for _ in 0..amount {
                            state.modes.commit.file_tree.select_next();
                        }
                    }
                    ScrollDirection::PageUp => {
                        state.modes.commit.file_tree.page_up(amount);
                    }
                    ScrollDirection::PageDown => {
                        state.modes.commit.file_tree.page_down(amount);
                    }
                    ScrollDirection::Top => {
                        state.modes.commit.file_tree.select_first();
                    }
                    ScrollDirection::Bottom => {
                        state.modes.commit.file_tree.select_last();
                    }
                }
            }
            PanelId::Center => {
                // Message editor scroll - handled by component
            }
            PanelId::Right => {
                // Diff view scroll
                match direction {
                    ScrollDirection::Up => {
                        state.modes.commit.diff_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.commit.diff_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
        },
        Mode::Review => match state.focused_panel {
            PanelId::Left => {
                // File tree navigation
                match direction {
                    ScrollDirection::Up => {
                        for _ in 0..amount {
                            state.modes.review.file_tree.select_prev();
                        }
                    }
                    ScrollDirection::Down => {
                        for _ in 0..amount {
                            state.modes.review.file_tree.select_next();
                        }
                    }
                    _ => {}
                }
            }
            PanelId::Center => {
                // Review content scroll (center panel shows review, not diff)
                let max_scroll = state
                    .modes
                    .review
                    .review_content
                    .lines()
                    .count()
                    .saturating_sub(1);
                match direction {
                    ScrollDirection::Up => {
                        state.modes.review.review_scroll =
                            state.modes.review.review_scroll.saturating_sub(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.review.review_scroll =
                            (state.modes.review.review_scroll + amount).min(max_scroll);
                    }
                    _ => {}
                }
            }
            PanelId::Right => {
                // Diff view scroll (right panel shows diff)
                match direction {
                    ScrollDirection::Up => {
                        state.modes.review.diff_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.review.diff_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
        },
        Mode::PR => match state.focused_panel {
            PanelId::Left => {
                // Commits list navigation
                match direction {
                    ScrollDirection::Up => {
                        if state.modes.pr.selected_commit > 0 {
                            state.modes.pr.selected_commit =
                                state.modes.pr.selected_commit.saturating_sub(amount);
                            if state.modes.pr.selected_commit < state.modes.pr.commit_scroll {
                                state.modes.pr.commit_scroll = state.modes.pr.selected_commit;
                            }
                        }
                    }
                    ScrollDirection::Down => {
                        let max_idx = state.modes.pr.commits.len().saturating_sub(1);
                        state.modes.pr.selected_commit =
                            (state.modes.pr.selected_commit + amount).min(max_idx);
                    }
                    _ => {}
                }
            }
            PanelId::Center => {
                // PR content scroll (center panel shows PR description)
                let max_scroll = state.modes.pr.pr_content.lines().count().saturating_sub(1);
                match direction {
                    ScrollDirection::Up => {
                        state.modes.pr.pr_scroll = state.modes.pr.pr_scroll.saturating_sub(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.pr.pr_scroll =
                            (state.modes.pr.pr_scroll + amount).min(max_scroll);
                    }
                    _ => {}
                }
            }
            PanelId::Right => {
                // Diff view scroll (right panel shows diff)
                match direction {
                    ScrollDirection::Up => {
                        state.modes.pr.diff_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.pr.diff_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
        },
        Mode::Changelog => match state.focused_panel {
            PanelId::Left => {
                // Commits list navigation
                match direction {
                    ScrollDirection::Up => {
                        if state.modes.changelog.selected_commit > 0 {
                            state.modes.changelog.selected_commit =
                                state.modes.changelog.selected_commit.saturating_sub(amount);
                            if state.modes.changelog.selected_commit
                                < state.modes.changelog.commit_scroll
                            {
                                state.modes.changelog.commit_scroll =
                                    state.modes.changelog.selected_commit;
                            }
                        }
                    }
                    ScrollDirection::Down => {
                        let max_idx = state.modes.changelog.commits.len().saturating_sub(1);
                        state.modes.changelog.selected_commit =
                            (state.modes.changelog.selected_commit + amount).min(max_idx);
                    }
                    _ => {}
                }
            }
            PanelId::Center => {
                // Changelog content scroll
                let max_scroll = state
                    .modes
                    .changelog
                    .changelog_content
                    .lines()
                    .count()
                    .saturating_sub(1);
                match direction {
                    ScrollDirection::Up => {
                        state.modes.changelog.changelog_scroll = state
                            .modes
                            .changelog
                            .changelog_scroll
                            .saturating_sub(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.changelog.changelog_scroll =
                            (state.modes.changelog.changelog_scroll + amount).min(max_scroll);
                    }
                    _ => {}
                }
            }
            PanelId::Right => {
                // Diff view scroll
                match direction {
                    ScrollDirection::Up => {
                        state.modes.changelog.diff_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.changelog.diff_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
        },
        Mode::ReleaseNotes => match state.focused_panel {
            PanelId::Left => {
                // Commits list navigation
                match direction {
                    ScrollDirection::Up => {
                        if state.modes.release_notes.selected_commit > 0 {
                            state.modes.release_notes.selected_commit = state
                                .modes
                                .release_notes
                                .selected_commit
                                .saturating_sub(amount);
                            if state.modes.release_notes.selected_commit
                                < state.modes.release_notes.commit_scroll
                            {
                                state.modes.release_notes.commit_scroll =
                                    state.modes.release_notes.selected_commit;
                            }
                        }
                    }
                    ScrollDirection::Down => {
                        let max_idx = state.modes.release_notes.commits.len().saturating_sub(1);
                        state.modes.release_notes.selected_commit =
                            (state.modes.release_notes.selected_commit + amount).min(max_idx);
                    }
                    _ => {}
                }
            }
            PanelId::Center => {
                // Release notes content scroll
                let max_scroll = state
                    .modes
                    .release_notes
                    .release_notes_content
                    .lines()
                    .count()
                    .saturating_sub(1);
                match direction {
                    ScrollDirection::Up => {
                        state.modes.release_notes.release_notes_scroll = state
                            .modes
                            .release_notes
                            .release_notes_scroll
                            .saturating_sub(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.release_notes.release_notes_scroll =
                            (state.modes.release_notes.release_notes_scroll + amount)
                                .min(max_scroll);
                    }
                    _ => {}
                }
            }
            PanelId::Right => {
                // Diff view scroll
                match direction {
                    ScrollDirection::Up => {
                        state.modes.release_notes.diff_view.scroll_up(amount);
                    }
                    ScrollDirection::Down => {
                        state.modes.release_notes.diff_view.scroll_down(amount);
                    }
                    _ => {}
                }
            }
        },
    }
    state.mark_dirty();
}
