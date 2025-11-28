//! Token-based theme system for Git-Iris.
//!
//! This module provides a flexible, TOML-configurable theme system that supports:
//! - Color primitives defined in a palette
//! - Semantic tokens that reference palette colors or other tokens
//! - Composed styles with foreground, background, and modifiers
//! - Gradient definitions for smooth color transitions
//! - Runtime theme switching with thread-safe global state
//!
//! # Theme File Format
//!
//! Themes are defined in TOML files with the following structure:
//!
//! ```toml
//! [meta]
//! name = "My Theme"
//! author = "Your Name"
//! variant = "dark"  # or "light"
//!
//! [palette]
//! purple_500 = "#e135ff"
//! cyan_400 = "#80ffea"
//!
//! [tokens]
//! text.primary = "#f8f8f2"
//! accent.primary = "purple_500"  # references palette
//! success = "cyan_400"
//!
//! [styles]
//! keyword = { fg = "accent.primary", bold = true }
//!
//! [gradients]
//! primary = ["purple_500", "cyan_400"]
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use git_iris::theme;
//!
//! // Get current theme
//! let theme = theme::current();
//!
//! // Access colors
//! let color = theme.color("accent.primary");
//!
//! // Access styles
//! let style = theme.style("keyword");
//!
//! // Access gradients
//! let gradient_color = theme.gradient("primary", 0.5);
//! ```

pub mod adapters;
mod color;
mod error;
mod gradient;
mod loader;
mod resolver;
mod schema;

pub mod builtins;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use parking_lot::RwLock;

// Re-exports
pub use color::ThemeColor;
pub use error::ThemeError;
pub use gradient::Gradient;
pub use schema::{ThemeMeta, ThemeVariant};
pub use style::ThemeStyle;

mod style;

/// A resolved theme with all tokens and styles ready for use.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme metadata.
    pub meta: ThemeMeta,

    /// Resolved color palette (palette name -> color).
    palette: HashMap<String, ThemeColor>,

    /// Resolved semantic tokens (token name -> color).
    tokens: HashMap<String, ThemeColor>,

    /// Resolved composed styles (style name -> style).
    styles: HashMap<String, ThemeStyle>,

    /// Resolved gradients (gradient name -> gradient).
    gradients: HashMap<String, Gradient>,
}

impl Theme {
    /// Get a color by token name.
    ///
    /// Falls back to `ThemeColor::FALLBACK` if the token is not found.
    #[must_use]
    pub fn color(&self, token: &str) -> ThemeColor {
        self.tokens
            .get(token)
            .or_else(|| self.palette.get(token))
            .copied()
            .unwrap_or(ThemeColor::FALLBACK)
    }

    /// Get a style by name.
    ///
    /// Returns a default (empty) style if not found.
    #[must_use]
    pub fn style(&self, name: &str) -> ThemeStyle {
        self.styles.get(name).cloned().unwrap_or_default()
    }

    /// Get a gradient color at position `t` (0.0 to 1.0).
    ///
    /// Falls back to `ThemeColor::FALLBACK` if the gradient is not found.
    #[must_use]
    pub fn gradient(&self, name: &str, t: f32) -> ThemeColor {
        self.gradients
            .get(name)
            .map_or(ThemeColor::FALLBACK, |g| g.at(t))
    }

    /// Get a gradient by name for manual interpolation.
    #[must_use]
    pub fn get_gradient(&self, name: &str) -> Option<&Gradient> {
        self.gradients.get(name)
    }

    /// Check if a token exists.
    #[must_use]
    pub fn has_token(&self, token: &str) -> bool {
        self.tokens.contains_key(token) || self.palette.contains_key(token)
    }

    /// Check if a style exists.
    #[must_use]
    pub fn has_style(&self, name: &str) -> bool {
        self.styles.contains_key(name)
    }

    /// Check if a gradient exists.
    #[must_use]
    pub fn has_gradient(&self, name: &str) -> bool {
        self.gradients.contains_key(name)
    }

    /// Get all token names.
    #[must_use]
    pub fn token_names(&self) -> Vec<&str> {
        self.tokens.keys().map(String::as_str).collect()
    }

    /// Get all style names.
    #[must_use]
    pub fn style_names(&self) -> Vec<&str> {
        self.styles.keys().map(String::as_str).collect()
    }

