//! Builtin themes embedded in the binary.

use super::Theme;
use super::loader::load_from_str;

/// The embedded `SilkCircuit` Neon theme TOML.
const SILKCIRCUIT_NEON_TOML: &str = include_str!("silkcircuit_neon.toml");

/// Load the builtin `SilkCircuit` Neon theme.
///
/// # Panics
/// Panics if the embedded theme TOML is invalid (this would be a bug).
#[must_use]
pub fn silkcircuit_neon() -> Theme {
    load_from_str(SILKCIRCUIT_NEON_TOML, None)
        .expect("builtin SilkCircuit Neon theme should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeColor;

    #[test]
    fn test_silkcircuit_neon_loads() {
        let theme = silkcircuit_neon();
        assert_eq!(theme.meta.name, "SilkCircuit Neon");
    }

    #[test]
    fn test_silkcircuit_neon_colors() {
        let theme = silkcircuit_neon();

        // Test core palette
        assert_eq!(theme.color("purple_500"), ThemeColor::new(225, 53, 255));
        assert_eq!(theme.color("cyan_400"), ThemeColor::new(128, 255, 234));

        // Test tokens
        assert_eq!(theme.color("accent.primary"), ThemeColor::new(225, 53, 255));
        assert_eq!(theme.color("success"), ThemeColor::new(80, 250, 123));
    }

    #[test]
    fn test_silkcircuit_neon_styles() {
        let theme = silkcircuit_neon();

        let keyword = theme.style("keyword");
        assert_eq!(keyword.fg, Some(ThemeColor::new(225, 53, 255)));
        assert!(keyword.bold);
    }

    #[test]
    fn test_silkcircuit_neon_gradients() {
        let theme = silkcircuit_neon();

        // Primary gradient: purple → cyan
        let start = theme.gradient("primary", 0.0);
        let end = theme.gradient("primary", 1.0);

        assert_eq!(start, ThemeColor::new(225, 53, 255));
        assert_eq!(end, ThemeColor::new(128, 255, 234));
    }
}
