//! Token resolver - resolves token references to concrete colors.

use std::collections::{HashMap, HashSet};

use super::color::ThemeColor;
use super::error::ThemeError;
use super::gradient::Gradient;
use super::schema::{StyleDef, ThemeFile};
use super::style::ThemeStyle;

/// Resolved color maps from theme resolution.
pub type ResolvedColors = (HashMap<String, ThemeColor>, HashMap<String, ThemeColor>);

/// Resolver state for building a theme.
pub struct Resolver {
    /// Original palette from TOML.
    palette_raw: HashMap<String, String>,
    /// Original tokens from TOML.
    tokens_raw: HashMap<String, String>,
    /// Resolved palette colors.
    palette: HashMap<String, ThemeColor>,
    /// Resolved token colors.
    tokens: HashMap<String, ThemeColor>,
}

impl Resolver {
    /// Create a new resolver from a theme file.
    pub fn new(theme: &ThemeFile) -> Self {
        Self {
            palette_raw: theme.palette.clone(),
            tokens_raw: theme.tokens.clone(),
            palette: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    /// Resolve all colors and return the resolved maps.
    ///
    /// # Errors
    /// Returns an error if there are circular references or invalid colors.
    pub fn resolve(mut self) -> Result<ResolvedColors, ThemeError> {
        // First pass: resolve palette (can only contain hex values)
        for (name, value) in &self.palette_raw {
            let color = ThemeColor::from_hex(value).map_err(|e| ThemeError::InvalidColor {
                token: name.clone(),
                source: e,
            })?;
            self.palette.insert(name.clone(), color);
        }

        // Second pass: resolve tokens (can reference palette or other tokens)
        let token_names: Vec<String> = self.tokens_raw.keys().cloned().collect();
        for name in token_names {
            self.resolve_token(&name, &mut HashSet::new())?;
        }

        Ok((self.palette, self.tokens))
    }

    /// Resolve a single token, tracking the resolution chain to detect cycles.
    fn resolve_token(
        &mut self,
        name: &str,
        chain: &mut HashSet<String>,
    ) -> Result<ThemeColor, ThemeError> {
        // Already resolved?
        if let Some(color) = self.tokens.get(name) {
            return Ok(*color);
        }

        // Check for circular reference
        if chain.contains(name) {
            return Err(ThemeError::CircularReference {
                token: name.to_string(),
                chain: chain.iter().cloned().collect(),
            });
        }

        // Get the raw value
        let value = self
            .tokens_raw
            .get(name)
            .ok_or_else(|| ThemeError::UnresolvedToken {
                token: name.to_string(),
                reference: name.to_string(),
            })?
            .clone();

        chain.insert(name.to_string());

        // Try to resolve the value
        let color = self.resolve_value(&value, chain)?;

        chain.remove(name);
        self.tokens.insert(name.to_string(), color);
        Ok(color)
    }

    /// Resolve a value which could be a hex color, palette reference, or token reference.
    fn resolve_value(
        &mut self,
        value: &str,
        chain: &mut HashSet<String>,
    ) -> Result<ThemeColor, ThemeError> {
        // Is it a hex color?
        if value.starts_with('#') {
            return ThemeColor::from_hex(value).map_err(|e| ThemeError::InvalidColor {
                token: value.to_string(),
                source: e,
            });
        }

        // Is it a palette reference?
        if let Some(color) = self.palette.get(value) {
            return Ok(*color);
        }

        // Is it a token reference?
        if self.tokens_raw.contains_key(value) {
            return self.resolve_token(value, chain);
        }

        // Not found - use fallback
        Ok(ThemeColor::FALLBACK)
    }
}

/// Resolve styles from their definitions.
pub fn resolve_styles(
    style_defs: &HashMap<String, StyleDef>,
    palette: &HashMap<String, ThemeColor>,
    tokens: &HashMap<String, ThemeColor>,
) -> HashMap<String, ThemeStyle> {
    style_defs
        .iter()
        .map(|(name, def)| {
            let style = ThemeStyle {
                fg: def
                    .fg
                    .as_ref()
                    .and_then(|s| resolve_color_ref(s, palette, tokens)),
                bg: def
                    .bg
                    .as_ref()
                    .and_then(|s| resolve_color_ref(s, palette, tokens)),
                bold: def.bold,
                italic: def.italic,
                underline: def.underline,
                dim: def.dim,
            };
            (name.clone(), style)
        })
        .collect()
}

/// Resolve a color reference string to a color.
fn resolve_color_ref(
    reference: &str,
    palette: &HashMap<String, ThemeColor>,
    tokens: &HashMap<String, ThemeColor>,
) -> Option<ThemeColor> {
    // Try hex first
    if reference.starts_with('#') {
        return ThemeColor::from_hex(reference).ok();
    }

    // Try tokens
    if let Some(color) = tokens.get(reference) {
        return Some(*color);
    }

    // Try palette
    if let Some(color) = palette.get(reference) {
        return Some(*color);
    }

    // Fallback
    None
}

/// Resolve gradients from their definitions.
pub fn resolve_gradients(
    gradient_defs: &HashMap<String, Vec<String>>,
    palette: &HashMap<String, ThemeColor>,
    tokens: &HashMap<String, ThemeColor>,
) -> HashMap<String, Gradient> {
    gradient_defs
        .iter()
        .filter_map(|(name, stops)| {
            let colors: Vec<ThemeColor> = stops
                .iter()
                .filter_map(|s| resolve_color_ref(s, palette, tokens))
                .collect();

            if colors.is_empty() {
                None
            } else {
                Some((name.clone(), Gradient::new(colors)))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_theme_file(
        palette: HashMap<String, String>,
        tokens: HashMap<String, String>,
    ) -> ThemeFile {
        ThemeFile {
            meta: super::super::schema::ThemeMeta {
                name: "Test".to_string(),
                author: None,
                variant: super::super::schema::ThemeVariant::Dark,
                version: None,
                description: None,
            },
            palette,
            tokens,
            styles: HashMap::new(),
            gradients: HashMap::new(),
        }
    }

    #[test]
    fn test_resolve_palette() {
        let theme = make_theme_file(
            [("red".into(), "#ff0000".into())].into_iter().collect(),
            HashMap::new(),
        );

        let resolver = Resolver::new(&theme);
        let (palette, _) = resolver.resolve().unwrap();

        assert_eq!(palette.get("red"), Some(&ThemeColor::new(255, 0, 0)));
    }

    #[test]
    fn test_resolve_token_reference() {
        let theme = make_theme_file(
            [("red".into(), "#ff0000".into())].into_iter().collect(),
            [("error".into(), "red".into())].into_iter().collect(),
        );

        let resolver = Resolver::new(&theme);
        let (_, tokens) = resolver.resolve().unwrap();

        assert_eq!(tokens.get("error"), Some(&ThemeColor::new(255, 0, 0)));
    }

    #[test]
    fn test_resolve_chained_reference() {
        let theme = make_theme_file(
            [("red".into(), "#ff0000".into())].into_iter().collect(),
            [
                ("danger".into(), "red".into()),
                ("error".into(), "danger".into()),
            ]
            .into_iter()
            .collect(),
        );

        let resolver = Resolver::new(&theme);
        let (_, tokens) = resolver.resolve().unwrap();

        assert_eq!(tokens.get("error"), Some(&ThemeColor::new(255, 0, 0)));
    }

    #[test]
    fn test_detect_circular_reference() {
        let theme = make_theme_file(
            HashMap::new(),
            [
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("c".into(), "a".into()),
            ]
            .into_iter()
            .collect(),
        );

        let resolver = Resolver::new(&theme);
        let result = resolver.resolve();

        assert!(matches!(result, Err(ThemeError::CircularReference { .. })));
    }
}
