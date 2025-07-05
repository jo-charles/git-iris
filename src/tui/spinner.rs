use crate::messages::ColoredMessage;
use crate::messages::get_waiting_message;
use ratatui::style::Color;
use unicode_width::UnicodeWidthStr;

pub struct SpinnerState {
    frames: Vec<char>,
    current_frame: usize,
    message: ColoredMessage,
    use_agent_status: bool,
}

impl SpinnerState {
    pub fn new() -> Self {
        // Automatically detect if agent mode is enabled
        let use_agent_status = crate::agents::status::is_agent_mode_enabled();

        if use_agent_status {
            Self {
                frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
                current_frame: 0,
                message: get_waiting_message(),
                use_agent_status: true,
            }
        } else {
            Self {
                frames: vec!['✦', '✧', '✶', '✷', '✸', '✹', '✺', '✻', '✼', '✽'],
                current_frame: 0,
                message: get_waiting_message(),
                use_agent_status: false,
            }
        }
    }

    pub fn new_with_agent() -> Self {
        Self {
            frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current_frame: 0,
            message: get_waiting_message(),
            use_agent_status: true,
        }
    }

    pub fn tick(&mut self) -> (String, String, Color, usize) {
        let frame = self.frames[self.current_frame];
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        let spinner_with_space = format!("{frame} ");

        // Update message from Iris status if in agent mode
        if self.use_agent_status {
            self.message = crate::agents::status::IRIS_STATUS.get_for_spinner();
        }

        let width = spinner_with_space.width() + self.message.text.width();
        (
            spinner_with_space,
            self.message.text.clone(),
            self.message.color,
            width,
        )
    }
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self::new()
    }
}
