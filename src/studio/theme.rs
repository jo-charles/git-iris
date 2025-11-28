//! `SilkCircuit` Neon theme for Iris Studio
//!
//! Electric meets elegant - the visual identity for git-iris TUI.
//!
//! This module now wraps the centralized token-based theme system,
//! providing backwards-compatible access to colors and styles.

#![allow(dead_code)] // Theme constants/functions are scaffolded for future use

use ratatui::style::{Color, Style};
use ratatui::text::Span;

use crate::theme::adapters::ratatui::{gradient_line as theme_gradient_line, ToRatatuiColor, ToRatatuiStyle};
use crate::theme;

// ═══════════════════════════════════════════════════════════════════════════════
// Core Palette (derived from theme tokens)
// ═══════════════════════════════════════════════════════════════════════════════

/// Get Electric Purple from theme
fn electric_purple() -> Color {
    theme::current().color("accent.primary").to_ratatui()
}

/// Get Neon Cyan from theme
fn neon_cyan() -> Color {
    theme::current().color("accent.secondary").to_ratatui()
}

/// Get Coral from theme
fn coral() -> Color {
    theme::current().color("accent.tertiary").to_ratatui()
}

/// Get Electric Yellow from theme
fn electric_yellow() -> Color {
    theme::current().color("warning").to_ratatui()
}

/// Get Success Green from theme
fn success_green() -> Color {
    theme::current().color("success").to_ratatui()
}

/// Get Error Red from theme
fn error_red() -> Color {
    theme::current().color("error").to_ratatui()
}

// Legacy constants - kept for backwards compatibility
// TODO: Deprecate these in favor of theme::current().color()

/// Electric Purple #e135ff - Keywords, markers, importance, active mode
pub const ELECTRIC_PURPLE: Color = Color::Rgb(225, 53, 255);

/// Neon Cyan #80ffea - Functions, paths, interactions, focus
pub const NEON_CYAN: Color = Color::Rgb(128, 255, 234);

/// Coral #ff6ac1 - Hashes, numbers, constants
pub const CORAL: Color = Color::Rgb(255, 106, 193);

/// Electric Yellow #f1fa8c - Warnings, timestamps, attention
pub const ELECTRIC_YELLOW: Color = Color::Rgb(241, 250, 140);

/// Success Green #50fa7b - Success states, confirmations
pub const SUCCESS_GREEN: Color = Color::Rgb(80, 250, 123);

/// Error Red #ff6363 - Errors, danger, removals
pub const ERROR_RED: Color = Color::Rgb(255, 99, 99);

// ═══════════════════════════════════════════════════════════════════════════════
// Gradient Colors (for SilkCircuit Neon aesthetic)
// ═══════════════════════════════════════════════════════════════════════════════

/// Purple gradient end - darker variant
pub const GRADIENT_PURPLE_DARK: Color = Color::Rgb(140, 30, 180);

/// Cyan gradient end - darker variant
pub const GRADIENT_CYAN_DARK: Color = Color::Rgb(60, 180, 160);

/// Pink/Magenta accent for highlights
pub const MAGENTA_ACCENT: Color = Color::Rgb(255, 85, 255);

/// Soft purple for subtle accents
pub const SOFT_PURPLE: Color = Color::Rgb(180, 130, 255);

// ═══════════════════════════════════════════════════════════════════════════════
// Backgrounds (derived from theme tokens)
// ═══════════════════════════════════════════════════════════════════════════════

/// Get background base from theme
fn bg_base() -> Color {
    theme::current().color("bg.base").to_ratatui()
}

/// Get panel background from theme
fn bg_panel() -> Color {
    theme::current().color("bg.panel").to_ratatui()
}

/// Get highlight background from theme
fn bg_highlight() -> Color {
    theme::current().color("bg.highlight").to_ratatui()
}

/// Get active background from theme
fn bg_active() -> Color {
    theme::current().color("bg.active").to_ratatui()
}

