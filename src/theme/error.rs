//! Theme error types.

use std::fmt;
use std::path::PathBuf;

use super::color::ColorParseError;

/// Errors that can occur when loading or resolving themes.
#[derive(Debug)]
pub enum ThemeError {
    /// Failed to read theme file.
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Failed to parse TOML.
    ParseError {
        path: Option<PathBuf>,
        source: toml::de::Error,
    },
    /// Invalid color value.
    InvalidColor {
        token: String,
        source: ColorParseError,
    },
    /// Circular reference detected in token resolution.
    CircularReference { token: String, chain: Vec<String> },
    /// Referenced token not found.
    UnresolvedToken { token: String, reference: String },
    /// Missing required section in theme file.
    MissingSection { section: String },
    /// Theme not found by name.
    ThemeNotFound { name: String },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError { path, source } => {
                write!(
                    f,
                    "failed to read theme file '{}': {}",
                    path.display(),
                    source
                )
            }
            Self::ParseError { path, source } => {
                if let Some(path) = path {
                    write!(f, "failed to parse theme '{}': {}", path.display(), source)
                } else {
                    write!(f, "failed to parse theme: {source}")
                }
            }
            Self::InvalidColor { token, source } => {
                write!(f, "invalid color for token '{token}': {source}")
            }
            Self::CircularReference { token, chain } => {
                write!(
                    f,
                    "circular reference detected for token '{token}': {}",
                    chain.join(" -> ")
                )
            }
            Self::UnresolvedToken { token, reference } => {
                write!(f, "token '{token}' references unknown token '{reference}'")
            }
            Self::MissingSection { section } => {
                write!(f, "missing required section '[{section}]' in theme file")
            }
            Self::ThemeNotFound { name } => {
                write!(f, "theme '{name}' not found")
            }
        }
    }
}

impl std::error::Error for ThemeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError { source, .. } => Some(source),
            Self::ParseError { source, .. } => Some(source),
            Self::InvalidColor { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<ColorParseError> for ThemeError {
    fn from(err: ColorParseError) -> Self {
        Self::InvalidColor {
            token: String::new(),
            source: err,
        }
    }
}
