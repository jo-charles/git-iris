//! Mode-specific state structs for Iris Studio
//!
//! Each mode (Explore, Commit, Review, PR, Changelog, `ReleaseNotes`) has its own state struct.

use std::path::PathBuf;

use crate::types::GeneratedMessage;

use super::super::components::{CodeViewState, DiffViewState, FileTreeState, MessageEditorState};
use super::EmojiMode;

// ═══════════════════════════════════════════════════════════════════════════════
// Explore Mode
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
    /// Selection anchor line for visual mode (where 'v' was pressed)
    pub selection_anchor: Option<usize>,
    /// Code view scroll offset
    pub code_scroll: usize,
    /// Heat map enabled
    pub show_heat_map: bool,
    /// File tree state
    pub file_tree: FileTreeState,
    /// Code view state
    pub code_view: CodeViewState,
    /// Current semantic blame result (for context panel)
    pub semantic_blame: Option<super::super::events::SemanticBlameResult>,
    /// Streaming blame content (while generating)
    pub streaming_blame: Option<String>,
    /// Whether semantic blame is loading
    pub blame_loading: bool,
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

// ═══════════════════════════════════════════════════════════════════════════════
// Commit Mode
// ═══════════════════════════════════════════════════════════════════════════════

/// State for Commit mode
#[allow(clippy::struct_excessive_bools)]
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
    /// Use gitmoji (legacy toggle, replaced by `emoji_mode`)
    pub use_gitmoji: bool,
    /// Current emoji mode (None, Auto, or Custom)
    pub emoji_mode: EmojiMode,
    /// Current preset name
    pub preset: String,
    /// File tree state for staged files
    pub file_tree: FileTreeState,
    /// Diff view state
    pub diff_view: DiffViewState,
    /// Message editor state
    pub message_editor: MessageEditorState,
    /// Show all tracked files (vs only staged/modified)
    pub show_all_files: bool,
    /// Whether we're amending the previous commit
    pub amend_mode: bool,
    /// Original commit message (when amending)
    pub original_message: Option<String>,
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
            emoji_mode: EmojiMode::Auto,
            preset: "default".to_string(),
            file_tree: FileTreeState::new(),
            diff_view: DiffViewState::new(),
            message_editor: MessageEditorState::new(),
            show_all_files: false,
            amend_mode: false,
            original_message: None,
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
            .field("amend_mode", &self.amend_mode)
            .finish_non_exhaustive()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Review Mode
// ═══════════════════════════════════════════════════════════════════════════════

/// State for Review mode
pub struct ReviewState {
    /// File tree for changed files
    pub file_tree: FileTreeState,
    /// Diff view for selected file
    pub diff_view: DiffViewState,
    /// Generated review content (markdown)
    pub review_content: String,
    /// Streaming content (while generating)
    pub streaming_content: Option<String>,
    /// Review scroll offset
    pub review_scroll: usize,
    /// Whether a review is being generated
    pub generating: bool,
    /// From ref for comparison (defaults to HEAD~1 for most recent commit)
    pub from_ref: String,
    /// To ref for comparison (defaults to HEAD)
    pub to_ref: String,
}

impl Default for ReviewState {
    fn default() -> Self {
        Self {
            file_tree: FileTreeState::default(),
            diff_view: DiffViewState::default(),
            review_content: String::new(),
            streaming_content: None,
            review_scroll: 0,
            generating: false,
            from_ref: "HEAD~1".to_string(),
            to_ref: "HEAD".to_string(),
        }
    }
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

// ═══════════════════════════════════════════════════════════════════════════════
// PR Mode
// ═══════════════════════════════════════════════════════════════════════════════

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
    /// Streaming content (while generating)
    pub streaming_content: Option<String>,
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
            streaming_content: None,
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

// ═══════════════════════════════════════════════════════════════════════════════
// Changelog Mode
// ═══════════════════════════════════════════════════════════════════════════════

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
    /// Streaming content (while generating)
    pub streaming_content: Option<String>,
    /// Changelog content scroll offset
    pub changelog_scroll: usize,
    /// Whether changelog is being generated
    pub generating: bool,
}

impl Default for ChangelogState {
    fn default() -> Self {
        Self {
            from_ref: "HEAD~1".to_string(),
            to_ref: "HEAD".to_string(),
            commits: Vec::new(),
            selected_commit: 0,
            commit_scroll: 0,
            diff_view: DiffViewState::new(),
            changelog_content: String::new(),
            streaming_content: None,
            changelog_scroll: 0,
            generating: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Release Notes Mode
// ═══════════════════════════════════════════════════════════════════════════════

/// State for Release Notes mode
#[derive(Debug)]
pub struct ReleaseNotesState {
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
    /// Generated release notes content (markdown)
    pub release_notes_content: String,
    /// Streaming content (while generating)
    pub streaming_content: Option<String>,
    /// Release notes content scroll offset
    pub release_notes_scroll: usize,
    /// Whether release notes are being generated
    pub generating: bool,
}

impl Default for ReleaseNotesState {
    fn default() -> Self {
        Self {
            from_ref: "HEAD~1".to_string(),
            to_ref: "HEAD".to_string(),
            commits: Vec::new(),
            selected_commit: 0,
            commit_scroll: 0,
            diff_view: DiffViewState::new(),
            release_notes_content: String::new(),
            streaming_content: None,
            release_notes_scroll: 0,
            generating: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mode States Container
// ═══════════════════════════════════════════════════════════════════════════════

/// Container for all mode states
#[derive(Debug, Default)]
pub struct ModeStates {
    pub explore: ExploreState,
    pub commit: CommitState,
    pub review: ReviewState,
    pub pr: PrState,
    pub changelog: ChangelogState,
    pub release_notes: ReleaseNotesState,
}
