//! State management for Iris Studio
//!
//! Centralized state for all modes and shared data.

use crate::config::Config;
use crate::git::GitRepo;
use crate::types::GeneratedMessage;
use lru::LruCache;
use std::collections::{HashMap, VecDeque};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;

use super::components::{DiffViewState, FileTreeState, MessageEditorState};

// ═══════════════════════════════════════════════════════════════════════════════
// Mode Enum
// ═══════════════════════════════════════════════════════════════════════════════

/// Available modes in Iris Studio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Explore mode - semantic code understanding
    #[default]
    Explore,
    /// Commit mode - generate and edit commit messages
    Commit,
    /// Review mode - AI-powered code review (future)
    Review,
    /// PR mode - pull request creation (future)
    PR,
    /// Changelog mode - release documentation (future)
    Changelog,
}

impl Mode {
    /// Get the display name for this mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Mode::Explore => "Explore",
            Mode::Commit => "Commit",
            Mode::Review => "Review",
            Mode::PR => "PR",
            Mode::Changelog => "Changelog",
        }
    }

    /// Get the keyboard shortcut for this mode
    pub fn shortcut(&self) -> char {
        match self {
            Mode::Explore => 'E',
            Mode::Commit => 'C',
            Mode::Review => 'R',
            Mode::PR => 'P',
            Mode::Changelog => 'L',
        }
    }

    /// Check if this mode is available (implemented)
    pub fn is_available(&self) -> bool {
        matches!(self, Mode::Explore | Mode::Commit)
    }

    /// Get all modes in order
    pub fn all() -> &'static [Mode] {
        &[
            Mode::Explore,
            Mode::Commit,
            Mode::Review,
            Mode::PR,
            Mode::Changelog,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Panel Focus
// ═══════════════════════════════════════════════════════════════════════════════

/// Generic panel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    /// Left panel (typically file tree or file list)
    Left,
    /// Center panel (typically code view or diff view)
    Center,
    /// Right panel (typically context or message)
    Right,
}

impl PanelId {
    /// Get the next panel in tab order
    pub fn next(&self) -> Self {
        match self {
            PanelId::Left => PanelId::Center,
            PanelId::Center => PanelId::Right,
            PanelId::Right => PanelId::Left,
        }
    }

