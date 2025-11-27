//! Settings modal key handler

use crossterm::event::{KeyCode, KeyEvent};

use crate::studio::events::SideEffect;
use crate::studio::state::{Modal, SettingsField, StudioState};

/// Handle key events in settings modal
pub fn handle(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    // Check if we're editing a field
    let is_editing = if let Some(Modal::Settings(settings)) = &state.modal {
        settings.editing
    } else {
        return vec![];
    };

    if is_editing {
        handle_editing_mode(state, key)
    } else {
        handle_navigation_mode(state, key)
    }
}

fn handle_editing_mode(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        KeyCode::Esc => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.cancel_editing();
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Enter => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.confirm_editing();
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Char(c) => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.input_buffer.push(c);
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Backspace => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.input_buffer.pop();
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}

fn handle_navigation_mode(state: &mut StudioState, key: KeyEvent) -> Vec<SideEffect> {
    match key.code {
        KeyCode::Esc => {
            state.close_modal();
            vec![]
        }
        KeyCode::Char('s') => {
            // Save settings
            vec![SideEffect::SaveSettings]
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.select_prev();
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.select_next();
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.start_editing();
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Cycle backwards for cyclable fields
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                let field = settings.current_field();
                match field {
                    SettingsField::Provider => {
                        if let Some(idx) = settings
                            .available_providers
                            .iter()
                            .position(|p| p == &settings.provider)
                        {
                            let prev = if idx == 0 {
                                settings.available_providers.len() - 1
                            } else {
                                idx - 1
                            };
                            settings.provider = settings.available_providers[prev].clone();
                            settings.modified = true;
                        }
                    }
                    SettingsField::UseGitmoji => {
                        settings.use_gitmoji = !settings.use_gitmoji;
                        settings.modified = true;
                    }
                    SettingsField::InstructionPreset => {
                        if let Some(idx) = settings
                            .available_presets
                            .iter()
                            .position(|p| p == &settings.instruction_preset)
                        {
                            let prev = if idx == 0 {
                                settings.available_presets.len() - 1
                            } else {
                                idx - 1
                            };
                            settings.instruction_preset = settings.available_presets[prev].clone();
                            settings.modified = true;
                        }
                    }
                    _ => {}
                }
            }
            state.mark_dirty();
            vec![]
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(Modal::Settings(settings)) = &mut state.modal {
                settings.cycle_current_field();
            }
            state.mark_dirty();
            vec![]
        }
        _ => vec![],
    }
}
