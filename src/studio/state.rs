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

use super::components::{CodeViewState, DiffViewState, FileTreeState, MessageEditorState};

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
        matches!(
            self,
            Mode::Explore | Mode::Commit | Mode::Review | Mode::PR | Mode::Changelog
        )
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
// Chat State
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

/// State for the chat interface
#[derive(Debug, Clone, Default)]
pub struct ChatState {
    /// Conversation history
    pub messages: Vec<ChatMessage>,
    /// Current input text
    pub input: String,
    /// Scroll offset for message display
    pub scroll_offset: usize,
    /// Whether Iris is currently responding
    pub is_responding: bool,
}

impl ChatState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(ChatMessage::user(content));
        self.input.clear();
    }

    /// Add or update Iris response
    pub fn add_iris_response(&mut self, content: &str) {
        self.messages.push(ChatMessage::iris(content));
        self.is_responding = false;
    }

    /// Clear the chat history
    pub fn clear(&mut self) {
        self.messages.clear();
        self.input.clear();
        self.scroll_offset = 0;
        self.is_responding = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Modal State
// ═══════════════════════════════════════════════════════════════════════════════

/// Preset info for display
#[derive(Debug, Clone)]
pub struct PresetInfo {
    /// Preset key (id)
    pub key: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Emoji
    pub emoji: String,
}

/// Active modal dialog
pub enum Modal {
    /// Help overlay showing keybindings
    Help,
    /// Search modal for files/symbols
    Search { query: String, results: Vec<String> },
    /// Confirmation dialog
    Confirm { message: String, action: String },
    /// Instructions input for commit message generation
    Instructions { input: String },
    /// Chat interface with Iris
    Chat(ChatState),
    /// Base branch/ref selector for PR/changelog modes
    RefSelector {
        /// Current input/filter
        input: String,
        /// Available refs (branches, tags)
        refs: Vec<String>,
        /// Selected index
        selected: usize,
        /// Target mode (which mode to update)
        target: RefSelectorTarget,
    },
    /// Preset selector for commit style
    PresetSelector {
        /// Current input/filter
        input: String,
        /// Available presets
        presets: Vec<PresetInfo>,
        /// Selected index
        selected: usize,
        /// Scroll offset for long lists
        scroll: usize,
    },
}

/// Target for ref selector modal
#[derive(Debug, Clone, Copy)]
pub enum RefSelectorTarget {
    /// Review from ref
    ReviewFrom,
    /// Review to ref
    ReviewTo,
    /// PR from ref (base branch)
    PrFrom,
    /// PR to ref
    PrTo,
    /// Changelog from version
    ChangelogFrom,
    /// Changelog to version
    ChangelogTo,
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
    /// Code view state
    pub code_view: CodeViewState,
}

impl std::fmt::Debug for ExploreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExploreState")
            .field("current_file", &self.current_file)
            .field("current_line", &self.current_line)
            .field("selection", &self.selection)
            .field("code_scroll", &self.code_scroll)
            .field("show_heat_map", &self.show_heat_map)
            .finish_non_exhaustive()
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
    /// Current preset name
    pub preset: String,
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
            preset: "default".to_string(),
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
            .finish_non_exhaustive()
    }
}

/// State for Review mode
#[derive(Default)]
pub struct ReviewState {
    /// File tree for changed files
    pub file_tree: FileTreeState,
    /// Diff view for selected file
    pub diff_view: DiffViewState,
    /// Generated review content (markdown)
    pub review_content: String,
    /// Review scroll offset
    pub review_scroll: usize,
    /// Whether a review is being generated
    pub generating: bool,
    /// From ref for comparison
    pub from_ref: String,
    /// To ref for comparison (defaults to HEAD)
    pub to_ref: String,
}

impl std::fmt::Debug for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReviewState")
            .field("review_content_len", &self.review_content.len())
            .field("review_scroll", &self.review_scroll)
            .field("generating", &self.generating)
            .finish_non_exhaustive()
    }
}

/// A commit entry for PR mode
#[derive(Debug, Clone)]
pub struct PrCommit {
    /// Short commit hash
    pub hash: String,
    /// Commit message (first line)
    pub message: String,
    /// Author name
    pub author: String,
}

/// State for PR mode
pub struct PrState {
    /// Base branch for PR comparison (from ref)
    pub base_branch: String,
    /// Target ref (defaults to HEAD)
    pub to_ref: String,
    /// Commits in this PR (from base to HEAD)
    pub commits: Vec<PrCommit>,
    /// Selected commit index
    pub selected_commit: usize,
    /// Commit list scroll offset
    pub commit_scroll: usize,
    /// File tree for changed files
    pub file_tree: FileTreeState,
    /// Diff view state
    pub diff_view: DiffViewState,
    /// Generated PR description (markdown)
    pub pr_content: String,
    /// PR content scroll offset
    pub pr_scroll: usize,
    /// Whether PR description is being generated
    pub generating: bool,
}

