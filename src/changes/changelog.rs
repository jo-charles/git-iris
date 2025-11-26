//! Changelog file utilities

use crate::git::GitRepo;
use crate::log_debug;
use anyhow::{Context, Result};
use regex;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// Utilities for changelog file management
pub struct ChangelogGenerator;

impl ChangelogGenerator {
    /// Updates a changelog file with new content
    ///
    /// This function reads the existing changelog file (if it exists), preserves the header,
    /// and prepends the new changelog content while maintaining the file structure.
    ///
    /// # Arguments
    ///
    /// * `changelog_content` - The new changelog content to prepend
    /// * `changelog_path` - Path to the changelog file
    /// * `git_repo` - `GitRepo` instance to use for retrieving commit dates
    /// * `to_ref` - The "to" Git reference (commit/tag) to extract the date from
    /// * `version_name` - Optional custom version name to use instead of version from Git
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    #[allow(clippy::too_many_lines)]
    pub fn update_changelog_file(
        changelog_content: &str,
        changelog_path: &str,
        git_repo: &Arc<GitRepo>,
        to_ref: &str,
        version_name: Option<String>,
    ) -> Result<()> {
        let path = Path::new(changelog_path);
        let default_header = "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n";

        // Get the date from the "to" Git reference
        let commit_date = match git_repo.get_commit_date(to_ref) {
            Ok(date) => {
                log_debug!("Got commit date for {}: {}", to_ref, date);
                date
            }
            Err(e) => {
                log_debug!("Failed to get commit date for {}: {}", to_ref, e);
                chrono::Local::now().format("%Y-%m-%d").to_string()
            }
        };

        // Strip ANSI color codes
        let stripped_content = strip_ansi_codes(changelog_content);

        // Skip the separator line if it exists (the first line with "━━━" or similar)
        let clean_content =
            if stripped_content.starts_with("━") || stripped_content.starts_with('-') {
                // Find the first newline and skip everything before it
                if let Some(pos) = stripped_content.find('\n') {
                    stripped_content[pos + 1..].to_string()
                } else {
                    stripped_content
                }
            } else {
                stripped_content
            };

        // Extract just the version content (skip the header)
        let mut version_content = if clean_content.contains("## [") {
            let parts: Vec<&str> = clean_content.split("## [").collect();
            if parts.len() > 1 {
                format!("## [{}", parts[1])
            } else {
                clean_content
            }
        } else {
            clean_content
        };

        // If version_name is provided, override the existing version
        if let Some(version) = version_name {
            if version_content.contains("## [") {
                let re = regex::Regex::new(r"## \[([^\]]+)\]").expect("Failed to compile regex");
                version_content = re
                    .replace(&version_content, &format!("## [{version}]"))
                    .to_string();
                log_debug!("Replaced version with user-provided version: {}", version);
            } else {
                log_debug!("Could not find version header to replace in changelog content");
            }
        }

        // Ensure version content has a date
        if version_content.contains(" - \n") {
            // Replace empty date placeholder with the commit date
            version_content = version_content.replace(" - \n", &format!(" - {commit_date}\n"));
            log_debug!("Replaced empty date with commit date: {}", commit_date);
        } else if version_content.contains("] - ") && !version_content.contains("] - 20") {
            // For cases where there's no date but a dash
            let parts: Vec<&str> = version_content.splitn(2, "] - ").collect();
            if parts.len() == 2 {
                version_content = format!(
                    "{}] - {}\n{}",
                    parts[0],
                    commit_date,
                    parts[1].trim_start_matches(['\n', ' '])
                );
                log_debug!("Added commit date after dash: {}", commit_date);
            }
        } else if !version_content.contains("] - ") {
            // If no date pattern at all, find the version line and add a date
            let line_end = version_content.find('\n').unwrap_or(version_content.len());
            let version_line = &version_content[..line_end];

            if version_line.contains("## [") && version_line.contains(']') {
                // Insert the date right after the closing bracket
                let bracket_pos = version_line
                    .rfind(']')
                    .expect("Failed to find closing bracket in version line");
                version_content = format!(
                    "{} - {}{}",
                    &version_content[..=bracket_pos],
                    commit_date,
                    &version_content[bracket_pos + 1..]
                );
                log_debug!("Added date to version line: {}", commit_date);
            }
        }

        // Add a decorative separator after the version content
        let separator =
            "\n<!-- -------------------------------------------------------------- -->\n\n";
        let version_content_with_separator = format!("{version_content}{separator}");

        let updated_content = if path.exists() {
            let existing_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read changelog file: {changelog_path}"))?;

            // Check if the file already has a Keep a Changelog header
            if existing_content.contains("# Changelog")
                && existing_content.contains("Keep a Changelog")
            {
                // Split at the first version heading
                if existing_content.contains("## [") {
                    let parts: Vec<&str> = existing_content.split("## [").collect();
                    let header = parts[0];

                    // Combine header with new version content and existing versions
                    if parts.len() > 1 {
                        let existing_versions = parts[1..].join("## [");
                        format!("{header}{version_content_with_separator}## [{existing_versions}")
                    } else {
                        format!("{header}{version_content_with_separator}")
                    }
                } else {
                    // No version sections yet, just append new content
                    format!("{existing_content}{version_content_with_separator}")
                }
            } else {
                // Existing file doesn't have proper format, overwrite with default structure
                format!("{default_header}{version_content_with_separator}")
            }
        } else {
            // File doesn't exist, create new with proper header
            format!("{default_header}{version_content_with_separator}")
        };

        // Write the updated content back to the file
        let mut file = fs::File::create(path)
            .with_context(|| format!("Failed to create changelog file: {changelog_path}"))?;

        file.write_all(updated_content.as_bytes())
            .with_context(|| format!("Failed to write to changelog file: {changelog_path}"))?;

        Ok(())
    }
}

/// Strips ANSI color/style codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})*)?[m|K]")
        .expect("Failed to compile ANSI escape code regex");
    re.replace_all(s, "").to_string()
}
