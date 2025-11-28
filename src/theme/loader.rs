//! Theme loader - loads themes from TOML files.

use std::path::Path;

use super::error::ThemeError;
use super::resolver::{resolve_gradients, resolve_styles, Resolver};
use super::schema::ThemeFile;
use super::Theme;

/// Load a theme from a file path.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn load_from_file(path: &Path) -> Result<Theme, ThemeError> {
    let content = std::fs::read_to_string(path).map_err(|e| ThemeError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;

    load_from_str(&content, Some(path))
}

/// Load a theme from a TOML string.
///
/// # Errors
/// Returns an error if the TOML is invalid or contains errors.
pub fn load_from_str(content: &str, path: Option<&Path>) -> Result<Theme, ThemeError> {
    let theme_file: ThemeFile = toml::from_str(content).map_err(|e| ThemeError::ParseError {
        path: path.map(Path::to_path_buf),
        source: e,
    })?;

    build_theme(theme_file)
}

/// Build a resolved Theme from a parsed `ThemeFile`.
fn build_theme(theme_file: ThemeFile) -> Result<Theme, ThemeError> {
    // Resolve palette and tokens
    let resolver = Resolver::new(&theme_file);
    let (palette, tokens) = resolver.resolve()?;

    // Resolve styles
    let styles = resolve_styles(&theme_file.styles, &palette, &tokens);

    // Resolve gradients
    let gradients = resolve_gradients(&theme_file.gradients, &palette, &tokens);

    Ok(Theme {
        meta: theme_file.meta,
        palette,
        tokens,
        styles,
        gradients,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeColor;

    #[test]
    fn test_load_minimal_theme() {
        let toml = r##"
            [meta]
            name = "Test"

            [palette]
            red = "#ff0000"

            [tokens]
            error = "red"
        "##;

        let theme = load_from_str(toml, None).unwrap();
        assert_eq!(theme.meta.name, "Test");
        assert_eq!(theme.color("error"), ThemeColor::new(255, 0, 0));
    }

    #[test]
    fn test_load_complete_theme() {
        let toml = r##"
            [meta]
            name = "Complete Test"
            author = "Test Author"
            variant = "dark"
            version = "1.0"

            [palette]
            purple = "#e135ff"
            cyan = "#80ffea"
            gray = "#808080"

            [tokens]
            "accent.primary" = "purple"
            "accent.secondary" = "cyan"
            "text.dim" = "gray"

            [styles]
            keyword = { fg = "accent.primary", bold = true }
            selected = { fg = "accent.secondary", bg = "#1e1e28" }

            [gradients]
            primary = ["purple", "cyan"]
        "##;

        let theme = load_from_str(toml, None).unwrap();

        assert_eq!(theme.meta.name, "Complete Test");
        assert_eq!(theme.meta.author, Some("Test Author".to_string()));

        // Test token resolution
        assert_eq!(theme.color("accent.primary"), ThemeColor::new(225, 53, 255));
        assert_eq!(theme.color("accent.secondary"), ThemeColor::new(128, 255, 234));

        // Test style resolution
        let keyword_style = theme.style("keyword");
        assert_eq!(keyword_style.fg, Some(ThemeColor::new(225, 53, 255)));
        assert!(keyword_style.bold);

        // Test gradient resolution
        let gradient_start = theme.gradient("primary", 0.0);
        let gradient_end = theme.gradient("primary", 1.0);
        assert_eq!(gradient_start, ThemeColor::new(225, 53, 255));
        assert_eq!(gradient_end, ThemeColor::new(128, 255, 234));
    }

    #[test]
    fn test_fallback_for_missing_token() {
        let toml = r##"
            [meta]
            name = "Test"
        "##;

        let theme = load_from_str(toml, None).unwrap();
        // Missing token should return fallback
        assert_eq!(theme.color("nonexistent"), ThemeColor::FALLBACK);
    }
}
