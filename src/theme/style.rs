//! Theme style type - combines color with text modifiers.

use super::color::ThemeColor;

/// A composed style with foreground, background, and modifiers.
///
/// This represents a complete text style that can be applied to UI elements.
/// Adapters convert this to framework-specific styles (Ratatui Style, etc.).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThemeStyle {
    /// Foreground (text) color.
    pub fg: Option<ThemeColor>,
    /// Background color.
    pub bg: Option<ThemeColor>,
    /// Bold text modifier.
    pub bold: bool,
    /// Italic text modifier.
    pub italic: bool,
    /// Underline text modifier.
    pub underline: bool,
    /// Dim/faint text modifier.
    pub dim: bool,
}

impl ThemeStyle {
    /// Create a new empty style.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }

    /// Create a style with only a foreground color.
    #[must_use]
    pub const fn fg(color: ThemeColor) -> Self {
        Self {
            fg: Some(color),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }

    /// Create a style with only a background color.
    #[must_use]
    pub const fn bg(color: ThemeColor) -> Self {
        Self {
            fg: None,
            bg: Some(color),
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }

    /// Set the foreground color.
    #[must_use]
    pub const fn with_fg(mut self, color: ThemeColor) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set the background color.
    #[must_use]
    pub const fn with_bg(mut self, color: ThemeColor) -> Self {
        self.bg = Some(color);
        self
    }

    /// Enable bold modifier.
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Enable italic modifier.
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Enable underline modifier.
    #[must_use]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Enable dim modifier.
    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Merge another style onto this one.
    /// Values from `other` take precedence where set.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            bold: other.bold || self.bold,
            italic: other.italic || self.italic,
            underline: other.underline || self.underline,
            dim: other.dim || self.dim,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_builder() {
        let style = ThemeStyle::fg(ThemeColor::new(255, 0, 0))
            .with_bg(ThemeColor::new(0, 0, 0))
            .bold();

        assert_eq!(style.fg, Some(ThemeColor::new(255, 0, 0)));
        assert_eq!(style.bg, Some(ThemeColor::new(0, 0, 0)));
        assert!(style.bold);
        assert!(!style.italic);
    }

    #[test]
    fn test_style_merge() {
        let base = ThemeStyle::fg(ThemeColor::new(255, 0, 0)).bold();
        let overlay = ThemeStyle::bg(ThemeColor::new(0, 0, 0)).italic();

        let merged = base.merge(&overlay);
        assert_eq!(merged.fg, Some(ThemeColor::new(255, 0, 0)));
        assert_eq!(merged.bg, Some(ThemeColor::new(0, 0, 0)));
        assert!(merged.bold);
        assert!(merged.italic);
    }
}