/// Get code background from theme
fn bg_code() -> Color {
    theme::current().color("bg.code").to_ratatui()
}

// Legacy background constants

/// Dark background base
pub const BG_DARK: Color = Color::Rgb(18, 18, 24);

/// Panel background
pub const BG_PANEL: Color = Color::Rgb(24, 24, 32);

/// Highlighted/selected background - purple tint, distinct from `BG_ACTIVE`
pub const BG_HIGHLIGHT: Color = Color::Rgb(55, 50, 75);

/// Elevated surface
pub const BG_ELEVATED: Color = Color::Rgb(55, 50, 70);

/// Active selection background - vibrant
pub const BG_ACTIVE: Color = Color::Rgb(60, 45, 85);

/// Inline code background - subtle dark
pub const BG_CODE: Color = Color::Rgb(30, 30, 40);

// ═══════════════════════════════════════════════════════════════════════════════
// Text Colors (derived from theme tokens)
// ═══════════════════════════════════════════════════════════════════════════════

/// Get primary text from theme
fn text_primary() -> Color {
    theme::current().color("text.primary").to_ratatui()
}

/// Get dim text from theme
fn text_dim() -> Color {
    theme::current().color("text.dim").to_ratatui()
}

/// Get muted text from theme
fn text_muted() -> Color {
    theme::current().color("text.muted").to_ratatui()
}

// Legacy text constants

/// Primary text - soft white
pub const TEXT_PRIMARY: Color = Color::Rgb(248, 248, 242);

/// Secondary/dimmed text - line numbers, metadata (~4.5:1 contrast)
pub const TEXT_DIM: Color = Color::Rgb(110, 125, 175);

/// Muted text for borders and less important elements (~5.5:1 contrast)
pub const TEXT_MUTED: Color = Color::Rgb(130, 135, 160);

/// Secondary text - slightly dimmed
pub const TEXT_SECONDARY: Color = Color::Rgb(188, 188, 202);

/// Selection background for multi-line selections
pub const SELECTION_BG: Color = Color::Rgb(60, 60, 80);

// ═══════════════════════════════════════════════════════════════════════════════
// Semantic Styles (using new theme system)
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for commit hashes
pub fn commit_hash() -> Style {
    theme::current().style("commit_hash").to_ratatui()
}

/// Style for file paths
pub fn file_path() -> Style {
    theme::current().style("file_path").to_ratatui()
}

/// Style for file paths with bold
pub fn file_path_bold() -> Style {
    theme::current().style("file_path_bold").to_ratatui()
}

/// Style for keywords and important markers
pub fn keyword() -> Style {
    theme::current().style("keyword").to_ratatui()
}

/// Style for line numbers
pub fn line_number() -> Style {
    theme::current().style("line_number").to_ratatui()
}

/// Style for the current/cursor line
pub fn cursor_line() -> Style {
    theme::current().style("cursor_line").to_ratatui()
}

/// Style for selected items
pub fn selected() -> Style {
    theme::current().style("selected").to_ratatui()
}

