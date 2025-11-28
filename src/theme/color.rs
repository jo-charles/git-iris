//! Theme color type - format-agnostic RGB color representation.

use std::fmt;
use std::str::FromStr;

/// A theme color in RGB format.
///
/// This is the canonical color representation used throughout the theme system.
/// Adapters convert this to framework-specific types (Ratatui Color, colored tuples).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    /// Create a new theme color from RGB values.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a theme color from a hex string (with or without #).
    ///
    /// # Errors
    /// Returns an error if the hex string is invalid.
    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(ColorParseError::InvalidLength(hex.len()));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;

        Ok(Self { r, g, b })
    }

    /// Convert to hex string with # prefix.
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to RGB tuple for use with colored crate.
    #[must_use]
    pub const fn to_rgb_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Linearly interpolate between two colors.
    ///
    /// `t` should be in range 0.0..=1.0 where 0.0 returns `self` and 1.0 returns `other`.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions
    )]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: (f32::from(self.r) + (f32::from(other.r) - f32::from(self.r)) * t) as u8,
            g: (f32::from(self.g) + (f32::from(other.g) - f32::from(self.g)) * t) as u8,
            b: (f32::from(self.b) + (f32::from(other.b) - f32::from(self.b)) * t) as u8,
        }
    }

    /// The fallback color used when a token cannot be resolved.
    /// A neutral gray that works on both light and dark backgrounds.
    pub const FALLBACK: Self = Self::new(128, 128, 128);
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self::FALLBACK
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for ThemeColor {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

/// Errors that can occur when parsing a color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    /// Hex string has wrong length (expected 6 characters without #).
    InvalidLength(usize),
    /// Hex string contains invalid characters.
    InvalidHex(String),
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(len) => {
                write!(f, "invalid hex color length: {len} (expected 6)")
            }
            Self::InvalidHex(hex) => {
                write!(f, "invalid hex color: {hex}")
            }
        }
    }
}

impl std::error::Error for ColorParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(
            ThemeColor::from_hex("#e135ff").unwrap(),
            ThemeColor::new(225, 53, 255)
        );
        assert_eq!(
            ThemeColor::from_hex("80ffea").unwrap(),
            ThemeColor::new(128, 255, 234)
        );
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(ThemeColor::new(225, 53, 255).to_hex(), "#e135ff");
    }

    #[test]
    fn test_lerp() {
        let black = ThemeColor::new(0, 0, 0);
        let white = ThemeColor::new(255, 255, 255);
        let mid = black.lerp(&white, 0.5);
        assert_eq!(mid, ThemeColor::new(127, 127, 127));
    }
}