    /// Get the previous panel in tab order
    pub fn prev(&self) -> Self {
        match self {
            PanelId::Left => PanelId::Right,
            PanelId::Center => PanelId::Left,
            PanelId::Right => PanelId::Center,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Git Status
// ═══════════════════════════════════════════════════════════════════════════════

/// Cached git repository status
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    /// Number of staged files
    pub staged_count: usize,
    /// Number of modified (unstaged) files
    pub modified_count: usize,
    /// Number of untracked files
    pub untracked_count: usize,
    /// Number of commits ahead of upstream
    pub commits_ahead: usize,
    /// Number of commits behind upstream
    pub commits_behind: usize,
    /// List of staged files
    pub staged_files: Vec<PathBuf>,
    /// List of modified files
    pub modified_files: Vec<PathBuf>,
    /// List of untracked files
    pub untracked_files: Vec<PathBuf>,
}

impl GitStatus {
    /// Check if we're on the main/master branch
    pub fn is_main_branch(&self) -> bool {
        self.branch == "main" || self.branch == "master"
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.staged_count > 0 || self.modified_count > 0 || self.untracked_count > 0
    }

    /// Check if there are staged changes ready to commit
    pub fn has_staged(&self) -> bool {
        self.staged_count > 0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Notifications
// ═══════════════════════════════════════════════════════════════════════════════

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A notification message to display to the user
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: std::time::Instant,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Info,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Success,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Warning,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Error,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Check if this notification has expired (older than 5 seconds)
    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > std::time::Duration::from_secs(5)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Modal State
// ═══════════════════════════════════════════════════════════════════════════════

/// Active modal dialog
#[derive(Debug, Clone)]
pub enum Modal {
    /// Help overlay showing keybindings
    Help,
    /// Search modal for files/symbols
    Search { query: String, results: Vec<String> },
    /// Confirmation dialog
    Confirm { message: String, action: String },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Iris Status
// ═══════════════════════════════════════════════════════════════════════════════

/// Status of the Iris agent
#[derive(Debug, Clone, Default)]
pub enum IrisStatus {
    #[default]
    Idle,
    Thinking {
        task: String,
        spinner_frame: usize,
    },
    Error(String),
}

impl IrisStatus {
    /// Get the spinner frame character
    pub fn spinner_char(&self) -> Option<char> {
        match self {
            IrisStatus::Thinking { spinner_frame, .. } => {
                let frames = super::theme::SPINNER_BRAILLE;
                Some(frames[*spinner_frame % frames.len()])
            }
            _ => None,
        }
    }

    /// Advance the spinner frame
    pub fn tick(&mut self) {
        if let IrisStatus::Thinking { spinner_frame, .. } = self {
            *spinner_frame = (*spinner_frame + 1) % super::theme::SPINNER_BRAILLE.len();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mode-Specific States
// ═══════════════════════════════════════════════════════════════════════════════

/// State for Explore mode
#[derive(Default)]
pub struct ExploreState {
    /// Currently selected file
    pub current_file: Option<PathBuf>,
    /// Current line in code view
    pub current_line: usize,
    /// Selection range (start, end) for multi-line queries
    pub selection: Option<(usize, usize)>,
    /// Code view scroll offset
    pub code_scroll: usize,
    /// Heat map enabled
    pub show_heat_map: bool,
    /// File tree state
    pub file_tree: FileTreeState,
}

impl std::fmt::Debug for ExploreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExploreState")
            .field("current_file", &self.current_file)
            .field("current_line", &self.current_line)
            .field("selection", &self.selection)
            .field("code_scroll", &self.code_scroll)
            .field("show_heat_map", &self.show_heat_map)
            .finish()
    }
}

/// State for Commit mode
pub struct CommitState {
    /// Generated commit messages
    pub messages: Vec<GeneratedMessage>,
    /// Index of current message
    pub current_index: usize,
    /// Custom instructions for regeneration
    pub custom_instructions: String,
    /// Selected file in staged list
    pub selected_file_index: usize,
    /// Is message being edited
    pub editing_message: bool,
    /// Is generating new message
    pub generating: bool,
    /// Use gitmoji
    pub use_gitmoji: bool,
    /// File tree state for staged files
    pub file_tree: FileTreeState,
    /// Diff view state
    pub diff_view: DiffViewState,
    /// Message editor state
    pub message_editor: MessageEditorState,
}

impl Default for CommitState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            current_index: 0,
            custom_instructions: String::new(),
            selected_file_index: 0,
            editing_message: false,
            generating: false,
            use_gitmoji: true,
            file_tree: FileTreeState::new(),
            diff_view: DiffViewState::new(),
            message_editor: MessageEditorState::new(),
        }
    }
}

impl std::fmt::Debug for CommitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitState")
            .field("messages_count", &self.messages.len())
            .field("current_index", &self.current_index)
            .field("selected_file_index", &self.selected_file_index)
            .field("editing_message", &self.editing_message)
            .field("generating", &self.generating)
            .finish()
    }
}

/// State for Review mode (placeholder)
#[derive(Debug, Default)]
pub struct ReviewState {
    /// Current file being reviewed
    pub current_file: Option<PathBuf>,
    /// Current issue index
    pub current_issue: usize,
}

/// State for PR mode (placeholder)
#[derive(Debug, Default)]
pub struct PrState {
    /// Base branch for PR
    pub base_branch: String,
    /// Description hint from commit messages
    pub description_hint: Option<String>,
}

/// State for Changelog mode (placeholder)
#[derive(Debug, Default)]
pub struct ChangelogState {
    /// Version range start
    pub from_version: String,
    /// Version range end
    pub to_version: String,
}

/// Container for all mode states
#[derive(Debug, Default)]
pub struct ModeStates {
    pub explore: ExploreState,
    pub commit: CommitState,
    pub review: ReviewState,
    pub pr: PrState,
    pub changelog: ChangelogState,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Studio State
// ═══════════════════════════════════════════════════════════════════════════════

/// Cache location key for semantic blame
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Location {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

/// Main application state for Iris Studio
pub struct StudioState {
    /// Git repository reference
    pub repo: Option<Arc<GitRepo>>,

    /// Cached git status
    pub git_status: GitStatus,

    /// Application configuration
    pub config: Config,

    /// Current active mode
    pub active_mode: Mode,

    /// Focused panel
    pub focused_panel: PanelId,

    /// Mode-specific states
    pub modes: ModeStates,

    /// Semantic blame cache
    pub semantic_cache: LruCache<Location, String>,

    /// Diff cache (file path -> diff content)
    pub diff_cache: HashMap<PathBuf, String>,

    /// Active modal
    pub modal: Option<Modal>,

    /// Notification queue
    pub notifications: VecDeque<Notification>,

    /// Iris agent status
    pub iris_status: IrisStatus,

    /// Whether the UI needs redraw
    pub dirty: bool,

    /// Last render timestamp for animations
    pub last_render: std::time::Instant,
}

impl StudioState {
    /// Create new studio state
    pub fn new(config: Config, repo: Option<Arc<GitRepo>>) -> Self {
        Self {
            repo,
            git_status: GitStatus::default(),
            config,
            active_mode: Mode::Explore,
            focused_panel: PanelId::Left,
            modes: ModeStates::default(),
            semantic_cache: LruCache::new(NonZeroUsize::new(100).expect("non-zero")),
            diff_cache: HashMap::new(),
            modal: None,
            notifications: VecDeque::new(),
            iris_status: IrisStatus::Idle,
            dirty: true,
            last_render: std::time::Instant::now(),
        }
    }

    /// Suggest the best initial mode based on repo state
    pub fn suggest_initial_mode(&self) -> Mode {
        let status = &self.git_status;

        // Staged changes? Probably want to commit
        if status.has_staged() {
            return Mode::Commit;
        }

        // On feature branch with commits ahead? PR time (future)
        // if status.commits_ahead > 0 && !status.is_main_branch() {
        //     return Mode::PR;
        // }

        // Default to exploration
        Mode::Explore
    }

    /// Switch to a new mode with context preservation
    pub fn switch_mode(&mut self, new_mode: Mode) {
        if !new_mode.is_available() {
            self.notify(Notification::warning(format!(
                "{} mode is not yet implemented",
                new_mode.display_name()
            )));
            return;
        }

        let old_mode = self.active_mode;

        // Context preservation logic
        match (old_mode, new_mode) {
            (Mode::Explore, Mode::Commit) => {
                // Carry current file context to commit
            }
            (Mode::Commit, Mode::Explore) => {
                // Carry last viewed file back
            }
            _ => {}
        }

        self.active_mode = new_mode;
        self.focused_panel = PanelId::Left;
        self.dirty = true;
    }

    /// Add a notification
    pub fn notify(&mut self, notification: Notification) {
        self.notifications.push_back(notification);
        // Keep only last 5 notifications
        while self.notifications.len() > 5 {
            self.notifications.pop_front();
        }
        self.dirty = true;
    }

    /// Get the current notification (most recent non-expired)
    pub fn current_notification(&self) -> Option<&Notification> {
        self.notifications.iter().rev().find(|n| !n.is_expired())
    }

    /// Clean up expired notifications
    pub fn cleanup_notifications(&mut self) {
        let had_notifications = !self.notifications.is_empty();
        self.notifications.retain(|n| !n.is_expired());
        if had_notifications && self.notifications.is_empty() {
            self.dirty = true;
        }
    }

    /// Mark state as dirty (needs redraw)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check and clear dirty flag
    pub fn check_dirty(&mut self) -> bool {
        let was_dirty = self.dirty;
        self.dirty = false;
        was_dirty
    }

    /// Focus the next panel
    pub fn focus_next_panel(&mut self) {
        self.focused_panel = self.focused_panel.next();
        self.dirty = true;
    }

    /// Focus the previous panel
    pub fn focus_prev_panel(&mut self) {
        self.focused_panel = self.focused_panel.prev();
        self.dirty = true;
    }

    /// Open help modal
    pub fn show_help(&mut self) {
        self.modal = Some(Modal::Help);
        self.dirty = true;
    }

    /// Close any open modal
    pub fn close_modal(&mut self) {
        if self.modal.is_some() {
            self.modal = None;
            self.dirty = true;
        }
    }

    /// Update Iris status
    pub fn set_iris_thinking(&mut self, task: impl Into<String>) {
        self.iris_status = IrisStatus::Thinking {
            task: task.into(),
            spinner_frame: 0,
        };
        self.dirty = true;
    }

    /// Set Iris idle
    pub fn set_iris_idle(&mut self) {
        self.iris_status = IrisStatus::Idle;
        self.dirty = true;
    }

    /// Set Iris error
    pub fn set_iris_error(&mut self, error: impl Into<String>) {
        self.iris_status = IrisStatus::Error(error.into());
        self.dirty = true;
    }

    /// Tick animations (spinner, etc.)
    pub fn tick(&mut self) {
        self.iris_status.tick();
        self.cleanup_notifications();

        // Only mark dirty if we have active animations
        if matches!(self.iris_status, IrisStatus::Thinking { .. }) {
            self.dirty = true;
        }
    }
}
