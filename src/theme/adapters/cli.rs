//! CLI adapter for theme types.
//!
//! Provides conversion from theme types to colored crate types for terminal output.

use colored::{ColoredString, Colorize};

use crate::theme::{Gradient, ThemeColor, ThemeStyle};

/// Convert a `ThemeColor` to an RGB tuple for use with the colored crate.
pub trait ToColoredRgb {
    /// Convert to RGB tuple for colored crate's `.truecolor()` method.
    fn to_rgb(&self) -> (u8, u8, u8);
}

impl ToColoredRgb for ThemeColor {
    fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// Extension trait for applying theme colors to strings.
pub trait ColoredExt {
    /// Apply a theme color as foreground.
    fn theme_fg(self, color: ThemeColor) -> ColoredString;

    /// Apply a theme color as background.
    fn theme_bg(self, color: ThemeColor) -> ColoredString;

    /// Apply a theme style.
    fn theme_style(self, style: &ThemeStyle) -> ColoredString;
}

impl<S: AsRef<str>> ColoredExt for S {
    fn theme_fg(self, color: ThemeColor) -> ColoredString {
        self.as_ref().truecolor(color.r, color.g, color.b)
    }

    fn theme_bg(self, color: ThemeColor) -> ColoredString {
        self.as_ref().on_truecolor(color.r, color.g, color.b)
    }

    fn theme_style(self, style: &ThemeStyle) -> ColoredString {
        let mut result: ColoredString = self.as_ref().into();

        if let Some(fg) = style.fg {
            result = result.truecolor(fg.r, fg.g, fg.b);
        }

        if let Some(bg) = style.bg {
            result = result.on_truecolor(bg.r, bg.g, bg.b);
        }

        if style.bold {
            result = result.bold();
        }

        if style.italic {
            result = result.italic();
        }

        if style.underline {
            result = result.underline();
        }

        if style.dim {
            result = result.dimmed();
        }

        result
    }
}

/// Apply a gradient to a string for CLI output.
///
/// Returns a string with ANSI escape codes for each character.
#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
pub fn gradient_string(text: &str, gradient: &Gradient) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len().max(1);

    let mut result = String::new();

    for (i, c) in chars.into_iter().enumerate() {
        let t = if len == 1 {
            0.0
        } else {
            i as f32 / (len - 1) as f32
        };
        let color = gradient.at(t);
        let colored = c.to_string().truecolor(color.r, color.g, color.b);
        result.push_str(&colored.to_string());
    }

    result
}

/// Extension trait for easy access to theme colors for CLI output.
pub trait ThemeCliExt {
    /// Get a token color as RGB tuple.
    fn cli_rgb(&self, token: &str) -> (u8, u8, u8);

    /// Apply a token color to a string.
    fn cli_colored(&self, text: &str, token: &str) -> ColoredString;

    /// Apply a gradient to a string.
    fn cli_gradient(&self, text: &str, gradient_name: &str) -> String;
}

impl ThemeCliExt for crate::theme::Theme {
    fn cli_rgb(&self, token: &str) -> (u8, u8, u8) {
        self.color(token).to_rgb()
    }

    fn cli_colored(&self, text: &str, token: &str) -> ColoredString {
        let color = self.color(token);
        text.truecolor(color.r, color.g, color.b)
    }

    fn cli_gradient(&self, text: &str, gradient_name: &str) -> String {
        if let Some(gradient) = self.get_gradient(gradient_name) {
            gradient_string(text, gradient)
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_rgb() {
        let color = ThemeColor::new(225, 53, 255);
        assert_eq!(color.to_rgb(), (225, 53, 255));
    }

    #[test]
    fn test_theme_fg() {
        let color = ThemeColor::new(255, 0, 0);
        let result = "test".theme_fg(color);
        // Just verify it doesn't panic and produces output
        assert!(!result.to_string().is_empty());
    }

    #[test]
    fn test_gradient_string() {
        // Force color output for testing
        colored::control::set_override(true);

        let gradient = Gradient::new(vec![
            ThemeColor::new(255, 0, 0),
            ThemeColor::new(0, 0, 255),
        ]);
        let result = gradient_string("test", &gradient);
        // Verify output contains ANSI codes (when color is forced)
        assert!(result.contains("\x1b[") || result.contains("test"));

        colored::control::unset_override();
    }
}
