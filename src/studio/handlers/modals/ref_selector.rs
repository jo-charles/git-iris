//! Ref selector modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, Notification, RefSelectorTarget, StudioState};

use super::super::{
    reload_changelog_data, reload_pr_data, reload_release_notes_data, reload_review_data,
};

/// Handle key events in ref selector modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Get current state for filtering
    let (input, refs, selected, target) = if let Some(Modal::RefSelector {
        input,
        refs,
        selected,
        target,
    }) = &state.modal
    {
        (input.clone(), refs.clone(), *selected, *target)
    } else {
        return vec![];
    };

    // Filter refs based on input
    let filtered: Vec<_> = refs
        .iter()
        .filter(|r| input.is_empty() || r.to_lowercase().contains(&input.to_lowercase()))
        .collect();

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Enter => {
            // Apply selection and determine which data reload effect is needed
            enum ReloadType {
                None,
                Pr,
                Review,
                Changelog,
                ReleaseNotes,
            }

            // Determine which ref to use:
            // 1. If there's a matching filtered ref, use that
            // 2. Otherwise, if input is not empty, use it as a custom ref (e.g., HEAD~5)
            let ref_to_use: Option<String> =
                filtered.get(selected).map(|s| (*s).clone()).or_else(|| {
                    if input.is_empty() {
                        None
                    } else {
                        Some(input.clone())
                    }
                });

            let reload_type = if let Some(ref_value) = ref_to_use {
                let (label, reload) = match target {
                    RefSelectorTarget::ReviewFrom => {
                        state.modes.review.from_ref.clone_from(&ref_value);
                        ("Review from", ReloadType::Review)
                    }
                    RefSelectorTarget::ReviewTo => {
                        state.modes.review.to_ref.clone_from(&ref_value);
                        ("Review to", ReloadType::Review)
                    }
                    RefSelectorTarget::PrFrom => {
                        state.modes.pr.base_branch.clone_from(&ref_value);
                        ("PR base", ReloadType::Pr)
                    }
                    RefSelectorTarget::PrTo => {
                        state.modes.pr.to_ref.clone_from(&ref_value);
                        ("PR target", ReloadType::Pr)
                    }
                    RefSelectorTarget::ChangelogFrom => {
                        state.modes.changelog.from_ref.clone_from(&ref_value);
                        ("Changelog from", ReloadType::Changelog)
                    }
                    RefSelectorTarget::ChangelogTo => {
                        state.modes.changelog.to_ref.clone_from(&ref_value);
                        ("Changelog to", ReloadType::Changelog)
                    }
                    RefSelectorTarget::ReleaseNotesFrom => {
                        state.modes.release_notes.from_ref.clone_from(&ref_value);
                        ("Release Notes from", ReloadType::ReleaseNotes)
                    }
                    RefSelectorTarget::ReleaseNotesTo => {
                        state.modes.release_notes.to_ref.clone_from(&ref_value);
                        ("Release Notes to", ReloadType::ReleaseNotes)
                    }
                };
                state.notify(Notification::info(format!("{label} set to {ref_value}")));
                reload
            } else {
                ReloadType::None
            };
            state.close_modal();
            match reload_type {
                ReloadType::Pr => vec![reload_pr_data(state)],
                ReloadType::Review => vec![reload_review_data(state)],
                ReloadType::Changelog => vec![reload_changelog_data(state)],
                ReloadType::ReleaseNotes => vec![reload_release_notes_data(state)],
                ReloadType::None => vec![],
            }
        }
        KeyCode::Up => {
            if let Some(Modal::RefSelector { selected, .. }) = &mut state.modal {
                *selected = selected.saturating_sub(1);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down => {
            if let Some(Modal::RefSelector {
                selected,
                refs,
                input,
                ..
            }) = &mut state.modal
            {
                let filtered_len = refs
                    .iter()
                    .filter(|r| {
                        input.is_empty() || r.to_lowercase().contains(&input.to_lowercase())
                    })
                    .count();
                if *selected + 1 < filtered_len {
                    *selected += 1;
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char(c) => {
            if let Some(Modal::RefSelector {
                input, selected, ..
            }) = &mut state.modal
            {
                input.push(c);
                *selected = 0; // Reset selection on filter change
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::RefSelector {
                input, selected, ..
            }) = &mut state.modal
            {
                input.pop();
                *selected = 0;
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
