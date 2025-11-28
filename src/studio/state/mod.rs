//! State management for Iris Studio
//!
//! Centralized state for all modes and shared data.

mod chat;
mod modes;

pub use chat::{ChatMessage, ChatRole, ChatState, truncate_preview};
pub use modes::{ChangelogCommit, ModeStates, PrCommit};

use crate::config::Config;
use crate::git::GitRepo;
use crate::types::format_commit_message;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════════
// Mode Enum
// ═══════════════════════════════════════════════════════════════════════════════

/// Available modes in Iris Studio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Explore mode - semantic code understanding
    #[default]
    Explore,
    /// Commit mode - generate and edit commit messages
    Commit,
    /// Review mode - AI-powered code review
    Review,
    /// PR mode - pull request creation
    PR,
    /// Changelog mode - structured changelogs
    Changelog,
    /// Release Notes mode - release documentation
    ReleaseNotes,
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
            Mode::ReleaseNotes => "Release",
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
            Mode::ReleaseNotes => 'N',
        }
    }

    /// Check if this mode is available (implemented)
    pub fn is_available(&self) -> bool {
        matches!(
            self,
            Mode::Explore
                | Mode::Commit
                | Mode::Review
                | Mode::PR
                | Mode::Changelog
                | Mode::ReleaseNotes
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
            Mode::ReleaseNotes,
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

/// Emoji info for display in selector
#[derive(Debug, Clone)]
pub struct EmojiInfo {
    /// The emoji character
    pub emoji: String,
    /// Short key/code (e.g., "feat", "fix")
    pub key: String,
    /// Description
    pub description: String,
}

/// Emoji mode for commit messages
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EmojiMode {
    /// No emoji
    None,
    /// Let AI choose the emoji
    #[default]
    Auto,
    /// User-selected specific emoji
    Custom(String),
}

/// Active modal dialog
pub enum Modal {
    /// Help overlay showing keybindings
    Help,
    /// Search modal for files/symbols
    Search {
        query: String,
        results: Vec<String>,
        selected: usize,
    },
    /// Confirmation dialog
    Confirm { message: String, action: String },
    /// Instructions input for commit message generation
    Instructions { input: String },
    /// Chat interface with Iris (state lives in `StudioState.chat_state`)
    Chat,
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
    /// Emoji selector for commit messages
    EmojiSelector {
        /// Current input/filter
        input: String,
        /// Available emojis (None, Auto, then all gitmojis)
        emojis: Vec<EmojiInfo>,
        /// Selected index
        selected: usize,
        /// Scroll offset for long lists
        scroll: usize,
    },
    /// Settings configuration modal
    Settings(Box<SettingsState>),
    /// Theme selector modal
    ThemeSelector {
        /// Current input/filter
        input: String,
        /// Available themes
        themes: Vec<ThemeOptionInfo>,
        /// Selected index
        selected: usize,
        /// Scroll offset for long lists
        scroll: usize,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Settings State
// ═══════════════════════════════════════════════════════════════════════════════

/// Settings section for grouped display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Provider,
    Appearance,
    Behavior,
}

impl SettingsSection {
    /// Get the display name for this section
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsSection::Provider => "Provider",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Behavior => "Behavior",
        }
    }
}

/// Field being edited in settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Provider,
    Model,
    ApiKey,
    Theme,
    UseGitmoji,
    InstructionPreset,
    CustomInstructions,
}

impl SettingsField {
    /// Get all fields in display order
    pub fn all() -> &'static [SettingsField] {
        &[
            SettingsField::Provider,
            SettingsField::Model,
            SettingsField::ApiKey,
            SettingsField::Theme,
            SettingsField::UseGitmoji,
            SettingsField::InstructionPreset,
            SettingsField::CustomInstructions,
        ]
    }

    /// Get field display name
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsField::Provider => "Provider",
            SettingsField::Model => "Model",
            SettingsField::ApiKey => "API Key",
            SettingsField::Theme => "Theme",
            SettingsField::UseGitmoji => "Gitmoji",
            SettingsField::InstructionPreset => "Preset",
            SettingsField::CustomInstructions => "Instructions",
        }
    }

    /// Get which section this field belongs to
    pub fn section(&self) -> SettingsSection {
        match self {
            SettingsField::Provider | SettingsField::Model | SettingsField::ApiKey => {
                SettingsSection::Provider
            }
            SettingsField::Theme => SettingsSection::Appearance,
            SettingsField::UseGitmoji
            | SettingsField::InstructionPreset
            | SettingsField::CustomInstructions => SettingsSection::Behavior,
        }
    }
}

