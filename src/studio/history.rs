//! History System for Iris Studio
//!
//! Single source of truth for all content changes, chat messages, and events.
//! Provides:
//! - Complete audit trail across all modes
//! - Content timeline per (mode, `content_type`)
//! - Chat history accessible from anywhere
//! - Event replay for debugging/undo
//! - Persistence metadata for future thread resume UI

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::GeneratedMessage;

use super::events::{ContentType, EventSource, TaskType, TimestampedEvent};
use super::state::Mode;

// ═══════════════════════════════════════════════════════════════════════════════
// Session Metadata (for persistence)
// ═══════════════════════════════════════════════════════════════════════════════

/// Metadata for history persistence and thread identification
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// Unique session/thread identifier
    pub session_id: Uuid,
    /// When this session was created (wall clock time for persistence)
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Repository path this session belongs to
    pub repo_path: Option<PathBuf>,
    /// Optional title/summary for the thread (e.g., "feat: add auth")
    pub title: Option<String>,
    /// Current branch at session start
    pub branch: Option<String>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionMetadata {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            created_at: now,
            last_activity: now,
            repo_path: None,
            title: None,
            branch: None,
        }
    }

    /// Create with repo context
    pub fn with_repo(repo_path: PathBuf, branch: Option<String>) -> Self {
        let mut meta = Self::new();
        meta.repo_path = Some(repo_path);
        meta.branch = branch;
        meta
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Set a title for this session
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// History
// ═══════════════════════════════════════════════════════════════════════════════

/// Single source of truth for all history in Iris Studio
#[derive(Debug, Clone)]
pub struct History {
    /// Session metadata for persistence and thread resume
    pub metadata: SessionMetadata,

    /// Complete event log (all events that modified state)
    events: VecDeque<HistoryEntry>,

    /// Maximum events to retain (prevents unbounded growth)
    pub(crate) max_events: usize,

    /// Chat messages (persists across modes)
    chat_messages: Vec<ChatMessage>,

    /// Content versions indexed by (mode, `content_type`)
    /// Each entry contains all versions of that content
    content_versions: HashMap<ContentKey, Vec<ContentVersion>>,

    /// Generation counter for unique IDs
    next_id: u64,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// Create a new unified history
    pub fn new() -> Self {
        Self {
            metadata: SessionMetadata::new(),
            events: VecDeque::with_capacity(1000),
            max_events: 1000,
            chat_messages: Vec::new(),
            content_versions: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create with repository context (for persistence)
    pub fn with_repo(repo_path: PathBuf, branch: Option<String>) -> Self {
        Self {
            metadata: SessionMetadata::with_repo(repo_path, branch),
            events: VecDeque::with_capacity(1000),
            max_events: 1000,
            chat_messages: Vec::new(),
            content_versions: HashMap::new(),
            next_id: 1,
        }
    }

    /// Get session ID for this history
    pub fn session_id(&self) -> Uuid {
        self.metadata.session_id
    }

    /// Set title for this session (e.g., from first commit message)
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.metadata.set_title(title);
    }

    /// Touch metadata to update `last_activity`
    fn touch(&mut self) {
        self.metadata.touch();
    }

    /// Record an event in history
    pub fn record_event(&mut self, event: &TimestampedEvent) {
        self.touch();
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: event.timestamp,
            source: event.source,
            change: HistoryChange::Event(format!("{:?}", event.event)),
        };

        self.push_entry(entry);
    }

    /// Record a content update
    pub fn record_content(
        &mut self,
        mode: Mode,
        content_type: ContentType,
        content: &ContentData,
        source: EventSource,
        trigger: &str,
    ) {
        self.touch();
        let key = ContentKey { mode, content_type };

        // Get previous content for diff tracking
        let previous = self
            .content_versions
            .get(&key)
            .and_then(|versions| versions.last())
            .map(|v| v.content.clone());

        let version = ContentVersion {
            id: self.next_id(),
            timestamp: Instant::now(),
            source,
            trigger: trigger.to_string(),
            content: content.clone(),
            previous_id: self
                .content_versions
                .get(&key)
                .and_then(|v| v.last())
                .map(|v| v.id),
        };

        // Record in content versions
        self.content_versions
            .entry(key.clone())
            .or_default()
            .push(version);

        // Record in event log
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: Instant::now(),
            source,
            change: HistoryChange::ContentUpdated {
                mode,
                content_type,
                trigger: trigger.to_string(),
                preview: content.preview(50),
                previous_preview: previous.map(|p| p.preview(50)),
            },
        };

        self.push_entry(entry);
    }

    /// Add a chat message
    pub fn add_chat_message(&mut self, role: ChatRole, content: String) {
        self.touch();
        let message = ChatMessage {
            id: self.next_id(),
            timestamp: Instant::now(),
            role,
            content: content.clone(),
            mode_context: None,
        };

        self.chat_messages.push(message);

        // Also record in event log
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: Instant::now(),
            source: match role {
                ChatRole::User => EventSource::User,
                ChatRole::Iris => EventSource::Agent,
            },
            change: HistoryChange::ChatMessage {
                role,
                preview: if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content
                },
            },
        };

        self.push_entry(entry);
    }

    /// Add a chat message with mode context
    pub fn add_chat_message_with_context(
        &mut self,
        role: ChatRole,
        content: String,
        mode: Mode,
        related_content: Option<String>,
    ) {
        self.touch();
        let message = ChatMessage {
            id: self.next_id(),
            timestamp: Instant::now(),
            role,
            content: content.clone(),
            mode_context: Some(ModeContext {
                mode,
                related_content,
            }),
        };

        self.chat_messages.push(message);

        // Also record in event log
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: Instant::now(),
            source: match role {
                ChatRole::User => EventSource::User,
                ChatRole::Iris => EventSource::Agent,
            },
            change: HistoryChange::ChatMessage {
                role,
                preview: if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content
                },
            },
        };

        self.push_entry(entry);
    }

    /// Record a mode switch
    pub fn record_mode_switch(&mut self, from: Mode, to: Mode) {
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: Instant::now(),
            source: EventSource::User,
            change: HistoryChange::ModeSwitch { from, to },
        };

        self.push_entry(entry);
    }

    /// Record an agent task start
    pub fn record_agent_start(&mut self, task_type: TaskType) {
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: Instant::now(),
            source: EventSource::System,
            change: HistoryChange::AgentTaskStarted { task_type },
        };

        self.push_entry(entry);
    }

    /// Record an agent task completion
    pub fn record_agent_complete(&mut self, task_type: TaskType, success: bool) {
        let entry = HistoryEntry {
            id: self.next_id(),
            timestamp: Instant::now(),
            source: EventSource::Agent,
            change: HistoryChange::AgentTaskCompleted { task_type, success },
        };

        self.push_entry(entry);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Query Methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Get all chat messages
    pub fn chat_messages(&self) -> &[ChatMessage] {
        &self.chat_messages
    }

    /// Get recent chat messages (last n)
    pub fn recent_chat_messages(&self, n: usize) -> &[ChatMessage] {
        let start = self.chat_messages.len().saturating_sub(n);
        &self.chat_messages[start..]
    }

    /// Get content versions for a specific (mode, `content_type`)
    pub fn content_versions(&self, mode: Mode, content_type: ContentType) -> &[ContentVersion] {
        let key = ContentKey { mode, content_type };
        self.content_versions.get(&key).map_or(&[], Vec::as_slice)
    }

    /// Get the latest content version
    pub fn latest_content(&self, mode: Mode, content_type: ContentType) -> Option<&ContentVersion> {
        let key = ContentKey { mode, content_type };
        self.content_versions.get(&key).and_then(|v| v.last())
    }

    /// Get content version count
    pub fn content_version_count(&self, mode: Mode, content_type: ContentType) -> usize {
        let key = ContentKey { mode, content_type };
        self.content_versions.get(&key).map_or(0, Vec::len)
    }

    /// Get all events (for debugging/audit)
    pub fn events(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.events.iter()
    }

    /// Get recent events (last n)
    pub fn recent_events(&self, n: usize) -> impl Iterator<Item = &HistoryEntry> {
        let skip = self.events.len().saturating_sub(n);
        self.events.iter().skip(skip)
    }

    /// Get events since a timestamp
    pub fn events_since(&self, since: Instant) -> impl Iterator<Item = &HistoryEntry> {
        self.events.iter().filter(move |e| e.timestamp >= since)
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Clear all history (for reset)
    pub fn clear(&mut self) {
        self.events.clear();
        self.chat_messages.clear();
        self.content_versions.clear();
    }

    /// Clear chat messages only
    pub fn clear_chat(&mut self) {
        self.chat_messages.clear();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn push_entry(&mut self, entry: HistoryEntry) {
        self.events.push_back(entry);

        // Trim if over capacity
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// History Entry Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A single entry in the history log
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Unique ID for this entry
    pub id: u64,
    /// When this happened
    pub timestamp: Instant,
    /// Where it came from
    pub source: EventSource,
    /// What changed
    pub change: HistoryChange,
}

/// What changed in this history entry
#[derive(Debug, Clone)]
pub enum HistoryChange {
    /// Generic event (for debugging)
    Event(String),

    /// Content was updated
    ContentUpdated {
        mode: Mode,
        content_type: ContentType,
        trigger: String,
        preview: String,
        previous_preview: Option<String>,
    },

    /// Chat message added
    ChatMessage { role: ChatRole, preview: String },

    /// Mode was switched
    ModeSwitch { from: Mode, to: Mode },

    /// Agent task started
    AgentTaskStarted { task_type: TaskType },

    /// Agent task completed
    AgentTaskCompleted { task_type: TaskType, success: bool },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Key for content version lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ContentKey {
    mode: Mode,
    content_type: ContentType,
}

/// A version of content
#[derive(Debug, Clone)]
pub struct ContentVersion {
    /// Unique ID
    pub id: u64,
    /// When created
    pub timestamp: Instant,
    /// Where it came from
    pub source: EventSource,
    /// What triggered this update
    pub trigger: String,
    /// The actual content
    pub content: ContentData,
    /// Previous version ID (for linking)
    pub previous_id: Option<u64>,
}

/// Content data - either structured (commit) or markdown
#[derive(Debug, Clone)]
pub enum ContentData {
    /// Structured commit message
    Commit(GeneratedMessage),

    /// Markdown content (PR, review, changelog, etc.)
    Markdown(String),
}

impl ContentData {
    /// Get a preview of the content
    pub fn preview(&self, max_len: usize) -> String {
        match self {
            Self::Commit(msg) => {
                let full = format!("{} {}", msg.emoji.as_deref().unwrap_or(""), msg.title);
                if full.len() > max_len {
                    format!("{}...", &full[..max_len])
                } else {
                    full
                }
            }
            Self::Markdown(content) => {
                // Get first non-empty line
                let first_line = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("");

                if first_line.len() > max_len {
                    format!("{}...", &first_line[..max_len])
                } else {
                    first_line.to_string()
                }
            }
        }
    }

    /// Get full content as string
    pub fn as_string(&self) -> String {
        match self {
            Self::Commit(msg) => {
                let emoji = msg.emoji.as_deref().unwrap_or("");
                if emoji.is_empty() {
                    format!("{}\n\n{}", msg.title, msg.message)
                } else {
                    format!("{} {}\n\n{}", emoji, msg.title, msg.message)
                }
            }
            Self::Markdown(content) => content.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Chat Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Unique ID
    pub id: u64,
    /// When sent
    pub timestamp: Instant,
    /// Who sent it
    pub role: ChatRole,
    /// The message content
    pub content: String,
    /// Optional mode context (what was being worked on)
    pub mode_context: Option<ModeContext>,
}

/// Who sent the message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Iris,
}

/// Context about what mode/content the message relates to
#[derive(Debug, Clone)]
pub struct ModeContext {
    /// Mode when message was sent
    pub mode: Mode,
    /// Related content (e.g., commit message being discussed)
    pub related_content: Option<String>,
}
