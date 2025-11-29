//! UI event handlers
//!
//! Handles notifications, scrolling, edit mode, message variants, clipboard.

use super::super::events::{NotificationLevel, SideEffect};
use super::super::state::{Mode, Notification, StudioState};
use super::navigation;

/// Handle Notify event
pub fn notify(state: &mut StudioState, level: NotificationLevel, message: String) {
    let notification = match level {
        NotificationLevel::Info => Notification::info(message),
        NotificationLevel::Success => Notification::success(message),
        NotificationLevel::Warning => Notification::warning(message),
        NotificationLevel::Error => Notification::error(message),
    };
    state.notify(notification);
}

/// Handle Scroll event
pub fn scroll(
    state: &mut StudioState,
    direction: super::super::events::ScrollDirection,
    amount: usize,
) {
    navigation::apply_scroll(state, direction, amount);
}

/// Handle `ToggleEditMode` event
pub fn toggle_edit_mode(state: &mut StudioState) {
    if state.active_mode == Mode::Commit {
        state.modes.commit.editing_message = !state.modes.commit.editing_message;
        if state.modes.commit.editing_message {
            state.modes.commit.message_editor.enter_edit_mode();
        } else {
            state.modes.commit.message_editor.exit_edit_mode();
        }
    }
    state.mark_dirty();
}

/// Handle `NextMessageVariant` event
pub fn next_message_variant(state: &mut StudioState) {
    let commit = &mut state.modes.commit;
    if !commit.messages.is_empty() {
        commit.current_index = (commit.current_index + 1) % commit.messages.len();
        // Use the editor's built-in navigation which syncs everything
        commit.message_editor.next_message();
    }
    state.mark_dirty();
}

/// Handle `PrevMessageVariant` event
pub fn prev_message_variant(state: &mut StudioState) {
    let commit = &mut state.modes.commit;
    if !commit.messages.is_empty() {
        commit.current_index = if commit.current_index == 0 {
            commit.messages.len() - 1
        } else {
            commit.current_index - 1
        };
        // Use the editor's built-in navigation which syncs everything
        commit.message_editor.prev_message();
    }
    state.mark_dirty();
}

/// Handle `CopyToClipboard` event
pub fn copy_to_clipboard(content: String) -> SideEffect {
    SideEffect::CopyToClipboard(content)
}