/// Theme info for settings and selector display
#[derive(Debug, Clone)]
pub struct ThemeOptionInfo {
    /// Theme identifier (e.g., `silkcircuit-neon`)
    pub id: String,
    /// Display name (e.g., `SilkCircuit Neon`)
    pub display_name: String,
    /// Variant indicator (dark/light)
    pub variant: String,
    /// Theme author
    pub author: String,
    /// Theme description
    pub description: String,
}

/// State for the settings modal
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Currently selected field
    pub selected_field: usize,
    /// Currently editing a field
    pub editing: bool,
    /// Text input buffer for editing
    pub input_buffer: String,
    /// Current provider
    pub provider: String,
    /// Current model
    pub model: String,
    /// API key (masked for display)
    pub api_key_display: String,
    /// Actual API key (for saving)
    pub api_key_actual: Option<String>,
    /// Current theme identifier
    pub theme: String,
    /// Use gitmoji
    pub use_gitmoji: bool,
    /// Instruction preset
    pub instruction_preset: String,
    /// Custom instructions for all operations
    pub custom_instructions: String,
    /// Available providers
    pub available_providers: Vec<String>,
    /// Available themes
    pub available_themes: Vec<ThemeOptionInfo>,
    /// Available presets
    pub available_presets: Vec<String>,
    /// Whether config was modified
    pub modified: bool,
    /// Error message if any
    pub error: Option<String>,
}

impl SettingsState {
    /// Create settings state from current config
    pub fn from_config(config: &Config) -> Self {
        use crate::instruction_presets::get_instruction_preset_library;
        use crate::providers::Provider;
        use crate::theme;

        let provider = config.default_provider.clone();
        let provider_config = config.get_provider_config(&provider);

        let model = provider_config.map(|p| p.model.clone()).unwrap_or_default();

        let api_key_display = provider_config
            .map(|p| Self::mask_api_key(&p.api_key))
            .unwrap_or_default();

        let available_providers: Vec<String> =
            Provider::ALL.iter().map(|p| p.name().to_string()).collect();

        // Get available themes (sorted: dark first, then light, alphabetically within each)
        let mut available_themes: Vec<ThemeOptionInfo> = theme::list_available_themes()
            .into_iter()
            .map(|info| ThemeOptionInfo {
                id: info.name,
                display_name: info.display_name,
                variant: match info.variant {
                    theme::ThemeVariant::Dark => "dark".to_string(),
                    theme::ThemeVariant::Light => "light".to_string(),
                },
                author: info.author,
                description: info.description,
            })
            .collect();
        available_themes.sort_by(|a, b| {
            // Dark themes first, then sort alphabetically
            match (a.variant.as_str(), b.variant.as_str()) {
                ("dark", "light") => std::cmp::Ordering::Less,
                ("light", "dark") => std::cmp::Ordering::Greater,
                _ => a.display_name.cmp(&b.display_name),
            }
        });

        // Get current theme name
        let current_theme = theme::current();
        let theme_id = available_themes
            .iter()
            .find(|t| t.display_name == current_theme.meta.name)
            .map_or_else(|| "silkcircuit-neon".to_string(), |t| t.id.clone());

        let preset_library = get_instruction_preset_library();
        let available_presets: Vec<String> = preset_library
            .list_presets()
            .iter()
            .map(|(key, _)| (*key).clone())
            .collect();

        Self {
            selected_field: 0,
            editing: false,
            input_buffer: String::new(),
            provider,
            model,
            api_key_display,
            api_key_actual: None, // Only set when user enters a new key
            theme: theme_id,
            use_gitmoji: config.use_gitmoji,
            instruction_preset: config.instruction_preset.clone(),
            custom_instructions: config.instructions.clone(),
            available_providers,
            available_themes,
            available_presets,
            modified: false,
            error: None,
        }
    }

    /// Mask an API key for display
    fn mask_api_key(key: &str) -> String {
        if key.is_empty() {
            "(not set)".to_string()
        } else {
            let len = key.len();
            if len <= 8 {
                "*".repeat(len)
            } else {
                format!("{}...{}", &key[..4], &key[len - 4..])
            }
        }
    }