impl Default for PrState {
    fn default() -> Self {
        Self {
            base_branch: "main".to_string(),
            to_ref: "HEAD".to_string(),
            commits: Vec::new(),
            selected_commit: 0,
            commit_scroll: 0,
            file_tree: FileTreeState::new(),
            diff_view: DiffViewState::new(),
            pr_content: String::new(),
            pr_scroll: 0,
            generating: false,
        }
    }
}

impl std::fmt::Debug for PrState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrState")
            .field("base_branch", &self.base_branch)
            .field("commits_count", &self.commits.len())
            .field("selected_commit", &self.selected_commit)
            .field("pr_content_len", &self.pr_content.len())
            .field("generating", &self.generating)
            .finish_non_exhaustive()
    }
}

/// A commit for display in changelog mode
#[derive(Debug, Clone)]
pub struct ChangelogCommit {
    /// Short commit hash
    pub hash: String,
    /// Commit message (first line)
    pub message: String,
    /// Author name
    pub author: String,
}

/// State for Changelog mode
#[derive(Debug)]
pub struct ChangelogState {
    /// Version range start (from ref)
    pub from_ref: String,
    /// Version range end (to ref)
    pub to_ref: String,
    /// Commits between refs
    pub commits: Vec<ChangelogCommit>,
    /// Selected commit index
    pub selected_commit: usize,
    /// Commit list scroll offset
    pub commit_scroll: usize,
    /// Diff view state
    pub diff_view: DiffViewState,
    /// Generated changelog content (markdown)
    pub changelog_content: String,
    /// Changelog content scroll offset
    pub changelog_scroll: usize,
    /// Whether changelog is being generated
    pub generating: bool,
}

impl Default for ChangelogState {
    fn default() -> Self {
        Self {
            from_ref: "HEAD~10".to_string(),
            to_ref: "HEAD".to_string(),
            commits: Vec::new(),
            selected_commit: 0,
            commit_scroll: 0,
            diff_view: DiffViewState::new(),
            changelog_content: String::new(),
            changelog_scroll: 0,
            generating: false,
        }
    }
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

    /// Open chat modal
    pub fn show_chat(&mut self) {
        self.modal = Some(Modal::Chat(ChatState::new()));
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

    /// Get list of branch refs for selection
    pub fn get_branch_refs(&self) -> Vec<String> {
        let Some(git_repo) = &self.repo else {
            return vec!["main".to_string(), "master".to_string()];
        };

        let Ok(repo) = git_repo.open_repo() else {
            return vec!["main".to_string(), "master".to_string()];
        };

        let mut refs = Vec::new();

        // Get local branches
        if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
            for branch in branches.flatten() {
                if let Ok(Some(name)) = branch.0.name() {
                    refs.push(name.to_string());
                }
            }
        }

        // Get remote branches (origin/*)
        if let Ok(branches) = repo.branches(Some(git2::BranchType::Remote)) {
            for branch in branches.flatten() {
                if let Ok(Some(name)) = branch.0.name() {
                    // Skip HEAD references
                    if !name.ends_with("/HEAD") {
                        refs.push(name.to_string());
                    }
                }
            }
        }

        // Sort with common branches first
        refs.sort_by(|a, b| {
            let priority = |s: &str| -> i32 {
                match s {
                    "main" => 0,
                    "master" => 1,
                    s if s.starts_with("origin/main") => 2,
                    s if s.starts_with("origin/master") => 3,
                    s if s.starts_with("origin/") => 5,
                    _ => 4,
                }
            };
            priority(a).cmp(&priority(b)).then(a.cmp(b))
        });

        if refs.is_empty() {
            refs.push("main".to_string());
        }

        refs
    }

    /// Get list of commit-related presets for selection
    pub fn get_commit_presets(&self) -> Vec<PresetInfo> {
        use crate::instruction_presets::{PresetType, get_instruction_preset_library};

        let library = get_instruction_preset_library();
        let mut presets: Vec<PresetInfo> = library
            .list_presets_by_type(Some(PresetType::Commit))
            .into_iter()
            .chain(library.list_presets_by_type(Some(PresetType::Both)))
            .map(|(key, preset)| PresetInfo {
                key: key.clone(),
                name: preset.name.clone(),
                description: preset.description.clone(),
                emoji: preset.emoji.clone(),
            })
            .collect();

        // Sort by name, but put "default" first
        presets.sort_by(|a, b| {
            if a.key == "default" {
                std::cmp::Ordering::Less
            } else if b.key == "default" {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        presets
    }
}
