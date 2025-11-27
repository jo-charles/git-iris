//! Content update tools for Iris Studio chat
//!
//! These tools allow Iris to update commit messages, PR descriptions, and reviews
//! through proper tool calls rather than JSON parsing.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::common::parameters_schema;

// Use standard tool error macro for consistency
crate::define_tool_error!(ContentUpdateError);

/// The types of content updates that can be sent to the Studio
#[derive(Debug, Clone)]
pub enum ContentUpdate {
    /// Update the commit message
    Commit {
        emoji: Option<String>,
        title: String,
        message: String,
    },
    /// Update the PR description
    PR { content: String },
    /// Update the code review
    Review { content: String },
}

/// Sender for content updates - passed to tools and listened to by Studio
pub type ContentUpdateSender = mpsc::UnboundedSender<ContentUpdate>;
pub type ContentUpdateReceiver = mpsc::UnboundedReceiver<ContentUpdate>;

/// Create a new content update channel
pub fn create_content_update_channel() -> (ContentUpdateSender, ContentUpdateReceiver) {
    mpsc::unbounded_channel()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Update Commit Tool
// ═══════════════════════════════════════════════════════════════════════════════

/// Tool for updating commit messages
#[derive(Clone)]
pub struct UpdateCommitTool {
    sender: Arc<ContentUpdateSender>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateCommitArgs {
    /// The emoji/gitmoji for the commit (e.g., "✨", "🐛", "♻️")
    #[serde(default)]
    pub emoji: Option<String>,
    /// The commit title (first line, should be concise)
    pub title: String,
    /// The commit body/message (detailed description)
    #[serde(default)]
    pub message: Option<String>,
}

impl UpdateCommitTool {
    pub fn new(sender: ContentUpdateSender) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl Tool for UpdateCommitTool {
    const NAME: &'static str = "update_commit";
    type Error = ContentUpdateError;
    type Args = UpdateCommitArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "update_commit".to_string(),
            description: "Update the current commit message. Use this when the user asks you to modify, change, or rewrite the commit message. The update will be applied immediately.".to_string(),
            parameters: parameters_schema::<UpdateCommitArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        tracing::info!(
            "update_commit tool called! emoji={:?}, title={}, message_len={}",
            args.emoji,
            args.title,
            args.message.as_ref().map_or(0, std::string::String::len)
        );

        let update = ContentUpdate::Commit {
            emoji: args.emoji.clone(),
            title: args.title.clone(),
            message: args.message.clone().unwrap_or_default(),
        };

        self.sender.send(update).map_err(|e| {
            tracing::error!("Failed to send content update: {}", e);
            ContentUpdateError(format!("Failed to send update: {}", e))
        })?;

        tracing::info!("Content update sent successfully via channel");

        let result = json!({
            "success": true,
            "message": "Commit message updated successfully",
            "new_title": args.title,
            "emoji": args.emoji
        });

        serde_json::to_string_pretty(&result).map_err(|e| ContentUpdateError(e.to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Update PR Tool
// ═══════════════════════════════════════════════════════════════════════════════

/// Tool for updating PR descriptions
#[derive(Clone)]
pub struct UpdatePRTool {
    sender: Arc<ContentUpdateSender>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdatePRArgs {
    /// The complete PR description content (markdown)
    pub content: String,
}

impl UpdatePRTool {
    pub fn new(sender: ContentUpdateSender) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl Tool for UpdatePRTool {
    const NAME: &'static str = "update_pr";
    type Error = ContentUpdateError;
    type Args = UpdatePRArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "update_pr".to_string(),
            description: "Update the current PR description. Use this when the user asks you to modify, change, or rewrite the PR content.".to_string(),
            parameters: parameters_schema::<UpdatePRArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let content_len = args.content.len();
        let update = ContentUpdate::PR {
            content: args.content,
        };

        self.sender
            .send(update)
            .map_err(|e| ContentUpdateError(format!("Failed to send update: {}", e)))?;

        let result = json!({
            "success": true,
            "message": "PR description updated successfully",
            "content_length": content_len
        });

        serde_json::to_string_pretty(&result).map_err(|e| ContentUpdateError(e.to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Update Review Tool
// ═══════════════════════════════════════════════════════════════════════════════

/// Tool for updating code reviews
#[derive(Clone)]
pub struct UpdateReviewTool {
    sender: Arc<ContentUpdateSender>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateReviewArgs {
    /// The complete review content (markdown)
    pub content: String,
}

impl UpdateReviewTool {
    pub fn new(sender: ContentUpdateSender) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl Tool for UpdateReviewTool {
    const NAME: &'static str = "update_review";
    type Error = ContentUpdateError;
    type Args = UpdateReviewArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "update_review".to_string(),
            description: "Update the current code review. Use this when the user asks you to modify, change, or rewrite the review content.".to_string(),
            parameters: parameters_schema::<UpdateReviewArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let content_len = args.content.len();
        let update = ContentUpdate::Review {
            content: args.content,
        };

        self.sender
            .send(update)
            .map_err(|e| ContentUpdateError(format!("Failed to send update: {}", e)))?;

        let result = json!({
            "success": true,
            "message": "Review updated successfully",
            "content_length": content_len
        });

        serde_json::to_string_pretty(&result).map_err(|e| ContentUpdateError(e.to_string()))
    }
}
