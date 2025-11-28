//! Navigation-related reducer functions
//!
//! Handles: `SwitchMode`, `FocusPanel`, `FocusNext`, `FocusPrev`, Scroll, `SelectFile`

use super::super::events::{DataType, ScrollDirection, SideEffect, StudioEvent};
use super::super::history::History;
use super::super::state::{Mode, PanelId, StudioState};

/// Reduce navigation-related events
pub fn reduce(
    state: &mut StudioState,
    event: StudioEvent,
    history: &mut History,
) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    match event {
        StudioEvent::SwitchMode(new_mode) => {
            let old_mode = state.active_mode;
            if old_mode != new_mode {
                history.record_mode_switch(old_mode, new_mode);
                state.switch_mode(new_mode);

                // Trigger data loading for the new mode
                effects.extend(get_mode_data_load_effect(state, new_mode));
            }
        }

        StudioEvent::FocusPanel(panel) => {
            state.focused_panel = panel;
            state.mark_dirty();
        }

        StudioEvent::FocusNext => {
            state.focus_next_panel();
            state.mark_dirty();
        }

        StudioEvent::FocusPrev => {
            state.focus_prev_panel();
            state.mark_dirty();
        }

        StudioEvent::Scroll { direction, amount } => {
            apply_scroll(state, direction, amount);
        }

        StudioEvent::SelectFile(path) => {
            match state.active_mode {
                Mode::Explore => {
                    state.modes.explore.current_file = Some(path);
                }
                Mode::Commit => {
                    if let Some(idx) = state
                        .git_status
                        .staged_files
                        .iter()
                        .position(|f| f == &path)
                    {
                        state.modes.commit.selected_file_index = idx;
                    }
                }
                _ => {}
            }
            state.mark_dirty();
        }

        _ => {}
    }

    effects
}

fn get_mode_data_load_effect(state: &StudioState, mode: Mode) -> Vec<SideEffect> {
    match mode {
        Mode::Commit => {
            vec![SideEffect::LoadData {
                data_type: DataType::CommitDiff,
                from_ref: None,
                to_ref: None,
            }]
        }
        Mode::Review => {
            let from = state.modes.review.from_ref.clone();
            let to = state.modes.review.to_ref.clone();
            vec![SideEffect::LoadData {
                data_type: DataType::ReviewDiff,
                from_ref: Some(from),
                to_ref: Some(to),
            }]
        }
        Mode::PR => {
            let base = state.modes.pr.base_branch.clone();
            let to = state.modes.pr.to_ref.clone();
            vec![SideEffect::LoadData {
                data_type: DataType::PRDiff,
                from_ref: Some(base),
                to_ref: Some(to),
            }]
        }
        Mode::Changelog => {
            let from = state.modes.changelog.from_ref.clone();
            let to = state.modes.changelog.to_ref.clone();
            vec![SideEffect::LoadData {
                data_type: DataType::ChangelogCommits,
                from_ref: Some(from),
                to_ref: Some(to),
            }]
        }
        Mode::ReleaseNotes => {
            let from = state.modes.release_notes.from_ref.clone();
            let to = state.modes.release_notes.to_ref.clone();
            vec![SideEffect::LoadData {
                data_type: DataType::ReleaseNotesCommits,
                from_ref: Some(from),
                to_ref: Some(to),
            }]
        }
        Mode::Explore => vec![],
    }
}

/// Apply scroll to the current focused panel
#[allow(clippy::cognitive_complexity)]
pub fn apply_scroll(state: &mut StudioState, direction: ScrollDirection, amount: usize) {
    match state.active_mode {
        Mode::Explore => match state.focused_panel {
            PanelId::Left => match direction {
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
            },
            PanelId::Center => match direction {
                ScrollDirection::Up => {
                    state.modes.explore.code_view.scroll_up(amount);
                }
                ScrollDirection::Down => {
                    state.modes.explore.code_view.scroll_down(amount);
                }
                _ => {}
            },
            PanelId::Right => {}
        },
        Mode::Commit => match state.focused_panel {
            PanelId::Left => match direction {
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
            },
            PanelId::Center => {}
            PanelId::Right => match direction {
                ScrollDirection::Up => {
                    state.modes.commit.diff_view.scroll_up(amount);
                }
                ScrollDirection::Down => {
                    state.modes.commit.diff_view.scroll_down(amount);
                }
                _ => {}
            },
        },
        Mode::Review => match state.focused_panel {
            PanelId::Left => match direction {
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
            },
            PanelId::Center => match direction {
                ScrollDirection::Up => {
                    state.modes.review.diff_view.scroll_up(amount);
                }
                ScrollDirection::Down => {
                    state.modes.review.diff_view.scroll_down(amount);
                }
                _ => {}
            },
            PanelId::Right => {
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
        },
        Mode::PR => match state.focused_panel {
            PanelId::Left => match direction {
                ScrollDirection::Up => {
                    for _ in 0..amount {
                        state.modes.pr.file_tree.select_prev();
                    }
                }
                ScrollDirection::Down => {
                    for _ in 0..amount {
                        state.modes.pr.file_tree.select_next();
                    }
                }
                _ => {}
            },
            PanelId::Center => match direction {
                ScrollDirection::Up => {
                    state.modes.pr.diff_view.scroll_up(amount);
                }
                ScrollDirection::Down => {
                    state.modes.pr.diff_view.scroll_down(amount);
                }
                _ => {}
            },
            PanelId::Right => {
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
        },
        Mode::Changelog => match state.focused_panel {
            PanelId::Center | PanelId::Right => {
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
            PanelId::Left => {}
        },
        Mode::ReleaseNotes => match state.focused_panel {
            PanelId::Center | PanelId::Right => {
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
            PanelId::Left => {}
        },
    }
    state.mark_dirty();
}
