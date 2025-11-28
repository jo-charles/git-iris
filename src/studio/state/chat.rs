//! Chat state management for Iris Studio
//!
//! Contains the chat interface state, message history, and related types.

use std::collections::VecDeque;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum chat messages retained (older messages are dropped)
const MAX_CHAT_MESSAGES: usize = 500;

/// Maximum tool history entries per response
const MAX_TOOL_HISTORY: usize = 20;

// ═══════════════════════════════════════════════════════════════════════════════
// Chat Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Role in a chat conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Iris,
}

/// A single message in the chat
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    pub fn iris(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Iris,
            content: content.into(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Chat State
// ═══════════════════════════════════════════════════════════════════════════════

/// State for the chat interface
#[derive(Debug, Clone)]
pub struct ChatState {
    /// Conversation history (bounded, oldest messages dropped when full)
    pub messages: VecDeque<ChatMessage>,
    /// Current input text
    pub input: String,
    /// Scroll offset for message display
    pub scroll_offset: usize,
    /// Whether Iris is currently responding
    pub is_responding: bool,
    /// Streaming response (while generating)
    pub streaming_response: Option<String>,
    /// Whether to auto-scroll to bottom on new content
    pub auto_scroll: bool,
    /// Current tool being executed (shown with spinner)
    pub current_tool: Option<String>,
    /// History of tools called during this response (bounded)
    pub tool_history: VecDeque<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            input: String::new(),
            scroll_offset: 0,
            is_responding: false,
            streaming_response: None,
            auto_scroll: true,
            current_tool: None,
            tool_history: VecDeque::new(),
        }
    }
}

impl ChatState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create chat with initial context showing current content
    pub fn with_context(mode_name: &str, current_content: Option<&str>) -> Self {
        let mut state = Self::default();

        // Add an initial Iris message with context
        let context_msg = if let Some(content) = current_content {
            let preview = truncate_preview(content, 200);
            format!(
                "I'm ready to help with your {}. Here's what we have so far:\n\n```\n{}\n```\n\nWhat would you like to change?",
                mode_name, preview
            )
        } else {
            format!(
                "I'm ready to help with your {}. What would you like to do?",
                mode_name
            )
        };

        state.messages.push_back(ChatMessage::iris(context_msg));
        state
    }

    /// Trim messages to stay within bounds (drops oldest messages)
    fn trim_messages(&mut self) {
        while self.messages.len() > MAX_CHAT_MESSAGES {
            self.messages.pop_front();
        }
    }

    /// Add a user message and auto-scroll to bottom
    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push_back(ChatMessage::user(content));
        self.trim_messages();
        self.input.clear();
        self.auto_scroll = true; // Re-enable auto-scroll on new messages
    }

    /// Add or update Iris response and auto-scroll to bottom
    pub fn add_iris_response(&mut self, content: &str) {
        self.messages.push_back(ChatMessage::iris(content));
        self.trim_messages();
        self.is_responding = false;
        self.streaming_response = None;
        self.current_tool = None;
        self.tool_history.clear();
        self.auto_scroll = true; // Re-enable auto-scroll on new messages
    }

    /// Manually scroll up (disables auto-scroll)
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.auto_scroll = false; // User manually scrolled, disable auto-scroll
    }

    /// Manually scroll down
    pub fn scroll_down(&mut self, amount: usize, max_scroll: usize) {
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
        // If scrolled to bottom, re-enable auto-scroll
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    /// Estimate max scroll based on message content (~3 lines per message + content lines)
    pub fn estimated_max_scroll(&self) -> usize {
        let mut total_lines = 0;
        for msg in &self.messages {
            total_lines += 2; // Role header + separator
            total_lines += msg.content.lines().count().max(1);
        }
        if let Some(ref streaming) = self.streaming_response {
            total_lines += 2 + streaming.lines().count().max(1);
        }
        total_lines.saturating_sub(10) // Assume ~10 visible lines
    }

    /// Add tool to history (bounded, drops oldest when full)
    pub fn add_tool_to_history(&mut self, tool: String) {
        self.tool_history.push_back(tool);
        while self.tool_history.len() > MAX_TOOL_HISTORY {
            self.tool_history.pop_front();
        }
    }

    /// Clear the chat history
    pub fn clear(&mut self) {
        self.messages.clear();
        self.input.clear();
        self.scroll_offset = 0;
        self.is_responding = false;
        self.streaming_response = None;
        self.current_tool = None;
        self.tool_history.clear();
        self.auto_scroll = true;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Safely truncate a string to `max_chars`, adding "..." if truncated.
/// Handles UTF-8 correctly (no panic on multi-byte chars).
pub fn truncate_preview(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    }
}
