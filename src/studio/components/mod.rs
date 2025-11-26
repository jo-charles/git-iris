//! Shared UI components for Iris Studio
//!
//! Reusable components across all modes:
//! - `file_tree`: Directory navigation with git status
//! - `code_view`: Syntax-highlighted source display
//! - `diff_view`: Unified/split diff rendering
//! - `commit_list`: Commit history display
//! - `message_editor`: Text editing for messages
//! - `context_panel`: Semantic context display
//! - `status_bar`: Bottom status and Iris status
//! - `help_overlay`: Keybinding reference

pub mod diff_view;
pub mod file_tree;
pub mod message_editor;

// Re-export commonly used items
pub use diff_view::{DiffViewState, FileDiff, parse_diff, render_diff_view};
pub use file_tree::{FileGitStatus, FileTreeState, TreeNode, render_file_tree};
pub use message_editor::{MessageEditorState, render_message_editor};

// Placeholder modules - to be implemented
// pub mod code_view;
// pub mod commit_list;
// pub mod message_editor;
// pub mod context_panel;
// pub mod status_bar;
// pub mod help_overlay;
