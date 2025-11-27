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

pub mod code_view;
pub mod diff_view;
pub mod file_tree;
pub mod message_editor;
pub mod syntax;

// Re-export commonly used items
pub use code_view::{CodeViewState, render_code_view};
pub use diff_view::{DiffHunk, DiffLine, DiffViewState, FileDiff, parse_diff, render_diff_view};
pub use file_tree::{FileGitStatus, FileTreeState, TreeNode, render_file_tree};
pub use message_editor::{MessageEditorState, render_message_editor};
pub use syntax::SyntaxHighlighter;
