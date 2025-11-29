//! Settings event handlers
//!
//! Handles preset, gitmoji, emoji, and amend mode settings.

use super::super::state::{EmojiMode, StudioState};

/// Handle `SetPreset` event
pub fn set_preset(state: &mut StudioState, preset: String) {
    state.modes.commit.preset = preset;
    state.mark_dirty();
}

/// Handle `ToggleGitmoji` event
pub fn toggle_gitmoji(state: &mut StudioState) {
    state.modes.commit.use_gitmoji = !state.modes.commit.use_gitmoji;
    state.modes.commit.emoji_mode = if state.modes.commit.use_gitmoji {
        EmojiMode::Auto
    } else {
        EmojiMode::None
    };
    state.mark_dirty();
}

/// Handle `SetEmoji` event
pub fn set_emoji(state: &mut StudioState, emoji: String) {
    state.modes.commit.emoji_mode = if emoji.is_empty() {
        EmojiMode::None
    } else {
        EmojiMode::Custom(emoji)
    };
    state.mark_dirty();
}

/// Handle `ToggleAmendMode` event
pub fn toggle_amend_mode(state: &mut StudioState) {
    state.modes.commit.amend_mode = !state.modes.commit.amend_mode;
    if state.modes.commit.amend_mode {
        // Load original message from HEAD if repo is available
        if let Some(repo) = &state.repo
            && let Ok(msg) = repo.get_head_commit_message()
        {
            state.modes.commit.original_message = Some(msg);
        }
    } else {
        state.modes.commit.original_message = None;
    }
    // Clear messages when toggling amend mode
    state.modes.commit.messages.clear();
    state.modes.commit.message_editor.clear();
    state.mark_dirty();
}
