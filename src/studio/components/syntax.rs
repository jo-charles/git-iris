//! Syntax highlighting for code view using `SilkCircuit` colors
//!
//! Maps syntect token types to our theme colors for a cohesive look.

use ratatui::style::{Color, Modifier, Style};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style as SyntectStyle, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::studio::theme;

/// Global syntax set - loaded once
static SYNTAX_SET: std::sync::LazyLock<SyntaxSet> =
    std::sync::LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Global theme set - load default themes for syntax highlighting
static THEME_SET: std::sync::LazyLock<ThemeSet> = std::sync::LazyLock::new(ThemeSet::load_defaults);

/// Syntax highlighter with caching
pub struct SyntaxHighlighter {
    syntax: Option<&'static SyntaxReference>,
}

impl SyntaxHighlighter {
    /// Create a new highlighter for the given file extension
    pub fn for_extension(ext: &str) -> Self {
        let syntax = SYNTAX_SET.find_syntax_by_extension(ext);
        Self { syntax }
    }

    /// Create a new highlighter for the given file path
    pub fn for_path(path: &std::path::Path) -> Self {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Self::for_extension(ext)
    }

    /// Check if syntax highlighting is available
    pub fn is_available(&self) -> bool {
        self.syntax.is_some()
    }

    /// Highlight a single line, returning styled spans
    pub fn highlight_line(&self, line: &str) -> Vec<(Style, String)> {
        let Some(syntax) = self.syntax else {
            // No syntax highlighting - return plain
            return vec![(Style::default().fg(theme::TEXT_PRIMARY), line.to_string())];
        };

        // Try to get a dark theme, fallback to any available theme, or give up
        let Some(theme) = THEME_SET
            .themes
            .get("base16-ocean.dark")
            .or_else(|| THEME_SET.themes.get("InspiredGitHub"))
            .or_else(|| THEME_SET.themes.values().next())
        else {
            // No themes available - return plain text
            return vec![(Style::default().fg(theme::TEXT_PRIMARY), line.to_string())];
        };

        let mut highlighter = HighlightLines::new(syntax, theme);

        match highlighter.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, text)| (syntect_to_ratatui(style), text.to_string()))
                .collect(),
            Err(_) => vec![(Style::default().fg(theme::TEXT_PRIMARY), line.to_string())],
        }
    }

    /// Highlight multiple lines
    pub fn highlight_lines(&self, lines: &[String]) -> Vec<Vec<(Style, String)>> {
        lines.iter().map(|line| self.highlight_line(line)).collect()
    }
}

/// Convert syntect style to ratatui style with `SilkCircuit` color mapping
fn syntect_to_ratatui(style: SyntectStyle) -> Style {
    let fg = syntect_color_to_silkcircuit(style.foreground);
    let mut ratatui_style = Style::default().fg(fg);

    if style.font_style.contains(FontStyle::BOLD) {
        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
    }

    ratatui_style
}

/// Map syntect colors to `SilkCircuit` palette
/// This creates a cohesive look by mapping token colors to our theme
fn syntect_color_to_silkcircuit(color: syntect::highlighting::Color) -> Color {
    // Extract RGB values
    let r = color.r;
    let g = color.g;
    let b = color.b;

    // Map common syntax highlighting colors to SilkCircuit palette
    // We analyze the color characteristics and map to our theme

    // Very bright/saturated colors -> map to our accent colors
    let saturation = color_saturation(r, g, b);
    let luminance = color_luminance(r, g, b);

    // Keywords, control flow (often purple/magenta in themes)
    if is_purple_ish(r, g, b) {
        return theme::ELECTRIC_PURPLE;
    }

    // Strings (often green/teal)
    if is_green_ish(r, g, b) && saturation > 0.3 {
        return theme::SUCCESS_GREEN;
    }

    // Numbers, constants (often orange/coral)
    if is_orange_ish(r, g, b) {
        return theme::CORAL;
    }

    // Functions, methods (often cyan/blue)
    if is_cyan_ish(r, g, b) {
        return theme::NEON_CYAN;
    }

    // Types, classes (often yellow)
    if is_yellow_ish(r, g, b) {
        return theme::ELECTRIC_YELLOW;
    }

    // Comments (usually gray/dim)
    if saturation < 0.15 && luminance < 0.6 {
        return theme::TEXT_MUTED;
    }

    // Default: use original color if it's reasonably visible
    if luminance > 0.2 {
        Color::Rgb(r, g, b)
    } else {
        theme::TEXT_SECONDARY
    }
}

// Color analysis helpers

fn color_saturation(r: u8, g: u8, b: u8) -> f32 {
    let max = f32::from(r.max(g).max(b));
    let min = f32::from(r.min(g).min(b));
    if max == 0.0 { 0.0 } else { (max - min) / max }
}

fn color_luminance(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b)) / 255.0
}

fn is_purple_ish(r: u8, g: u8, b: u8) -> bool {
    // Purple: high red, low green, high blue
    r > 150 && g < 150 && b > 150
}

fn is_green_ish(r: u8, g: u8, b: u8) -> bool {
    // Green: low red, high green, variable blue
    g > r && g > b && g > 120
}

fn is_orange_ish(r: u8, g: u8, b: u8) -> bool {
    // Orange/coral: high red, medium green, low blue
    r > 180 && g > 80 && g < 180 && b < 150
}

fn is_cyan_ish(r: u8, g: u8, b: u8) -> bool {
    // Cyan: low red, high green, high blue
    r < 150 && g > 150 && b > 150
}

fn is_yellow_ish(r: u8, g: u8, b: u8) -> bool {
    // Yellow: high red, high green, low blue
    r > 180 && g > 180 && b < 150
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_rust() {
        let highlighter = SyntaxHighlighter::for_extension("rs");
        assert!(highlighter.is_available());

        let spans = highlighter.highlight_line("fn main() { }");
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlighter_unknown() {
        let highlighter = SyntaxHighlighter::for_extension("xyz_unknown");
        assert!(!highlighter.is_available());
    }
}
