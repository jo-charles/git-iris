//! Modal-related reducer helper functions
//!
//! Provides helper functions for modal operations.
//! The main reducer in mod.rs handles the actual event dispatch.

use super::super::events::{ModalType, RefField};
use super::super::state::{Modal, Mode, RefSelectorTarget, SettingsState, StudioState};

/// Create a modal from a modal type
pub fn create_modal(state: &StudioState, modal_type: ModalType) -> Modal {
    match modal_type {
        ModalType::Help => Modal::Help,
        ModalType::Chat => Modal::Chat,
        ModalType::Settings => Modal::Settings(Box::new(SettingsState::from_config(&state.config))),
        ModalType::PresetSelector => {
            let presets = state.get_commit_presets();
            Modal::PresetSelector {
                input: String::new(),
                presets,
                selected: 0,
                scroll: 0,
            }
        }
        ModalType::EmojiSelector => {
            let emojis = state.get_emoji_list();
            Modal::EmojiSelector {
                input: String::new(),
                emojis,
                selected: 0,
                scroll: 0,
            }
        }
        ModalType::RefSelector { field } => {
            let refs = state.get_branch_refs();
            let target = match field {
                RefField::From => match state.active_mode {
                    Mode::Review => RefSelectorTarget::ReviewFrom,
                    Mode::Changelog => RefSelectorTarget::ChangelogFrom,
                    Mode::ReleaseNotes => RefSelectorTarget::ReleaseNotesFrom,
                    _ => RefSelectorTarget::ReviewFrom,
                },
                RefField::To => match state.active_mode {
                    Mode::Review => RefSelectorTarget::ReviewTo,
                    Mode::Changelog => RefSelectorTarget::ChangelogTo,
                    Mode::ReleaseNotes => RefSelectorTarget::ReleaseNotesTo,
                    _ => RefSelectorTarget::ReviewTo,
                },
                RefField::Base => RefSelectorTarget::PrFrom,
            };
            Modal::RefSelector {
                input: String::new(),
                refs,
                selected: 0,
                target,
            }
        }
        ModalType::ConfirmCommit => {
            if let Some(msg) = state
                .modes
                .commit
                .messages
                .get(state.modes.commit.current_index)
            {
                Modal::Confirm {
                    message: format!("Commit with message:\n\n{}", msg.title),
                    action: "commit".to_string(),
                }
            } else {
                Modal::Confirm {
                    message: "No commit message to commit".to_string(),
                    action: "cancel".to_string(),
                }
            }
        }
        ModalType::ConfirmAmend => {
            if let Some(msg) = state
                .modes
                .commit
                .messages
                .get(state.modes.commit.current_index)
            {
                Modal::Confirm {
                    message: format!("Amend previous commit with message:\n\n{}", msg.title),
                    action: "amend".to_string(),
                }
            } else {
                Modal::Confirm {
                    message: "No commit message to amend with".to_string(),
                    action: "cancel".to_string(),
                }
            }
        }
        ModalType::ConfirmQuit => Modal::Confirm {
            message: "Quit Iris Studio?".to_string(),
            action: "quit".to_string(),
        },
    }
}

/// Apply ref selection to the appropriate mode
pub fn apply_ref_selection(state: &mut StudioState, field: RefField, value: String) {
    match (state.active_mode, field) {
        (Mode::Review, RefField::From) => {
            state.modes.review.from_ref = value;
        }
        (Mode::Review, RefField::To) => {
            state.modes.review.to_ref = value;
        }
        (Mode::PR, RefField::Base) => {
            state.modes.pr.base_branch = value;
        }
        (Mode::PR, RefField::To) => {
            state.modes.pr.to_ref = value;
        }
        (Mode::Changelog, RefField::From) => {
            state.modes.changelog.from_ref = value;
        }
        (Mode::Changelog, RefField::To) => {
            state.modes.changelog.to_ref = value;
        }
        (Mode::ReleaseNotes, RefField::From) => {
            state.modes.release_notes.from_ref = value;
        }
        (Mode::ReleaseNotes, RefField::To) => {
            state.modes.release_notes.to_ref = value;
        }
        _ => {}
    }
}
