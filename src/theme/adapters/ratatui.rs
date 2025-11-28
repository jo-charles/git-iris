//! Ratatui adapter for theme types.
//!
//! Provides conversion from theme types to Ratatui types for TUI rendering.

use ratatui::style::{Color, Modifier, Style};

use crate::theme::{Gradient, ThemeColor, ThemeStyle};

/// Convert a `ThemeColor` to a Ratatui `Color`.
pub trait ToRatatuiColor {
    /// Convert to Ratatui Color.
    fn to_ratatui(&self) -> Color;
}

impl ToRatatuiColor for ThemeColor {
    fn to_ratatui(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

/// Convert a `ThemeStyle` to a Ratatui `Style`.
pub trait ToRatatuiStyle {
    /// Convert to Ratatui Style.
    fn to_ratatui(&self) -> Style;
}

impl ToRatatuiStyle for ThemeStyle {
    fn to_ratatui(&self) -> Style {
        let mut style = Style::default();

        if let Some(fg) = self.fg {
            style = style.fg(fg.to_ratatui());
        }

        if let Some(bg) = self.bg {
            style = style.bg(bg.to_ratatui());
        }

        let mut modifiers = Modifier::empty();
        if self.bold {
            modifiers |= Modifier::BOLD;
        }
        if self.italic {
            modifiers |= Modifier::ITALIC;
        }
        if self.underline {
            modifiers |= Modifier::UNDERLINED;
        }
        if self.dim {
            modifiers |= Modifier::DIM;
        }

        if !modifiers.is_empty() {
            style = style.add_modifier(modifiers);
        }

        style
    }
}

/// Extension trait for easy access to theme colors as Ratatui Colors.
pub trait ThemeColorExt {
    /// Get a token color as Ratatui Color.
    fn ratatui_color(&self, token: &str) -> Color;

    /// Get a token style as Ratatui Style.
    fn ratatui_style(&self, name: &str) -> Style;

    /// Get a gradient color as Ratatui Color.
    fn ratatui_gradient(&self, name: &str, t: f32) -> Color;
}

impl ThemeColorExt for crate::theme::Theme {
    fn ratatui_color(&self, token: &str) -> Color {
        self.color(token).to_ratatui()
    }

    fn ratatui_style(&self, name: &str) -> Style {
        self.style(name).to_ratatui()
    }

    fn ratatui_gradient(&self, name: &str, t: f32) -> Color {
        self.gradient(name, t).to_ratatui()
    }
}

/// Generate styled spans for gradient text.
///
/// Creates a vector of single-character spans, each colored according to
/// its position in the gradient.
#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
pub fn gradient_spans(text: &str, gradient: &Gradient) -> Vec<ratatui::text::Span<'static>> {
    let len = text.chars().count().max(1);
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            let t = if len == 1 {
                0.0
            } else {
                i as f32 / (len - 1) as f32
            };
            let color = gradient.at(t).to_ratatui();
            ratatui::text::Span::styled(c.to_string(), Style::default().fg(color))
        })
        .collect()
}

/// Generate a gradient line of a given character.
#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
pub fn gradient_line(
    width: usize,
    ch: char,
    gradient: &Gradient,
) -> Vec<ratatui::text::Span<'static>> {
    (0..width)
        .map(|i| {
            let t = if width <= 1 {
                0.0
            } else {
                i as f32 / (width - 1) as f32
            };
            let color = gradient.at(t).to_ratatui();
            ratatui::text::Span::styled(ch.to_string(), Style::default().fg(color))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeColor;

    #[test]
    fn test_color_conversion() {
        let theme_color = ThemeColor::new(225, 53, 255);
        let ratatui_color = theme_color.to_ratatui();
        assert_eq!(ratatui_color, Color::Rgb(225, 53, 255));
    }

    #[test]
    fn test_style_conversion() {
        let theme_style = ThemeStyle::fg(ThemeColor::new(255, 0, 0))
            .with_bg(ThemeColor::new(0, 0, 0))
            .bold()
            .italic();

        let ratatui_style = theme_style.to_ratatui();

        assert_eq!(ratatui_style.fg, Some(Color::Rgb(255, 0, 0)));
        assert_eq!(ratatui_style.bg, Some(Color::Rgb(0, 0, 0)));
        assert!(ratatui_style.add_modifier.contains(Modifier::BOLD));
        assert!(ratatui_style.add_modifier.contains(Modifier::ITALIC));
    }
}