    /// Get the currently selected field
    pub fn current_field(&self) -> SettingsField {
        SettingsField::all()[self.selected_field]
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_field > 0 {
            self.selected_field -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let max = SettingsField::all().len() - 1;
        if self.selected_field < max {
            self.selected_field += 1;
        }
    }

    /// Get the current value for a field
    pub fn get_field_value(&self, field: SettingsField) -> String {
        match field {
            SettingsField::Provider => self.provider.clone(),
            SettingsField::Model => self.model.clone(),
            SettingsField::ApiKey => self.api_key_display.clone(),
            SettingsField::Theme => self
                .available_themes
                .iter()
                .find(|t| t.id == self.theme)
                .map_or_else(|| self.theme.clone(), |t| t.display_name.clone()),
            SettingsField::UseGitmoji => {
                if self.use_gitmoji {
                    "yes".to_string()
                } else {
                    "no".to_string()
                }
            }
            SettingsField::InstructionPreset => self.instruction_preset.clone(),
            SettingsField::CustomInstructions => {
                if self.custom_instructions.is_empty() {
                    "(none)".to_string()
                } else {
                    // Truncate for display if too long
                    let preview = self.custom_instructions.lines().next().unwrap_or("");
                    if preview.len() > 30 || self.custom_instructions.lines().count() > 1 {
                        format!("{}...", &preview.chars().take(30).collect::<String>())
                    } else {
                        preview.to_string()
                    }
                }
            }
        }
    }

    /// Get the current theme info
    pub fn current_theme_info(&self) -> Option<&ThemeOptionInfo> {
        self.available_themes.iter().find(|t| t.id == self.theme)
    }

    /// Cycle through options for the current field (forward direction)
    pub fn cycle_current_field(&mut self) {
        self.cycle_field_direction(true);
    }

    /// Cycle through options for the current field (backward direction)
    pub fn cycle_current_field_back(&mut self) {
        self.cycle_field_direction(false);
    }

    /// Cycle through options for the current field in given direction
    fn cycle_field_direction(&mut self, forward: bool) {
        let field = self.current_field();
        match field {
            SettingsField::Provider => {
                if let Some(idx) = self
                    .available_providers
                    .iter()
                    .position(|p| p == &self.provider)
                {
                    let next = if forward {
                        (idx + 1) % self.available_providers.len()
                    } else if idx == 0 {
                        self.available_providers.len() - 1
                    } else {
                        idx - 1
                    };
                    self.provider = self.available_providers[next].clone();
                    self.modified = true;
                }
            }
            SettingsField::Theme => {
                if let Some(idx) = self
                    .available_themes
                    .iter()
                    .position(|t| t.id == self.theme)
                {
                    let next = if forward {
                        (idx + 1) % self.available_themes.len()
                    } else if idx == 0 {
                        self.available_themes.len() - 1
                    } else {
                        idx - 1
                    };
                    self.theme = self.available_themes[next].id.clone();
                    self.modified = true;
                    // Apply theme immediately for live preview
                    let _ = crate::theme::load_theme_by_name(&self.theme);
                }
            }
            SettingsField::UseGitmoji => {
                self.use_gitmoji = !self.use_gitmoji;
                self.modified = true;
            }
            SettingsField::InstructionPreset => {
                if let Some(idx) = self
                    .available_presets
                    .iter()
                    .position(|p| p == &self.instruction_preset)
                {
                    let next = if forward {
                        (idx + 1) % self.available_presets.len()
                    } else if idx == 0 {
                        self.available_presets.len() - 1
                    } else {
                        idx - 1
                    };
                    self.instruction_preset = self.available_presets[next].clone();
                    self.modified = true;
                }
            }
            _ => {}
        }
    }

    /// Start editing the current field
    pub fn start_editing(&mut self) {
        let field = self.current_field();
        match field {
            SettingsField::Model => {
                self.input_buffer = self.model.clone();
                self.editing = true;
            }
            SettingsField::ApiKey => {
                self.input_buffer.clear(); // Start fresh for API key
                self.editing = true;
            }
            SettingsField::CustomInstructions => {
                self.input_buffer = self.custom_instructions.clone();
                self.editing = true;
            }
            _ => {
                // For other fields, cycle instead
                self.cycle_current_field();
            }
        }
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.input_buffer.clear();
    }

    /// Confirm editing
    pub fn confirm_editing(&mut self) {
        if !self.editing {
            return;
        }

        let field = self.current_field();
        match field {
            SettingsField::Model => {
                if !self.input_buffer.is_empty() {
                    self.model = self.input_buffer.clone();
                    self.modified = true;
                }
            }
            SettingsField::ApiKey => {
                if !self.input_buffer.is_empty() {
                    // Store actual key, update display
                    let key = self.input_buffer.clone();
                    self.api_key_display = Self::mask_api_key(&key);
                    self.api_key_actual = Some(key);
                    self.modified = true;
                }
            }
            SettingsField::CustomInstructions => {
                // Allow empty (clears instructions)
                self.custom_instructions = self.input_buffer.clone();
                self.modified = true;
            }
            _ => {}
        }

        self.editing = false;
        self.input_buffer.clear();
    }
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
    /// Release notes from version
    ReleaseNotesFrom,
    /// Release notes to version
    ReleaseNotesTo,
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
// Main Studio State
// ═══════════════════════════════════════════════════════════════════════════════

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

    /// Active modal
    pub modal: Option<Modal>,

    /// Persistent chat state (survives modal close, universal across modes)
    pub chat_state: ChatState,

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
            modal: None,
            chat_state: ChatState::new(),
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

        // Set default focus based on mode
        self.focused_panel = match new_mode {
            // Commit mode: focus on message editor (center panel)
            Mode::Commit => PanelId::Center,
            // Review/PR/Changelog/Release: focus on output (center panel)
            Mode::Review | Mode::PR | Mode::Changelog | Mode::ReleaseNotes => PanelId::Center,
            // Explore: focus on file tree (left panel)
            Mode::Explore => PanelId::Left,
        };
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

    /// Open chat modal (universal, persists across modes)
    pub fn show_chat(&mut self) {
        // If chat is empty, initialize with context from all generated content
        if self.chat_state.messages.is_empty() {
            let context = self.build_chat_context();
            self.chat_state = ChatState::with_context("git workflow", context.as_deref());
        }

        // Open chat modal (state lives in self.chat_state)
        self.modal = Some(Modal::Chat);
        self.dirty = true;
    }

    /// Build context summary from all generated content for chat
    fn build_chat_context(&self) -> Option<String> {
        let mut sections = Vec::new();

        // Commit message
        if let Some(msg) = self
            .modes
            .commit
            .messages
            .get(self.modes.commit.current_index)
        {
            let formatted = format_commit_message(msg);
            if !formatted.trim().is_empty() {
                sections.push(format!("Commit Message:\n{}", formatted));
            }
        }

        // Code review
        if !self.modes.review.review_content.is_empty() {
            let preview = truncate_preview(&self.modes.review.review_content, 300);
            sections.push(format!("Code Review:\n{}", preview));
        }

        // PR description
        if !self.modes.pr.pr_content.is_empty() {
            let preview = truncate_preview(&self.modes.pr.pr_content, 300);
            sections.push(format!("PR Description:\n{}", preview));
        }

        // Changelog
        if !self.modes.changelog.changelog_content.is_empty() {
            let preview = truncate_preview(&self.modes.changelog.changelog_content, 300);
            sections.push(format!("Changelog:\n{}", preview));
        }

        // Release notes
        if !self.modes.release_notes.release_notes_content.is_empty() {
            let preview = truncate_preview(&self.modes.release_notes.release_notes_content, 300);
            sections.push(format!("Release Notes:\n{}", preview));
        }

        if sections.is_empty() {
            None
        } else {
            Some(sections.join("\n\n"))
        }
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

        // Mark dirty if we have active animations
        if matches!(self.iris_status, IrisStatus::Thinking { .. }) {
            self.dirty = true;
        }

        // Also mark dirty if chat modal is responding (for spinner animation)
        if matches!(self.modal, Some(Modal::Chat)) && self.chat_state.is_responding {
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

    /// Get list of emojis for selection (None, Auto, then all gitmojis)
    pub fn get_emoji_list(&self) -> Vec<EmojiInfo> {
        use crate::gitmoji::get_gitmoji_list;

        let mut emojis = vec![
            EmojiInfo {
                emoji: "∅".to_string(),
                key: "none".to_string(),
                description: "No emoji".to_string(),
            },
            EmojiInfo {
                emoji: "✨".to_string(),
                key: "auto".to_string(),
                description: "Let AI choose".to_string(),
            },
        ];

        // Parse gitmoji list and add all entries
        for line in get_gitmoji_list().lines() {
            // Format: "emoji - :key: - description"
            let parts: Vec<&str> = line.splitn(3, " - ").collect();
            if parts.len() >= 3 {
                let emoji = parts[0].trim().to_string();
                let key = parts[1].trim_matches(':').to_string();
                let description = parts[2].to_string();
                emojis.push(EmojiInfo {
                    emoji,
                    key,
                    description,
                });
            }
        }

        emojis
    }
}
