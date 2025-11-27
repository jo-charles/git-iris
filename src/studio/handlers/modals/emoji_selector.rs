//! Emoji selector modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{EmojiMode, Modal, Notification, StudioState};

/// Visible items in the list
const VISIBLE_ITEMS: usize = 18;

/// Handle key events in emoji selector modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
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
        return vec![];
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
            vec![]
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
            vec![]
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
            vec![]
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
            vec![]
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
            vec![]
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
            vec![]
        }
        _ => vec![],
    }
}
