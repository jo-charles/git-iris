//! Modal key handling for Iris Studio

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::state::{EmojiMode, Modal, Notification, RefSelectorTarget, StudioState};

use super::{Action, IrisQueryRequest};

/// Handle key events when a modal is open
pub fn handle_modal_key(state: &mut StudioState, key: KeyEvent) -> Action {
    match &state.modal {
        Some(Modal::Help) => {
            // Any key closes help
            state.close_modal();
            Action::Redraw
        }
        Some(Modal::Search { .. }) => handle_search_modal(state, key),
        Some(Modal::Confirm { .. }) => handle_confirm_modal(state, key),
        Some(Modal::Instructions { .. }) => handle_instructions_modal(state, key),
        Some(Modal::Chat(_)) => handle_chat_modal(state, key),
        Some(Modal::RefSelector { .. }) => handle_ref_selector_modal(state, key),
        Some(Modal::PresetSelector { .. }) => handle_preset_selector_modal(state, key),
        Some(Modal::EmojiSelector { .. }) => handle_emoji_selector_modal(state, key),
        None => Action::None,
    }
}

fn handle_search_modal(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Enter => {
            // TODO: Handle search selection
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Char(c) => {
            // TODO: Update search query
            let _ = c;
            Action::Redraw
        }
        KeyCode::Backspace => {
            // TODO: Update search query
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_confirm_modal(state: &mut StudioState, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y' | 'Y') => {
            // TODO: Execute confirmed action
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Char('n' | 'N') | KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_instructions_modal(state: &mut StudioState, key: KeyEvent) -> Action {
    // Get current input for Enter handling
    let current_input = if let Some(Modal::Instructions { input }) = &state.modal {
        input.clone()
    } else {
        String::new()
    };

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Generate commit with instructions
            let instructions = if current_input.is_empty() {
                None
            } else {
                Some(current_input)
            };
            let preset = state.modes.commit.preset.clone();
            let use_gitmoji = state.modes.commit.use_gitmoji;
            state.close_modal();
            state.set_iris_thinking("Generating commit message...");
            state.modes.commit.generating = true;
            Action::IrisQuery(IrisQueryRequest::GenerateCommit {
                instructions,
                preset,
                use_gitmoji,
            })
        }
        KeyCode::Char(c) => {
            if let Some(Modal::Instructions { input }) = &mut state.modal {
                input.push(c);
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Backspace => {
            if let Some(Modal::Instructions { input }) = &mut state.modal {
                input.pop();
            }
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_chat_modal(state: &mut StudioState, key: KeyEvent) -> Action {
    // Get current state for Enter handling
    let (current_input, is_responding) = if let Some(Modal::Chat(chat)) = &state.modal {
        (chat.input.clone(), chat.is_responding)
    } else {
        (String::new(), false)
    };

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Send message if not empty and not already responding
            if !current_input.is_empty() && !is_responding {
                let message = current_input;
                if let Some(Modal::Chat(chat)) = &mut state.modal {
                    chat.add_user_message(&message);
                    chat.is_responding = true;
                }
                state.mark_dirty();
                Action::IrisQuery(IrisQueryRequest::Chat { message })
            } else {
                Action::None
            }
        }
        KeyCode::Char(c) => {
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.input.push(c);
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Backspace => {
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.input.pop();
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Up => {
            // Scroll up in chat history
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.scroll_offset = chat.scroll_offset.saturating_add(1);
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Down => {
            // Scroll down in chat history
            if let Some(Modal::Chat(chat)) = &mut state.modal {
                chat.scroll_offset = chat.scroll_offset.saturating_sub(1);
            }
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_preset_selector_modal(state: &mut StudioState, key: KeyEvent) -> Action {
    // Visible items in the list (modal height - header - footer)
    const VISIBLE_ITEMS: usize = 18;

    // Get current state for filtering
    let (input, presets, selected) = if let Some(Modal::PresetSelector {
        input,
        presets,
        selected,
        ..
    }) = &state.modal
    {
        (input.clone(), presets.clone(), *selected)
    } else {
        return Action::None;
    };

    // Filter presets based on input
    let filtered: Vec<_> = presets
        .iter()
        .filter(|p| {
            input.is_empty()
                || p.name.to_lowercase().contains(&input.to_lowercase())
                || p.key.to_lowercase().contains(&input.to_lowercase())
        })
        .collect();

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Apply selection and auto-regenerate
            if let Some(preset) = filtered.get(selected) {
                state.modes.commit.preset.clone_from(&preset.key);
                state.notify(Notification::info(format!(
                    "Preset: {} {} - regenerating...",
                    preset.emoji, preset.name
                )));
                let preset_key = preset.key.clone();
                state.close_modal();
                state.set_iris_thinking("Generating commit message...");
                state.modes.commit.generating = true;
                return Action::IrisQuery(IrisQueryRequest::GenerateCommit {
                    instructions: if state.modes.commit.custom_instructions.is_empty() {
                        None
                    } else {
                        Some(state.modes.commit.custom_instructions.clone())
                    },
                    preset: preset_key,
                    use_gitmoji: state.modes.commit.emoji_mode != EmojiMode::None,
                });
            }
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Up => {
            if let Some(Modal::PresetSelector {
                selected, scroll, ..
            }) = &mut state.modal
            {
                *selected = selected.saturating_sub(1);
                // Scroll up if selection goes above visible area
                if *selected < *scroll {
                    *scroll = *selected;
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Down => {
            if let Some(Modal::PresetSelector {
                selected,
                scroll,
                presets,
                input,
            }) = &mut state.modal
            {
                let filtered_len = presets
                    .iter()
                    .filter(|p| {
                        input.is_empty()
                            || p.name.to_lowercase().contains(&input.to_lowercase())
                            || p.key.to_lowercase().contains(&input.to_lowercase())
                    })
                    .count();
                if *selected + 1 < filtered_len {
                    *selected += 1;
                    // Scroll down if selection goes below visible area
                    if *selected >= *scroll + VISIBLE_ITEMS {
                        *scroll = *selected - VISIBLE_ITEMS + 1;
                    }
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char(c) => {
            if let Some(Modal::PresetSelector {
                input,
                selected,
                scroll,
                ..
            }) = &mut state.modal
            {
                input.push(c);
                *selected = 0; // Reset selection on filter change
                *scroll = 0;
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Backspace => {
            if let Some(Modal::PresetSelector {
                input,
                selected,
                scroll,
                ..
            }) = &mut state.modal
            {
                input.pop();
                *selected = 0;
                *scroll = 0;
            }
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_ref_selector_modal(state: &mut StudioState, key: KeyEvent) -> Action {
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
        return Action::None;
    };

    // Filter refs based on input
    let filtered: Vec<_> = refs
        .iter()
        .filter(|r| input.is_empty() || r.to_lowercase().contains(&input.to_lowercase()))
        .collect();

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Apply selection and determine which data reload action is needed
            #[derive(Clone, Copy)]
            enum ReloadType {
                None,
                Pr,
                Review,
                Changelog,
                ReleaseNotes,
            }

            let reload_type = if let Some(selected_ref) = filtered.get(selected) {
                let (label, reload) = match target {
                    RefSelectorTarget::ReviewFrom => {
                        state.modes.review.from_ref.clone_from(selected_ref);
                        ("Review from", ReloadType::Review)
                    }
                    RefSelectorTarget::ReviewTo => {
                        state.modes.review.to_ref.clone_from(selected_ref);
                        ("Review to", ReloadType::Review)
                    }
                    RefSelectorTarget::PrFrom => {
                        state.modes.pr.base_branch.clone_from(selected_ref);
                        ("PR base", ReloadType::Pr)
                    }
                    RefSelectorTarget::PrTo => {
                        state.modes.pr.to_ref.clone_from(selected_ref);
                        ("PR target", ReloadType::Pr)
                    }
                    RefSelectorTarget::ChangelogFrom => {
                        state.modes.changelog.from_ref.clone_from(selected_ref);
                        ("Changelog from", ReloadType::Changelog)
                    }
                    RefSelectorTarget::ChangelogTo => {
                        state.modes.changelog.to_ref.clone_from(selected_ref);
                        ("Changelog to", ReloadType::Changelog)
                    }
                    RefSelectorTarget::ReleaseNotesFrom => {
                        state.modes.release_notes.from_ref.clone_from(selected_ref);
                        ("Release Notes from", ReloadType::ReleaseNotes)
                    }
                    RefSelectorTarget::ReleaseNotesTo => {
                        state.modes.release_notes.to_ref.clone_from(selected_ref);
                        ("Release Notes to", ReloadType::ReleaseNotes)
                    }
                };
                state.notify(Notification::info(format!("{label} set to {selected_ref}")));
                reload
            } else {
                ReloadType::None
            };
            state.close_modal();
            match reload_type {
                ReloadType::Pr => Action::ReloadPrData,
                ReloadType::Review => Action::ReloadReviewData,
                ReloadType::Changelog => Action::ReloadChangelogData,
                ReloadType::ReleaseNotes => Action::ReloadReleaseNotesData,
                ReloadType::None => Action::Redraw,
            }
        }
        KeyCode::Up => {
            if let Some(Modal::RefSelector { selected, .. }) = &mut state.modal {
                *selected = selected.saturating_sub(1);
            }
            state.mark_dirty();
            Action::Redraw
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
            Action::Redraw
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
            Action::Redraw
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
            Action::Redraw
        }
        _ => Action::None,
    }
}

fn handle_emoji_selector_modal(state: &mut StudioState, key: KeyEvent) -> Action {
    // Visible items in the list
    const VISIBLE_ITEMS: usize = 18;

    // Get current state for filtering
    let (input, emojis, selected) = if let Some(Modal::EmojiSelector {
        input,
        emojis,
        selected,
        ..
    }) = &state.modal
    {
        (input.clone(), emojis.clone(), *selected)
    } else {
        return Action::None;
    };

    // Filter emojis based on input
    let filtered: Vec<_> = emojis
        .iter()
        .filter(|e| {
            input.is_empty()
                || e.key.to_lowercase().contains(&input.to_lowercase())
                || e.description.to_lowercase().contains(&input.to_lowercase())
                || e.emoji.contains(&input)
        })
        .collect();

    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Enter => {
            // Apply selection
            if let Some(emoji_info) = filtered.get(selected) {
                let new_mode = match emoji_info.key.as_str() {
                    "none" => EmojiMode::None,
                    "auto" => EmojiMode::Auto,
                    _ => EmojiMode::Custom(emoji_info.emoji.clone()),
                };
                state.modes.commit.emoji_mode = new_mode.clone();
                // Sync legacy flag
                state.modes.commit.use_gitmoji = new_mode != EmojiMode::None;
                let status = match &new_mode {
                    EmojiMode::None => "off".to_string(),
                    EmojiMode::Auto => "auto".to_string(),
                    EmojiMode::Custom(e) => format!("{} ({})", e, emoji_info.key),
                };
                state.notify(Notification::info(format!("Emoji: {}", status)));
            }
            state.close_modal();
            Action::Redraw
        }
        KeyCode::Up => {
            if let Some(Modal::EmojiSelector {
                selected, scroll, ..
            }) = &mut state.modal
            {
                *selected = selected.saturating_sub(1);
                if *selected < *scroll {
                    *scroll = *selected;
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Down => {
            if let Some(Modal::EmojiSelector {
                selected,
                scroll,
                emojis,
                input,
            }) = &mut state.modal
            {
                let filtered_len = emojis
                    .iter()
                    .filter(|e| {
                        input.is_empty()
                            || e.key.to_lowercase().contains(&input.to_lowercase())
                            || e.description.to_lowercase().contains(&input.to_lowercase())
                            || e.emoji.contains(input.as_str())
                    })
                    .count();
                if *selected + 1 < filtered_len {
                    *selected += 1;
                    if *selected >= *scroll + VISIBLE_ITEMS {
                        *scroll = *selected - VISIBLE_ITEMS + 1;
                    }
                }
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Char(c) => {
            if let Some(Modal::EmojiSelector {
                input,
                selected,
                scroll,
                ..
            }) = &mut state.modal
            {
                input.push(c);
                *selected = 0;
                *scroll = 0;
            }
            state.mark_dirty();
            Action::Redraw
        }
        KeyCode::Backspace => {
            if let Some(Modal::EmojiSelector {
                input,
                selected,
                scroll,
                ..
            }) = &mut state.modal
            {
                input.pop();
                *selected = 0;
                *scroll = 0;
            }
            state.mark_dirty();
            Action::Redraw
        }
        _ => Action::None,
    }
}
