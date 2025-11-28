//! `SilkCircuit` Neon theme for Iris Studio
//!
//! Electric meets elegant - the visual identity for git-iris TUI.
//!
//! This module wraps the centralized token-based theme system,
//! providing access to colors and styles through the theme API.

use ratatui::style::{Color, Style};

use crate::theme;
use crate::theme::adapters::ratatui::{ToRatatuiColor, ToRatatuiStyle};

// ═══════════════════════════════════════════════════════════════════════════════
// Semantic Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for commit hashes
pub fn commit_hash() -> Style {
    theme::current().style("commit_hash").to_ratatui()
}

/// Style for file paths
pub fn file_path() -> Style {
    theme::current().style("file_path").to_ratatui()
}

/// Style for keywords and important markers
pub fn keyword() -> Style {
    theme::current().style("keyword").to_ratatui()
}

/// Style for line numbers in code views
pub fn line_number() -> Style {
    theme::current().style("line_number").to_ratatui()
}

/// Style for selected items
pub fn selected() -> Style {
    theme::current().style("selected").to_ratatui()
}

/// Style for focused panel border
pub fn focused_border() -> Style {
    theme::current().style("focused_border").to_ratatui()
}

/// Style for unfocused panel border
pub fn unfocused_border() -> Style {
    theme::current().style("unfocused_border").to_ratatui()
}

/// Style for success messages
pub fn success() -> Style {
    theme::current().style("success_style").to_ratatui()
}

/// Style for error messages
pub fn error() -> Style {
    theme::current().style("error_style").to_ratatui()
}

/// Style for warning messages
pub fn warning() -> Style {
    theme::current().style("warning_style").to_ratatui()
}

/// Style for timestamps
pub fn timestamp() -> Style {
    theme::current().style("timestamp").to_ratatui()
}

/// Style for author names
pub fn author() -> Style {
    theme::current().style("author").to_ratatui()
}

/// Style for dimmed/secondary text
pub fn dimmed() -> Style {
    theme::current().style("dimmed").to_ratatui()
}

/// Style for inline code in chat/markdown
pub fn inline_code() -> Style {
    theme::current().style("inline_code").to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Git Status Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for git staged files
pub fn git_staged() -> Style {
    theme::current().style("git_staged").to_ratatui()
}

/// Style for git modified files
pub fn git_modified() -> Style {
    theme::current().style("git_modified").to_ratatui()
}

/// Style for git untracked files
pub fn git_untracked() -> Style {
    theme::current().style("git_untracked").to_ratatui()
}

/// Style for git deleted files
pub fn git_deleted() -> Style {
    theme::current().style("git_deleted").to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diff Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for added lines in diff
pub fn diff_added() -> Style {
    theme::current().style("diff_added").to_ratatui()
}

/// Style for removed lines in diff
pub fn diff_removed() -> Style {
    theme::current().style("diff_removed").to_ratatui()
}

/// Style for diff hunk headers
pub fn diff_hunk() -> Style {
    theme::current().style("diff_hunk").to_ratatui()
}

/// Style for diff context lines
pub fn diff_context() -> Style {
    theme::current().style("diff_context").to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Color Accessors
// ═══════════════════════════════════════════════════════════════════════════════

/// Get primary accent color (Electric Purple)
pub fn accent_primary() -> Color {
    theme::current().color("accent.primary").to_ratatui()
}

/// Get secondary accent color (Neon Cyan)
pub fn accent_secondary() -> Color {
    theme::current().color("accent.secondary").to_ratatui()
}

/// Get tertiary accent color (Coral)
pub fn accent_tertiary() -> Color {
    theme::current().color("accent.tertiary").to_ratatui()
}

/// Get warning color (Electric Yellow)
pub fn warning_color() -> Color {
    theme::current().color("warning").to_ratatui()
}

/// Get success color (Success Green)
pub fn success_color() -> Color {
    theme::current().color("success").to_ratatui()
}

/// Get error color (Error Red)
pub fn error_color() -> Color {
    theme::current().color("error").to_ratatui()
}

/// Get primary text color
pub fn text_primary_color() -> Color {
    theme::current().color("text.primary").to_ratatui()
}

/// Get secondary text color
pub fn text_secondary_color() -> Color {
    theme::current().color("text.secondary").to_ratatui()
}

/// Get dim text color
pub fn text_dim_color() -> Color {
    theme::current().color("text.dim").to_ratatui()
}

/// Get muted text color
pub fn text_muted_color() -> Color {
    theme::current().color("text.muted").to_ratatui()
}

/// Get highlight background color
pub fn bg_highlight_color() -> Color {
    theme::current().color("bg.highlight").to_ratatui()
}

/// Get selection background color
pub fn bg_selection_color() -> Color {
    theme::current().color("bg.selection").to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mode Tab Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for inactive mode tab
pub fn mode_inactive() -> Style {
    theme::current().style("mode_inactive").to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Gradients
// ═══════════════════════════════════════════════════════════════════════════════

/// Get a color for a gradient position (0.0 = start, 1.0 = end)
/// Gradient goes from Electric Purple → Neon Cyan
pub fn gradient_purple_cyan(position: f32) -> Color {
    theme::current().gradient("primary", position).to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Animation
// ═══════════════════════════════════════════════════════════════════════════════

/// Braille spinner frames for loading indicators
pub const SPINNER_BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
