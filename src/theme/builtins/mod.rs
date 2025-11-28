//! Builtin themes embedded in the binary.

use super::Theme;
use super::loader::load_from_str;

// SilkCircuit family
const SILKCIRCUIT_NEON_TOML: &str = include_str!("silkcircuit_neon.toml");
const SILKCIRCUIT_SOFT_TOML: &str = include_str!("silkcircuit_soft.toml");
const SILKCIRCUIT_GLOW_TOML: &str = include_str!("silkcircuit_glow.toml");
const SILKCIRCUIT_VIBRANT_TOML: &str = include_str!("silkcircuit_vibrant.toml");
const SILKCIRCUIT_DAWN_TOML: &str = include_str!("silkcircuit_dawn.toml");

// Popular dark themes
const CATPPUCCIN_MOCHA_TOML: &str = include_str!("catppuccin_mocha.toml");
const DRACULA_TOML: &str = include_str!("dracula.toml");
const NORD_TOML: &str = include_str!("nord.toml");
const TOKYO_NIGHT_TOML: &str = include_str!("tokyo_night.toml");
const GRUVBOX_DARK_TOML: &str = include_str!("gruvbox_dark.toml");
const ONE_DARK_TOML: &str = include_str!("one_dark.toml");

// Popular light themes
const CATPPUCCIN_LATTE_TOML: &str = include_str!("catppuccin_latte.toml");
const SOLARIZED_LIGHT_TOML: &str = include_str!("solarized_light.toml");

// ═══════════════════════════════════════════════════════════════════════════════
// SilkCircuit Family
// ═══════════════════════════════════════════════════════════════════════════════

/// Load the builtin `SilkCircuit` Neon theme (default).
#[must_use]
pub fn silkcircuit_neon() -> Theme {
    load_from_str(SILKCIRCUIT_NEON_TOML, None)
        .expect("builtin SilkCircuit Neon theme should be valid")
}

/// Load the builtin `SilkCircuit` Soft theme.
#[must_use]
pub fn silkcircuit_soft() -> Theme {
    load_from_str(SILKCIRCUIT_SOFT_TOML, None)
        .expect("builtin SilkCircuit Soft theme should be valid")
}

/// Load the builtin `SilkCircuit` Glow theme.
#[must_use]
pub fn silkcircuit_glow() -> Theme {
    load_from_str(SILKCIRCUIT_GLOW_TOML, None)
        .expect("builtin SilkCircuit Glow theme should be valid")
}

/// Load the builtin `SilkCircuit` Vibrant theme.
#[must_use]
pub fn silkcircuit_vibrant() -> Theme {
    load_from_str(SILKCIRCUIT_VIBRANT_TOML, None)
        .expect("builtin SilkCircuit Vibrant theme should be valid")
}

