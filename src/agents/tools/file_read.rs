//! File reading tool
//!
//! Simple file reading capability with support for partial reads (head/tail).
//! This is more efficient than using `code_search` when you need the actual content.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::define_tool_error;

use super::common::{get_current_repo, parameters_schema};

define_tool_error!(FileReadError);

/// File reading tool for accessing file contents directly
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileRead;

impl FileRead {
    /// Maximum lines to return by default (to avoid overwhelming context)
    const DEFAULT_MAX_LINES: usize = 500;

    /// Line number column width (supports files up to 999,999 lines)
    const LINE_NUM_WIDTH: usize = 6;

    /// Check if file appears to be binary
    fn is_binary(content: &[u8]) -> bool {
        let check_size = content.len().min(8192);
        content[..check_size].contains(&0)
    }

    /// Check if file extension indicates binary
    fn is_binary_extension(path: &str) -> bool {
        const BINARY_EXTENSIONS: &[&str] = &[
            ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".pdf", ".zip", ".tar",
            ".gz", ".rar", ".7z", ".exe", ".dll", ".so", ".dylib", ".bin", ".wasm", ".ttf", ".otf",
            ".woff", ".woff2", ".mp3", ".mp4", ".wav", ".sqlite", ".db", ".pyc", ".class", ".o",
            ".a",
        ];
        let path_lower = path.to_lowercase();
        BINARY_EXTENSIONS
            .iter()
            .any(|ext| path_lower.ends_with(ext))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FileReadArgs {
    /// Path to the file to read (relative to repo root)
    pub path: String,
    /// Starting line number (1-indexed, default: 1)
    #[serde(default)]
    pub start_line: Option<usize>,
    /// Number of lines to read (default: 500, max: 1000)
    #[serde(default)]
    pub num_lines: Option<usize>,
}

impl Tool for FileRead {
    const NAME: &'static str = "file_read";
    type Error = FileReadError;
    type Args = FileReadArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "file_read".to_string(),
            description: "Read file contents directly. Use start_line and num_lines for partial reads on large files. Returns line-numbered content.".to_string(),
            parameters: parameters_schema::<FileReadArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let repo = get_current_repo().map_err(FileReadError::from)?;
        let repo_path = repo.repo_path();

        // Reject absolute paths - all paths must be relative to repo root
        if Path::new(&args.path).is_absolute() {
            return Err(FileReadError(
                "Absolute paths not allowed. Use paths relative to repository root.".into(),
            ));
        }

        // Join path to repo root
        let file_path = repo_path.join(&args.path);

        // Check file exists before canonicalization
        if !file_path.exists() {
            return Err(FileReadError(format!("File not found: {}", args.path)));
        }

        // Canonicalize both paths to resolve symlinks and .. components
        let canonical_file = file_path
            .canonicalize()
            .map_err(|e| FileReadError(format!("Cannot resolve path: {e}")))?;
        let canonical_repo = repo_path
            .canonicalize()
            .map_err(|e| FileReadError(format!("Cannot resolve repo path: {e}")))?;

        // Security: verify resolved path is within repository bounds
        if !canonical_file.starts_with(&canonical_repo) {
            return Err(FileReadError("Path escapes repository boundaries".into()));
        }

        if !canonical_file.is_file() {
            return Err(FileReadError(format!("Not a file: {}", args.path)));
        }

        // Check for binary extension
        if Self::is_binary_extension(&args.path) {
            return Ok(format!(
                "[Binary file: {} - content not displayed]",
                args.path
            ));
        }

        // Read the file (use canonical path for actual read)
        let content = fs::read(&canonical_file).map_err(|e| FileReadError(e.to_string()))?;

        // Check for binary content
        if Self::is_binary(&content) {
            return Ok(format!(
                "[Binary file detected: {} - content not displayed]",
                args.path
            ));
        }

        // Convert to string
        let content_str = String::from_utf8(content).map_err(|e| FileReadError(e.to_string()))?;

        let lines: Vec<&str> = content_str.lines().collect();
        let total_lines = lines.len();

        // Calculate range
        let start = args.start_line.unwrap_or(1).saturating_sub(1); // Convert to 0-indexed
        let max_lines = args.num_lines.unwrap_or(Self::DEFAULT_MAX_LINES).min(1000);
        let end = (start + max_lines).min(total_lines);

        // Build output with line numbers
        let mut output = String::new();
        output.push_str(&format!(
            "=== {} ({} total lines) ===\n",
            args.path, total_lines
        ));

        if start > 0 || end < total_lines {
            output.push_str(&format!(
                "Showing lines {}-{} of {}\n",
                start + 1,
                end,
                total_lines
            ));
        }
        output.push('\n');

        for (i, line) in lines.iter().enumerate().skip(start).take(end - start) {
            output.push_str(&format!(
                "{:>width$}│ {}\n",
                i + 1,
                line,
                width = Self::LINE_NUM_WIDTH
            ));
        }

        if end < total_lines {
            output.push_str(&format!(
                "\n... {} more lines (use start_line={} to continue)\n",
                total_lines - end,
                end + 1
            ));
        }

        Ok(output)
    }
}