/// Style for actively selected (focused panel, selected item)
pub fn active_selected() -> Style {
    theme::current().style("active_selected").to_ratatui()
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
// Mode Tab Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for active mode tab
pub fn mode_active() -> Style {
    theme::current().style("mode_active").to_ratatui()
}

/// Style for inactive mode tab
pub fn mode_inactive() -> Style {
    theme::current().style("mode_inactive").to_ratatui()
}

/// Style for mode tab hover
pub fn mode_hover() -> Style {
    theme::current().style("mode_hover").to_ratatui()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Status Indicator Styles
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
// Heat Map Colors
// ═══════════════════════════════════════════════════════════════════════════════

/// Get heat map color based on change frequency (0.0 = cold, 1.0 = hot)
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions
)]
pub fn heat_color(frequency: f32) -> Color {
    let frequency = frequency.clamp(0.0, 1.0);
    let t = theme::current();

    // Get colors from theme
    let cold = t.color("text.dim");
    let warm = t.color("accent.tertiary");
    let hot = t.color("error");

    if frequency < 0.5 {
        let interp = frequency * 2.0;
        cold.lerp(&warm, interp).to_ratatui()
    } else {
        let interp = (frequency - 0.5) * 2.0;
        warm.lerp(&hot, interp).to_ratatui()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diff Colors
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
// Spinner Characters
// ═══════════════════════════════════════════════════════════════════════════════

/// Braille spinner frames
pub const SPINNER_BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Standard spinner frames
pub const SPINNER_STANDARD: &[char] = &['✦', '✧', '✶', '✷', '✸', '✹', '✺', '✻', '✼', '✽'];

// ═══════════════════════════════════════════════════════════════════════════════
// Box Drawing Characters
// ═══════════════════════════════════════════════════════════════════════════════

/// Thin horizontal line
pub const LINE_THIN: char = '─';

/// Thick horizontal line
pub const LINE_THICK: char = '━';

/// Vertical line
pub const LINE_VERTICAL: char = '│';

/// Corner top-left
pub const CORNER_TL: char = '┌';

/// Corner top-right
pub const CORNER_TR: char = '┐';

/// Corner bottom-left
pub const CORNER_BL: char = '└';

/// Corner bottom-right
pub const CORNER_BR: char = '┘';

// ═══════════════════════════════════════════════════════════════════════════════
// Title Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// Format the Iris Studio title with appropriate styling
pub fn studio_title() -> &'static str {
    "Iris Studio"
}

/// Format a mode indicator
pub fn format_mode_indicator(_name: &str, active: bool) -> Style {
    if active {
        mode_active()
    } else {
        mode_inactive()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Gradient & Effect Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Get a color for a gradient position (0.0 = start, 1.0 = end)
/// Gradient goes from `ELECTRIC_PURPLE` → `NEON_CYAN`
pub fn gradient_purple_cyan(position: f32) -> Color {
    theme::current().gradient("primary", position).to_ratatui()
}

/// Get a color for a gradient position (0.0 = start, 1.0 = end)
/// Gradient goes from CORAL → `ELECTRIC_YELLOW`
pub fn gradient_coral_yellow(position: f32) -> Color {
    theme::current().gradient("warm", position).to_ratatui()
}

/// Unicode block characters for drawing gradient bars
pub const BLOCK_FULL: char = '█';
pub const BLOCK_3_4: char = '▓';
pub const BLOCK_1_2: char = '▒';
pub const BLOCK_1_4: char = '░';

/// Thin gradient line characters
pub const GRADIENT_LINE: &[char] = &['━', '━', '━', '━'];

/// Create styled text with gradient coloring
#[allow(clippy::as_conversions, clippy::cast_precision_loss)]
pub fn styled_gradient_text(text: &str, gradient_fn: fn(f32) -> Color) -> Vec<Span<'static>> {
    let len = text.chars().count().max(1);
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            let position = i as f32 / (len - 1).max(1) as f32;
            Span::styled(c.to_string(), Style::default().fg(gradient_fn(position)))
        })
        .collect()
}

/// Create a horizontal gradient line using theme gradient
#[allow(clippy::as_conversions, clippy::cast_precision_loss)]
pub fn gradient_line(width: usize) -> Vec<Span<'static>> {
    if let Some(gradient) = theme::current().get_gradient("primary") {
        theme_gradient_line(width, LINE_THIN, gradient)
    } else {
        // Fallback
        vec![Span::raw(LINE_THIN.to_string().repeat(width))]
    }
}

/// Create a thick horizontal gradient line
#[allow(clippy::as_conversions, clippy::cast_precision_loss)]
pub fn gradient_line_thick(width: usize) -> Vec<Span<'static>> {
    if let Some(gradient) = theme::current().get_gradient("primary") {
        theme_gradient_line(width, LINE_THICK, gradient)
    } else {
        // Fallback
        vec![Span::raw(LINE_THICK.to_string().repeat(width))]
    }
}