    /// Get all gradient names.
    #[must_use]
    pub fn gradient_names(&self) -> Vec<&str> {
        self.gradients.keys().map(String::as_str).collect()
    }

    /// Load the builtin `SilkCircuit` Neon theme.
    #[must_use]
    pub fn builtin_neon() -> Self {
        builtins::silkcircuit_neon()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::builtin_neon()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global Theme State
// ═══════════════════════════════════════════════════════════════════════════════

/// Global active theme.
static ACTIVE_THEME: LazyLock<RwLock<Arc<Theme>>> =
    LazyLock::new(|| RwLock::new(Arc::new(Theme::builtin_neon())));

/// Get the current active theme.
#[must_use]
pub fn current() -> Arc<Theme> {
    ACTIVE_THEME.read().clone()
}

/// Set the active theme.
pub fn set_theme(theme: Theme) {
    *ACTIVE_THEME.write() = Arc::new(theme);
}

/// Load and set a theme from a file path.
///
/// # Errors
/// Returns an error if the theme file cannot be loaded or parsed.
pub fn load_theme(path: &Path) -> Result<(), ThemeError> {
    let theme = loader::load_from_file(path)?;
    set_theme(theme);
    Ok(())
}

/// Load and set a theme by name (searches discovery paths).
///
/// # Errors
/// Returns an error if the theme is not found or cannot be loaded.
pub fn load_theme_by_name(name: &str) -> Result<(), ThemeError> {
    // Check builtins first
    if let Some(theme) = builtins::load_by_name(name) {
        set_theme(theme);
        return Ok(());
    }

    // Search discovery paths
    for path in discovery_paths() {
        let theme_path = path.join(format!("{name}.toml"));
        if theme_path.exists() {
            return load_theme(&theme_path);
        }
    }

    Err(ThemeError::ThemeNotFound {
        name: name.to_string(),
    })
}

/// List all available themes.
#[must_use]
pub fn list_available_themes() -> Vec<ThemeInfo> {
    // Start with all builtin themes
    let mut themes: Vec<ThemeInfo> = builtins::builtin_names()
        .iter()
        .map(|(name, display_name)| {
            let theme = builtins::load_by_name(name).expect("builtin theme should load");
            ThemeInfo {
                name: (*name).to_string(),
                display_name: (*display_name).to_string(),
                variant: theme.meta.variant,
                author: theme.meta.author.clone().unwrap_or_default(),
                description: theme.meta.description.clone().unwrap_or_default(),
                builtin: true,
                path: None,
            }
        })
        .collect();

    // Scan discovery paths for additional themes
    for dir in discovery_paths() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml")
                    && let Ok(theme) = loader::load_from_file(&path)
                {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    themes.push(ThemeInfo {
                        name,
                        display_name: theme.meta.name,
                        variant: theme.meta.variant,
                        author: theme.meta.author.unwrap_or_default(),
                        description: theme.meta.description.unwrap_or_default(),
                        builtin: false,
                        path: Some(path),
                    });
                }
            }
        }
    }

    themes
}

/// Information about an available theme.
#[derive(Debug, Clone)]
pub struct ThemeInfo {
    /// Theme identifier (filename without extension).
    pub name: String,
    /// Display name from theme metadata.
    pub display_name: String,
    /// Theme variant (dark/light).
    pub variant: ThemeVariant,
    /// Theme author.
    pub author: String,
    /// Theme description.
    pub description: String,
    /// Whether this is a builtin theme.
    pub builtin: bool,
    /// Path to theme file (None for builtins).
    pub path: Option<std::path::PathBuf>,
}

/// Get the theme discovery paths.
///
/// Themes are searched in order:
/// 1. `~/.config/git-iris/themes/`
/// 2. `$XDG_CONFIG_HOME/git-iris/themes/` (if different from above)
fn discovery_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    // User config directory
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".config/git-iris/themes"));
    }

    // XDG config directory
    if let Some(xdg_config) = dirs::config_dir() {
        let xdg_path = xdg_config.join("git-iris/themes");
        if !paths.contains(&xdg_path) {
            paths.push(xdg_path);
        }
    }

    paths
}

#[cfg(test)]
mod tests;
