//! TOML schema types for theme files.
//!
//! These types define the structure of theme TOML files and handle deserialization.

use std::collections::HashMap;

use serde::Deserialize;

/// Root structure of a theme TOML file.
#[derive(Debug, Deserialize)]
pub struct ThemeFile {
    /// Theme metadata.
    pub meta: ThemeMeta,

    /// Color palette - named color primitives.
    #[serde(default)]
    pub palette: HashMap<String, String>,

    /// Semantic tokens - map to palette colors or other tokens.
    #[serde(default)]
    pub tokens: HashMap<String, String>,

    /// Composed styles with modifiers.
    #[serde(default)]
    pub styles: HashMap<String, StyleDef>,

    /// Gradient definitions.
    #[serde(default)]
    pub gradients: HashMap<String, Vec<String>>,
}

/// Theme metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeMeta {
    /// Display name of the theme.
    pub name: String,

    /// Theme author.
    #[serde(default)]
    pub author: Option<String>,

    /// Theme variant (dark/light).
    #[serde(default = "default_variant")]
    pub variant: ThemeVariant,

    /// Theme version.
    #[serde(default)]
    pub version: Option<String>,

    /// Theme description.
    #[serde(default)]
    pub description: Option<String>,
}

fn default_variant() -> ThemeVariant {
    ThemeVariant::Dark
}

/// Theme variant - light or dark.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    #[default]
    Dark,
    Light,
}

/// Style definition from TOML.
#[derive(Debug, Clone, Deserialize)]
#[derive(Default)]
pub struct StyleDef {
    /// Foreground color token reference.
    #[serde(default)]
    pub fg: Option<String>,

    /// Background color token reference.
    #[serde(default)]
    pub bg: Option<String>,

    /// Bold modifier.
    #[serde(default)]
    pub bold: bool,

    /// Italic modifier.
    #[serde(default)]
    pub italic: bool,

    /// Underline modifier.
    #[serde(default)]
    pub underline: bool,

    /// Dim modifier.
    #[serde(default)]
    pub dim: bool,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_theme() {
        let toml = r##"
            [meta]
            name = "Test Theme"

            [palette]
            red = "#ff0000"

            [tokens]
            error = "red"
        "##;

        let theme: ThemeFile = toml::from_str(toml).unwrap();
        assert_eq!(theme.meta.name, "Test Theme");
        assert_eq!(theme.palette.get("red"), Some(&"#ff0000".to_string()));
        assert_eq!(theme.tokens.get("error"), Some(&"red".to_string()));
    }

    #[test]
    fn test_parse_style_def() {
        let toml = r##"
            [meta]
            name = "Test"

            [styles]
            keyword = { fg = "purple", bold = true }
            selected = { fg = "cyan", bg = "highlight" }
        "##;

        let theme: ThemeFile = toml::from_str(toml).unwrap();
        let keyword = theme.styles.get("keyword").unwrap();
        assert_eq!(keyword.fg, Some("purple".to_string()));
        assert!(keyword.bold);
        assert!(!keyword.italic);
    }

    #[test]
    fn test_parse_gradients() {
        let toml = r##"
            [meta]
            name = "Test"

            [gradients]
            primary = ["purple", "cyan"]
            warm = ["coral", "yellow", "orange"]
        "##;

        let theme: ThemeFile = toml::from_str(toml).unwrap();
        assert_eq!(
            theme.gradients.get("primary"),
            Some(&vec!["purple".to_string(), "cyan".to_string()])
        );
    }
}
