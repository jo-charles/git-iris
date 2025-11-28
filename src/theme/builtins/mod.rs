//! Builtin themes embedded in the binary.

use super::Theme;
use super::loader::load_from_str;

/// The embedded `SilkCircuit` Neon theme TOML.
const SILKCIRCUIT_NEON_TOML: &str = include_str!("silkcircuit_neon.toml");

/// The embedded `SilkCircuit` Soft theme TOML.
const SILKCIRCUIT_SOFT_TOML: &str = include_str!("silkcircuit_soft.toml");

/// The embedded `SilkCircuit` Glow theme TOML.
const SILKCIRCUIT_GLOW_TOML: &str = include_str!("silkcircuit_glow.toml");

/// The embedded `SilkCircuit` Vibrant theme TOML.
const SILKCIRCUIT_VIBRANT_TOML: &str = include_str!("silkcircuit_vibrant.toml");

/// The embedded `SilkCircuit` Dawn theme TOML.
const SILKCIRCUIT_DAWN_TOML: &str = include_str!("silkcircuit_dawn.toml");

/// Load the builtin `SilkCircuit` Neon theme.
///
/// # Panics
/// Panics if the embedded theme TOML is invalid (this would be a bug).
#[must_use]
pub fn silkcircuit_neon() -> Theme {
    load_from_str(SILKCIRCUIT_NEON_TOML, None)
        .expect("builtin SilkCircuit Neon theme should be valid")
}

/// Load the builtin `SilkCircuit` Soft theme.
///
/// # Panics
/// Panics if the embedded theme TOML is invalid (this would be a bug).
#[must_use]
pub fn silkcircuit_soft() -> Theme {
    load_from_str(SILKCIRCUIT_SOFT_TOML, None)
        .expect("builtin SilkCircuit Soft theme should be valid")
}

/// Load the builtin `SilkCircuit` Glow theme.
///
/// # Panics
/// Panics if the embedded theme TOML is invalid (this would be a bug).
#[must_use]
pub fn silkcircuit_glow() -> Theme {
    load_from_str(SILKCIRCUIT_GLOW_TOML, None)
        .expect("builtin SilkCircuit Glow theme should be valid")
}

/// Load the builtin `SilkCircuit` Vibrant theme.
///
/// # Panics
/// Panics if the embedded theme TOML is invalid (this would be a bug).
#[must_use]
pub fn silkcircuit_vibrant() -> Theme {
    load_from_str(SILKCIRCUIT_VIBRANT_TOML, None)
        .expect("builtin SilkCircuit Vibrant theme should be valid")
}

/// Load the builtin `SilkCircuit` Dawn theme.
///
/// # Panics
/// Panics if the embedded theme TOML is invalid (this would be a bug).
#[must_use]
pub fn silkcircuit_dawn() -> Theme {
    load_from_str(SILKCIRCUIT_DAWN_TOML, None)
        .expect("builtin SilkCircuit Dawn theme should be valid")
}

/// Get all builtin theme names (for theme listings).
#[must_use]
pub fn builtin_names() -> &'static [(&'static str, &'static str)] {
    &[
        ("silkcircuit-neon", "SilkCircuit Neon"),
        ("silkcircuit-soft", "SilkCircuit Soft"),
        ("silkcircuit-glow", "SilkCircuit Glow"),
        ("silkcircuit-vibrant", "SilkCircuit Vibrant"),
        ("silkcircuit-dawn", "SilkCircuit Dawn"),
    ]
}

/// Load a builtin theme by name.
///
/// Returns `None` if the name is not a builtin theme.
#[must_use]
pub fn load_by_name(name: &str) -> Option<Theme> {
    match name {
        "silkcircuit-neon" | "default" => Some(silkcircuit_neon()),
        "silkcircuit-soft" => Some(silkcircuit_soft()),
        "silkcircuit-glow" => Some(silkcircuit_glow()),
        "silkcircuit-vibrant" => Some(silkcircuit_vibrant()),
        "silkcircuit-dawn" => Some(silkcircuit_dawn()),
        _ => None,
    }
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

    #[test]
    fn test_silkcircuit_soft_loads() {
        let theme = silkcircuit_soft();
        assert_eq!(theme.meta.name, "SilkCircuit Soft");
    }

    #[test]
    fn test_silkcircuit_glow_loads() {
        let theme = silkcircuit_glow();
        assert_eq!(theme.meta.name, "SilkCircuit Glow");
    }

    #[test]
    fn test_silkcircuit_vibrant_loads() {
        let theme = silkcircuit_vibrant();
        assert_eq!(theme.meta.name, "SilkCircuit Vibrant");
    }

    #[test]
    fn test_silkcircuit_dawn_loads() {
        let theme = silkcircuit_dawn();
        assert_eq!(theme.meta.name, "SilkCircuit Dawn");
        assert_eq!(
            theme.meta.variant,
            crate::theme::schema::ThemeVariant::Light
        );
    }

    #[test]
    fn test_load_by_name() {
        assert!(load_by_name("silkcircuit-neon").is_some());
        assert!(load_by_name("default").is_some());
        assert!(load_by_name("silkcircuit-soft").is_some());
        assert!(load_by_name("silkcircuit-glow").is_some());
        assert!(load_by_name("silkcircuit-vibrant").is_some());
        assert!(load_by_name("silkcircuit-dawn").is_some());
        assert!(load_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_builtin_names() {
        let names = builtin_names();
        assert_eq!(names.len(), 5);
        assert!(names.iter().any(|(n, _)| *n == "silkcircuit-neon"));
        assert!(names.iter().any(|(n, _)| *n == "silkcircuit-dawn"));
    }
}
