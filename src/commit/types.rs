use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use textwrap::wrap;

/// Model for commit message generation results
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct GeneratedMessage {
    /// Optional emoji for the commit message
    pub emoji: Option<String>,
    /// Commit message title/subject line
    pub title: String,
    /// Detailed commit message body
    pub message: String,
}

impl From<String> for GeneratedMessage {
    fn from(s: String) -> Self {
        match serde_json::from_str(&s) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse JSON: {e}\nInput was: {s}");
                Self {
                    emoji: None,
                    title: "Error parsing commit message".to_string(),
                    message: "There was an error parsing the commit message from the AI. Please try again.".to_string(),
                }
            }
        }
    }
}

/// Formats a commit message from a `GeneratedMessage`
pub fn format_commit_message(response: &GeneratedMessage) -> String {
    let mut message = String::new();

    if let Some(emoji) = &response.emoji {
        message.push_str(&format!("{emoji} "));
    }

    message.push_str(&response.title);
    message.push_str("\n\n");

    let wrapped_message = wrap(&response.message, 78);
    for line in wrapped_message {
        message.push_str(&line);
        message.push('\n');
    }

    message
}
