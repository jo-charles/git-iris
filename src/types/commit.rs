//! Commit message types and formatting

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use textwrap;

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

/// Formats a commit message from a `GeneratedMessage`
pub fn format_commit_message(response: &GeneratedMessage) -> String {
    let mut message = String::new();

    if let Some(emoji) = &response.emoji {
        write!(&mut message, "{emoji} ").expect("write to string should not fail");
    }

    message.push_str(&response.title);
    message.push_str("\n\n");

    let wrapped_message = textwrap::wrap(&response.message, 78);
    for line in wrapped_message {
        message.push_str(&line);
        message.push('\n');
    }

    message
}
