//! Tests for theme module

use crate::theme::{
    self, Theme, ThemeColor, ThemeError, ThemeVariant, current, list_available_themes,
    load_theme_by_name, set_theme,
};

#[test]
fn test_current_returns_theme() {
    // Reset to default first to handle parallel test execution
    set_theme(Theme::builtin_neon());

    let theme = current();
    // Should be SilkCircuit Neon now
    assert_eq!(theme.meta.name, "SilkCircuit Neon");
    assert_eq!(theme.meta.variant, ThemeVariant::Dark);
}

#[test]
fn test_set_and_get_theme() {
    // Create a custom theme
    let mut custom = Theme::builtin_neon();
    custom.meta.name = "Test Theme For Set".to_string();

    // Set it
    set_theme(custom);

    // Verify it's active
    let theme = current();
    assert_eq!(theme.meta.name, "Test Theme For Set");

    // Reset to default
    set_theme(Theme::builtin_neon());
}

#[test]
fn test_load_theme_by_name_builtin() {
    // Should load all builtin themes
    assert!(load_theme_by_name("silkcircuit-neon").is_ok());
    assert_eq!(current().meta.name, "SilkCircuit Neon");

    assert!(load_theme_by_name("silkcircuit-soft").is_ok());
    assert_eq!(current().meta.name, "SilkCircuit Soft");

    assert!(load_theme_by_name("silkcircuit-glow").is_ok());
    assert_eq!(current().meta.name, "SilkCircuit Glow");

    assert!(load_theme_by_name("silkcircuit-vibrant").is_ok());
    assert_eq!(current().meta.name, "SilkCircuit Vibrant");

    assert!(load_theme_by_name("silkcircuit-dawn").is_ok());
    assert_eq!(current().meta.name, "SilkCircuit Dawn");

    // Also test "default" alias
    assert!(load_theme_by_name("default").is_ok());
    assert_eq!(current().meta.name, "SilkCircuit Neon");
}

#[test]
fn test_load_theme_by_name_not_found() {
    let result = load_theme_by_name("nonexistent-theme");
    assert!(result.is_err());
    assert!(matches!(result, Err(ThemeError::ThemeNotFound { .. })));
}

#[test]
fn test_list_available_themes_includes_builtins() {
    let themes = list_available_themes();

    // Should have all 13 builtin themes (5 SilkCircuit + 6 popular dark + 2 light)
    let builtins: Vec<_> = themes.iter().filter(|t| t.builtin).collect();
    assert_eq!(builtins.len(), 13);

    // Check that we have SilkCircuit variants
    assert!(builtins.iter().any(|t| t.name == "silkcircuit-neon"));
    assert!(builtins.iter().any(|t| t.name == "silkcircuit-soft"));
    assert!(builtins.iter().any(|t| t.name == "silkcircuit-glow"));
    assert!(builtins.iter().any(|t| t.name == "silkcircuit-vibrant"));
    assert!(builtins.iter().any(|t| t.name == "silkcircuit-dawn"));

    // Check popular dark themes
    assert!(builtins.iter().any(|t| t.name == "catppuccin-mocha"));
    assert!(builtins.iter().any(|t| t.name == "dracula"));
    assert!(builtins.iter().any(|t| t.name == "nord"));
    assert!(builtins.iter().any(|t| t.name == "tokyo-night"));
    assert!(builtins.iter().any(|t| t.name == "gruvbox-dark"));
    assert!(builtins.iter().any(|t| t.name == "one-dark"));

    // Check light themes
    assert!(builtins.iter().any(|t| t.name == "catppuccin-latte"));
    assert!(builtins.iter().any(|t| t.name == "solarized-light"));

    // Check that light themes have correct variant
    let light_themes: Vec<_> = builtins
        .iter()
        .filter(|t| t.variant == ThemeVariant::Light)
        .collect();
    assert_eq!(light_themes.len(), 3); // dawn, latte, solarized
}

#[test]
fn test_theme_color_access() {
    let theme = current();

    // Test existing tokens
    let primary = theme.color("accent.primary");
    assert_ne!(primary, ThemeColor::FALLBACK);

    // Test fallback for missing tokens
    let missing = theme.color("nonexistent.token");
    assert_eq!(missing, ThemeColor::FALLBACK);
}

#[test]
fn test_theme_style_access() {
    let theme = current();

    // Test existing style
    let keyword = theme.style("keyword");
    assert!(keyword.fg.is_some());
    assert!(keyword.bold);

    // Test fallback for missing style
    let missing = theme.style("nonexistent_style");
    assert!(missing.fg.is_none());
}

#[test]
fn test_theme_gradient_access() {
    let theme = current();

    // Test existing gradient
    let color = theme.gradient("primary", 0.5);
    assert_ne!(color, ThemeColor::FALLBACK);

    // Test fallback for missing gradient
    let missing = theme.gradient("nonexistent", 0.5);
    assert_eq!(missing, ThemeColor::FALLBACK);
}

#[test]
fn test_theme_has_methods() {
    let theme = current();

    // Test has_token
    assert!(theme.has_token("accent.primary"));
    assert!(!theme.has_token("nonexistent.token"));

    // Test has_style
    assert!(theme.has_style("keyword"));
    assert!(!theme.has_style("nonexistent_style"));

    // Test has_gradient
    assert!(theme.has_gradient("primary"));
    assert!(!theme.has_gradient("nonexistent"));
}

#[test]
fn test_theme_names_methods() {
    let theme = current();

    // Should have tokens
    let tokens = theme.token_names();
    assert!(!tokens.is_empty());
    assert!(tokens.contains(&"accent.primary"));

    // Should have styles
    let styles = theme.style_names();
    assert!(!styles.is_empty());
    assert!(styles.contains(&"keyword"));

    // Should have gradients
    let gradients = theme.gradient_names();
    assert!(!gradients.is_empty());
    assert!(gradients.contains(&"primary"));
}
