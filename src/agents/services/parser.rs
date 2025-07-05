//! Response Parser Service
//!
//! Extracted from the monolithic `IrisAgent` to handle all JSON parsing and response
//! processing with robust fallback handling for malformed responses.

use anyhow::Result;
use serde::de::DeserializeOwned;

/// Response parser for handling LLM JSON responses with robust fallback handling
#[derive(Clone)]
pub struct ResponseParser;

impl ResponseParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse JSON response with comprehensive fallback handling
    pub fn parse_json_response<T>(&self, response: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        crate::log_debug!(
            "🔍 Parser: Parsing JSON response - {} chars",
            response.len()
        );
        crate::log_debug!(
            "📝 Parser: Response preview: {}",
            response.chars().take(200).collect::<String>()
        );

        // First try to parse the response directly
        crate::log_debug!("🎯 Parser: Attempting direct JSON parsing");
        if let Ok(parsed) = serde_json::from_str::<T>(response) {
            crate::log_debug!("✅ Parser: Direct JSON parsing successful");
            return Ok(parsed);
        }
        crate::log_debug!("❌ Parser: Direct JSON parsing failed, trying markdown extraction");

        // Try to extract JSON from markdown code blocks
        if let Some(json_content) = self.extract_from_markdown(response) {
            crate::log_debug!(
                "📄 Parser: Extracted JSON content - {} chars",
                json_content.trim().len()
            );
            if let Ok(parsed) = serde_json::from_str::<T>(&json_content) {
                crate::log_debug!("✅ Parser: Markdown JSON parsing successful");
                return Ok(parsed);
            }
            crate::log_debug!("❌ Parser: Markdown JSON parsing failed");
        }

        // Try to find any JSON object in the response
        if let Some(potential_json) = self.extract_json_object(response) {
            crate::log_debug!(
                "🔍 Parser: Found potential JSON object - {} chars",
                potential_json.len()
            );
            crate::log_debug!(
                "📄 Parser: Potential JSON preview: {}",
                potential_json.chars().take(100).collect::<String>()
            );
            if let Ok(parsed) = serde_json::from_str::<T>(&potential_json) {
                crate::log_debug!("✅ Parser: Extracted JSON parsing successful");
                return Ok(parsed);
            }
            crate::log_debug!("❌ Parser: Extracted JSON parsing failed");
        }

        // Last resort: try to handle truncated JSON
        if let Some(truncated_json) = self.fix_truncated_json(response) {
            crate::log_debug!(
                "🔧 Parser: Attempting to parse truncated JSON - {} chars",
                truncated_json.len()
            );
            if let Ok(parsed) = serde_json::from_str::<T>(&truncated_json) {
                crate::log_debug!("✅ Parser: Truncated JSON parsing successful");
                return Ok(parsed);
            }
        }

        crate::log_debug!("🚨 Parser: All JSON parsing attempts failed");
        Err(anyhow::anyhow!(
            "Failed to parse JSON response. Raw response: {}",
            response.chars().take(1000).collect::<String>() // Limit error message length
        ))
    }

    /// Clean JSON response by removing markdown code blocks and other formatting
    pub fn clean_json_response(&self, response: &str) -> String {
        let response = response.trim();

        // Remove markdown code blocks if present
        if response.starts_with("```json") && response.ends_with("```") {
            // Remove ```json from start and ``` from end
            let without_start = response.strip_prefix("```json").unwrap_or(response);
            let without_end = without_start.strip_suffix("```").unwrap_or(without_start);
            without_end.trim().to_string()
        } else if response.starts_with("```") && response.ends_with("```") {
            // Remove generic ``` code blocks
            let without_start = response.strip_prefix("```").unwrap_or(response);
            let without_end = without_start.strip_suffix("```").unwrap_or(without_start);
            without_end.trim().to_string()
        } else {
            response.to_string()
        }
    }

    /// Extract JSON from markdown code blocks
    pub fn extract_from_markdown(&self, response: &str) -> Option<String> {
        if let Some(json_start) = response.find("```json") {
            crate::log_debug!(
                "🔍 Parser: Found markdown JSON block at position {}",
                json_start
            );
            let content_start = json_start + 7; // Skip past "```json"
            if let Some(json_end_relative) = response[content_start..].find("```") {
                let json_end = content_start + json_end_relative;
                let json_content = &response[content_start..json_end];
                return Some(json_content.trim().to_string());
            }
            crate::log_debug!("❌ Parser: Found ```json but no closing ```");
        } else {
            crate::log_debug!("🔍 Parser: No markdown JSON blocks found");
        }
        None
    }

    /// Extract JSON object from arbitrary text
    fn extract_json_object(&self, response: &str) -> Option<String> {
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end > start {
                    let potential_json = &response[start..=end];
                    crate::log_debug!(
                        "🔍 Parser: Found potential JSON object from {} to {} - {} chars",
                        start,
                        end,
                        potential_json.len()
                    );
                    return Some(potential_json.to_string());
                }
            } else {
                crate::log_debug!("❌ Parser: Found opening {{ but no closing }}");
            }
        } else {
            crate::log_debug!("❌ Parser: No JSON objects found in response");
        }
        None
    }

    /// Try to fix truncated JSON by finding the last complete field
    fn fix_truncated_json(&self, response: &str) -> Option<String> {
        if let Some(start) = response.find('{') {
            // Find the last complete field by looking for the last closing brace before any truncation
            let mut brace_count = 0;
            let mut last_valid_end = start;

            for (i, c) in response[start..].char_indices() {
                match c {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            last_valid_end = start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if last_valid_end > start {
                let truncated_json = &response[start..=last_valid_end];
                return Some(truncated_json.to_string());
            }
        }
        None
    }

    /// Parse changelog response with special handling
    pub fn parse_changelog_response(&self, json_response: &str) -> Result<String> {
        // Clean the JSON response
        let clean_json = self.clean_json_response(json_response);

        // Parse the JSON response (with fallback for malformed JSON)
        let parsed_response: serde_json::Value = match serde_json::from_str(&clean_json) {
            Ok(json) => json,
            Err(_) => {
                // Try our robust JSON parsing as fallback
                self.parse_json_response::<serde_json::Value>(&clean_json)?
            }
        };

        // Format using the original style
        self.format_changelog_from_json(parsed_response)
    }

    /// Format changelog JSON into markdown
    fn format_changelog_from_json(&self, parsed_response: serde_json::Value) -> Result<String> {
        let mut formatted = String::new();

        // Add header (no colors in agent output for consistency)
        formatted.push_str("# Changelog\n\n");
        formatted
            .push_str("All notable changes to this project will be documented in this file.\n\n");
        formatted.push_str(
            "The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\n",
        );
        formatted.push_str("and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n");

        // Add version
        let version = parsed_response
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("Unreleased");
        use std::fmt::Write;
        write!(formatted, "## [{version}] - \n\n").unwrap();

        // Process sections in order
        let ordered_types = [
            "Added",
            "Changed",
            "Fixed",
            "Removed",
            "Deprecated",
            "Security",
        ];

        if let Some(sections) = parsed_response.get("sections").and_then(|s| s.as_object()) {
            for change_type in &ordered_types {
                if let Some(entries) = sections.get(*change_type).and_then(|e| e.as_array()) {
                    if !entries.is_empty() {
                        // Add emoji and section header
                        let emoji = match *change_type {
                            "Added" => "✨",
                            "Changed" => "🔄",
                            "Fixed" => "🐛",
                            "Removed" => "🗑️",
                            "Deprecated" => "⚠️",
                            "Security" => "🔒",
                            _ => "📝",
                        };
                        formatted.push_str(&format!("### {emoji} {change_type}\n\n"));

                        // Add entries
                        for entry in entries {
                            if let Some(description) =
                                entry.get("description").and_then(|d| d.as_str())
                            {
                                formatted.push_str(&format!("- {description}"));

                                // Add commit hashes
                                if let Some(hashes) =
                                    entry.get("commit_hashes").and_then(|h| h.as_array())
                                {
                                    let hash_strs: Vec<String> = hashes
                                        .iter()
                                        .filter_map(|h| h.as_str())
                                        .map(std::string::ToString::to_string)
                                        .collect();
                                    if !hash_strs.is_empty() {
                                        formatted.push_str(&format!(" ({})", hash_strs.join(", ")));
                                    }
                                }
                                formatted.push('\n');
                            }
                        }
                        formatted.push('\n');
                    }
                }
            }
        }

        // Add metrics
        if let Some(metrics) = parsed_response.get("metrics").and_then(|m| m.as_object()) {
            formatted.push_str("### 📊 Metrics\n\n");
            if let Some(commits) = metrics
                .get("total_commits")
                .and_then(serde_json::Value::as_u64)
            {
                formatted.push_str(&format!("- Total Commits: {commits}\n"));
            }
            if let Some(files) = metrics
                .get("files_changed")
                .and_then(serde_json::Value::as_u64)
            {
                formatted.push_str(&format!("- Files Changed: {files}\n"));
            }
            if let Some(insertions) = metrics
                .get("insertions")
                .and_then(serde_json::Value::as_u64)
            {
                formatted.push_str(&format!("- Insertions: {insertions}\n"));
            }
            if let Some(deletions) = metrics.get("deletions").and_then(serde_json::Value::as_u64) {
                formatted.push_str(&format!("- Deletions: {deletions}\n"));
            }
        }

        Ok(formatted)
    }
}

impl Default for ResponseParser {
    fn default() -> Self {
        Self::new()
    }
}
