//! Tests for builtin themes

use crate::theme::ThemeColor;
use crate::theme::builtins::{
    builtin_names, catppuccin_latte, catppuccin_mocha, dracula, gruvbox_dark, load_by_name, nord,
    one_dark, silkcircuit_dawn, silkcircuit_glow, silkcircuit_neon, silkcircuit_soft,
    silkcircuit_vibrant, solarized_light, tokyo_night,
};
use crate::theme::schema::ThemeVariant;

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