/// Load the builtin `SilkCircuit` Dawn theme.
#[must_use]
pub fn silkcircuit_dawn() -> Theme {
    load_from_str(SILKCIRCUIT_DAWN_TOML, None)
        .expect("builtin SilkCircuit Dawn theme should be valid")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Popular Dark Themes
// ═══════════════════════════════════════════════════════════════════════════════

/// Load the Catppuccin Mocha theme.
#[must_use]
pub fn catppuccin_mocha() -> Theme {
    load_from_str(CATPPUCCIN_MOCHA_TOML, None)
        .expect("builtin Catppuccin Mocha theme should be valid")
}

/// Load the Dracula theme.
#[must_use]
pub fn dracula() -> Theme {
    load_from_str(DRACULA_TOML, None).expect("builtin Dracula theme should be valid")
}

/// Load the Nord theme.
#[must_use]
pub fn nord() -> Theme {
    load_from_str(NORD_TOML, None).expect("builtin Nord theme should be valid")
}

/// Load the Tokyo Night theme.
#[must_use]
pub fn tokyo_night() -> Theme {
    load_from_str(TOKYO_NIGHT_TOML, None).expect("builtin Tokyo Night theme should be valid")
}

/// Load the Gruvbox Dark theme.
#[must_use]
pub fn gruvbox_dark() -> Theme {
    load_from_str(GRUVBOX_DARK_TOML, None).expect("builtin Gruvbox Dark theme should be valid")
}

/// Load the One Dark theme.
#[must_use]
pub fn one_dark() -> Theme {
    load_from_str(ONE_DARK_TOML, None).expect("builtin One Dark theme should be valid")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Popular Light Themes
// ═══════════════════════════════════════════════════════════════════════════════

/// Load the Catppuccin Latte theme.
#[must_use]
pub fn catppuccin_latte() -> Theme {
    load_from_str(CATPPUCCIN_LATTE_TOML, None)
        .expect("builtin Catppuccin Latte theme should be valid")
}

/// Load the Solarized Light theme.
#[must_use]
pub fn solarized_light() -> Theme {
    load_from_str(SOLARIZED_LIGHT_TOML, None)
        .expect("builtin Solarized Light theme should be valid")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Theme Registry
// ═══════════════════════════════════════════════════════════════════════════════

/// Get all builtin theme names (for theme listings).
#[must_use]
pub fn builtin_names() -> &'static [(&'static str, &'static str)] {
    &[
        // SilkCircuit family (default first)
        ("silkcircuit-neon", "SilkCircuit Neon"),
        ("silkcircuit-soft", "SilkCircuit Soft"),
        ("silkcircuit-glow", "SilkCircuit Glow"),
        ("silkcircuit-vibrant", "SilkCircuit Vibrant"),
        ("silkcircuit-dawn", "SilkCircuit Dawn"),
        // Popular dark themes
        ("catppuccin-mocha", "Catppuccin Mocha"),
        ("dracula", "Dracula"),
        ("nord", "Nord"),
        ("tokyo-night", "Tokyo Night"),
        ("gruvbox-dark", "Gruvbox Dark"),
        ("one-dark", "One Dark"),
        // Popular light themes
        ("catppuccin-latte", "Catppuccin Latte"),
        ("solarized-light", "Solarized Light"),
    ]
}

/// Load a builtin theme by name.
///
/// Returns `None` if the name is not a builtin theme.
#[must_use]
pub fn load_by_name(name: &str) -> Option<Theme> {
    match name {
        // SilkCircuit family
        "silkcircuit-neon" | "default" => Some(silkcircuit_neon()),
        "silkcircuit-soft" => Some(silkcircuit_soft()),
        "silkcircuit-glow" => Some(silkcircuit_glow()),
        "silkcircuit-vibrant" => Some(silkcircuit_vibrant()),
        "silkcircuit-dawn" => Some(silkcircuit_dawn()),
        // Popular dark themes
        "catppuccin-mocha" => Some(catppuccin_mocha()),
        "dracula" => Some(dracula()),
        "nord" => Some(nord()),
        "tokyo-night" => Some(tokyo_night()),
        "gruvbox-dark" => Some(gruvbox_dark()),
        "one-dark" => Some(one_dark()),
        // Popular light themes
        "catppuccin-latte" => Some(catppuccin_latte()),
        "solarized-light" => Some(solarized_light()),
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
        assert_eq!(theme.color("purple_500"), ThemeColor::new(225, 53, 255));
        assert_eq!(theme.color("cyan_400"), ThemeColor::new(128, 255, 234));
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
        let start = theme.gradient("primary", 0.0);
        let end = theme.gradient("primary", 1.0);
        assert_eq!(start, ThemeColor::new(225, 53, 255));
        assert_eq!(end, ThemeColor::new(128, 255, 234));
    }

    #[test]
    fn test_all_silkcircuit_themes_load() {
        assert_eq!(silkcircuit_neon().meta.name, "SilkCircuit Neon");
        assert_eq!(silkcircuit_soft().meta.name, "SilkCircuit Soft");
        assert_eq!(silkcircuit_glow().meta.name, "SilkCircuit Glow");
        assert_eq!(silkcircuit_vibrant().meta.name, "SilkCircuit Vibrant");
        assert_eq!(silkcircuit_dawn().meta.name, "SilkCircuit Dawn");
    }

    #[test]
    fn test_popular_dark_themes_load() {
        assert_eq!(catppuccin_mocha().meta.name, "Catppuccin Mocha");
        assert_eq!(dracula().meta.name, "Dracula");
        assert_eq!(nord().meta.name, "Nord");
        assert_eq!(tokyo_night().meta.name, "Tokyo Night");
        assert_eq!(gruvbox_dark().meta.name, "Gruvbox Dark");
        assert_eq!(one_dark().meta.name, "One Dark");
    }

    #[test]
    fn test_popular_light_themes_load() {
        assert_eq!(catppuccin_latte().meta.name, "Catppuccin Latte");
        assert_eq!(solarized_light().meta.name, "Solarized Light");
    }

    #[test]
    fn test_light_themes_are_light_variant() {
        use crate::theme::schema::ThemeVariant;
        assert_eq!(silkcircuit_dawn().meta.variant, ThemeVariant::Light);
        assert_eq!(catppuccin_latte().meta.variant, ThemeVariant::Light);
        assert_eq!(solarized_light().meta.variant, ThemeVariant::Light);
    }

    #[test]
    fn test_load_by_name() {
        // SilkCircuit
        assert!(load_by_name("silkcircuit-neon").is_some());
        assert!(load_by_name("default").is_some());
        assert!(load_by_name("silkcircuit-soft").is_some());
        assert!(load_by_name("silkcircuit-glow").is_some());
        assert!(load_by_name("silkcircuit-vibrant").is_some());
        assert!(load_by_name("silkcircuit-dawn").is_some());
        // Popular dark
        assert!(load_by_name("catppuccin-mocha").is_some());
        assert!(load_by_name("dracula").is_some());
        assert!(load_by_name("nord").is_some());
        assert!(load_by_name("tokyo-night").is_some());
        assert!(load_by_name("gruvbox-dark").is_some());
        assert!(load_by_name("one-dark").is_some());
        // Popular light
        assert!(load_by_name("catppuccin-latte").is_some());
        assert!(load_by_name("solarized-light").is_some());
        // Invalid
        assert!(load_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_builtin_names() {
        let names = builtin_names();
        assert_eq!(names.len(), 13);
        assert!(names.iter().any(|(n, _)| *n == "silkcircuit-neon"));
        assert!(names.iter().any(|(n, _)| *n == "catppuccin-mocha"));
        assert!(names.iter().any(|(n, _)| *n == "dracula"));
        assert!(names.iter().any(|(n, _)| *n == "solarized-light"));
    }
}
