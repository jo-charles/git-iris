//! `SilkCircuit` Neon theme for Iris Studio
//!
//! Electric meets elegant - the visual identity for git-iris TUI.

use ratatui::style::{Color, Modifier, Style};

// ═══════════════════════════════════════════════════════════════════════════════
// Core Palette
// ═══════════════════════════════════════════════════════════════════════════════

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
// Backgrounds
// ═══════════════════════════════════════════════════════════════════════════════

/// Dark background base
pub const BG_DARK: Color = Color::Rgb(22, 22, 30);

/// Panel background
pub const BG_PANEL: Color = Color::Rgb(30, 30, 40);

/// Highlighted/selected background
pub const BG_HIGHLIGHT: Color = Color::Rgb(40, 42, 54);

/// Elevated surface
pub const BG_ELEVATED: Color = Color::Rgb(50, 52, 64);

// ═══════════════════════════════════════════════════════════════════════════════
// Text Colors
// ═══════════════════════════════════════════════════════════════════════════════

/// Primary text - soft white
pub const TEXT_PRIMARY: Color = Color::Rgb(248, 248, 242);

/// Secondary/dimmed text
pub const TEXT_DIM: Color = Color::Rgb(98, 114, 164);

/// Muted text for borders and less important elements
pub const TEXT_MUTED: Color = Color::Rgb(68, 71, 90);

// ═══════════════════════════════════════════════════════════════════════════════
// Semantic Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for commit hashes
pub fn commit_hash() -> Style {
    Style::default().fg(CORAL)
}

/// Style for file paths
pub fn file_path() -> Style {
    Style::default().fg(NEON_CYAN)
}

/// Style for file paths with bold
pub fn file_path_bold() -> Style {
    Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD)
}

/// Style for keywords and important markers
pub fn keyword() -> Style {
    Style::default().fg(ELECTRIC_PURPLE)
}

/// Style for line numbers
pub fn line_number() -> Style {
    Style::default().fg(TEXT_DIM)
}

/// Style for the current/cursor line
pub fn cursor_line() -> Style {
    Style::default().bg(BG_HIGHLIGHT)
}

/// Style for selected items
pub fn selected() -> Style {
    Style::default().bg(BG_HIGHLIGHT).fg(NEON_CYAN)
}

/// Style for focused panel border
pub fn focused_border() -> Style {
    Style::default().fg(NEON_CYAN)
}

/// Style for unfocused panel border
pub fn unfocused_border() -> Style {
    Style::default().fg(TEXT_MUTED)
}

/// Style for success messages
pub fn success() -> Style {
    Style::default().fg(SUCCESS_GREEN)
}

/// Style for error messages
pub fn error() -> Style {
    Style::default().fg(ERROR_RED)
}

/// Style for warning messages
pub fn warning() -> Style {
    Style::default().fg(ELECTRIC_YELLOW)
}

/// Style for timestamps
pub fn timestamp() -> Style {
    Style::default().fg(ELECTRIC_YELLOW)
}

/// Style for author names
pub fn author() -> Style {
    Style::default().fg(TEXT_PRIMARY)
}

/// Style for dimmed/secondary text
pub fn dimmed() -> Style {
    Style::default().fg(TEXT_DIM)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mode Tab Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for active mode tab
pub fn mode_active() -> Style {
    Style::default()
        .fg(ELECTRIC_PURPLE)
        .add_modifier(Modifier::BOLD)
}

/// Style for inactive mode tab
pub fn mode_inactive() -> Style {
    Style::default().fg(TEXT_DIM)
}

/// Style for mode tab hover
pub fn mode_hover() -> Style {
    Style::default().fg(NEON_CYAN)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Status Indicator Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for git staged files
pub fn git_staged() -> Style {
    Style::default().fg(SUCCESS_GREEN)
}

/// Style for git modified files
pub fn git_modified() -> Style {
    Style::default().fg(ELECTRIC_YELLOW)
}

/// Style for git untracked files
pub fn git_untracked() -> Style {
    Style::default().fg(TEXT_DIM)
}

/// Style for git deleted files
pub fn git_deleted() -> Style {
    Style::default().fg(ERROR_RED)
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

    // Cold: TEXT_DIM (98, 114, 164)
    // Warm: CORAL (255, 106, 193)
    // Hot: ERROR_RED (255, 99, 99)

    if frequency < 0.5 {
        let t = frequency * 2.0;
        Color::Rgb(
            (98.0 + (255.0 - 98.0) * t) as u8,
            (114.0 + (106.0 - 114.0) * t) as u8,
            (164.0 + (193.0 - 164.0) * t) as u8,
        )
    } else {
        let t = (frequency - 0.5) * 2.0;
        Color::Rgb(
            255,
            (106.0 + (99.0 - 106.0) * t) as u8,
            (193.0 + (99.0 - 193.0) * t) as u8,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diff Colors
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for added lines in diff
pub fn diff_added() -> Style {
    Style::default().fg(SUCCESS_GREEN)
}

/// Style for removed lines in diff
pub fn diff_removed() -> Style {
    Style::default().fg(ERROR_RED)
}

/// Style for diff hunk headers
pub fn diff_hunk() -> Style {
    Style::default().fg(NEON_CYAN)
}

/// Style for diff context lines
pub fn diff_context() -> Style {
    Style::default().fg(TEXT_DIM)
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
