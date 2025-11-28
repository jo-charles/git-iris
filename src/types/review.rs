//! Code review types and formatting
//!
//! This module provides markdown-based review output that lets the LLM drive
//! the review structure while we beautify it for terminal display.

use colored::Colorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// Helper to get themed colors for terminal output
mod colors {
    use crate::theme;

    pub fn accent_primary() -> (u8, u8, u8) {
        let c = theme::current().color("accent.primary");
        (c.r, c.g, c.b)
    }

    pub fn accent_secondary() -> (u8, u8, u8) {
        let c = theme::current().color("accent.secondary");
        (c.r, c.g, c.b)
    }

    pub fn accent_tertiary() -> (u8, u8, u8) {
        let c = theme::current().color("accent.tertiary");
        (c.r, c.g, c.b)
    }

    pub fn warning() -> (u8, u8, u8) {
        let c = theme::current().color("warning");
        (c.r, c.g, c.b)
    }

    pub fn error() -> (u8, u8, u8) {
        let c = theme::current().color("error");
        (c.r, c.g, c.b)
    }

    pub fn text_secondary() -> (u8, u8, u8) {
        let c = theme::current().color("text.secondary");
        (c.r, c.g, c.b)
    }

    pub fn text_dim() -> (u8, u8, u8) {
        let c = theme::current().color("text.dim");
        (c.r, c.g, c.b)
    }
}

/// Simple markdown-based review that lets the LLM determine structure
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MarkdownReview {
    /// The full markdown content of the review
    pub content: String,
}

impl MarkdownReview {
    /// Render the markdown content with `SilkCircuit` terminal styling
    pub fn format(&self) -> String {
        render_markdown_for_terminal(&self.content)
    }
}

/// Render markdown content with `SilkCircuit` terminal styling
///
/// This function parses markdown and applies our color palette for beautiful
/// terminal output. It handles:
/// - Headers (H1, H2, H3) with Electric Purple styling
/// - Bold text with Neon Cyan
/// - Code blocks with dimmed background styling
/// - Bullet lists with Coral bullets
/// - Severity badges [CRITICAL], [HIGH], etc.
#[allow(clippy::too_many_lines)]
pub fn render_markdown_for_terminal(markdown: &str) -> String {
    let mut output = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();

    for line in markdown.lines() {
        // Handle code blocks
        if line.starts_with("```") {
            if in_code_block {
                // End of code block - output it
                let dim = colors::text_secondary();
                for code_line in code_block_content.lines() {
                    writeln!(output, "  {}", code_line.truecolor(dim.0, dim.1, dim.2))
                        .expect("write to string should not fail");
                }
                code_block_content.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
            continue;
        }

        // Handle headers
        if let Some(header) = line.strip_prefix("### ") {
            let cyan = colors::accent_secondary();
            let dim = colors::text_dim();
            writeln!(
                output,
                "\n{} {} {}",
                "─".truecolor(cyan.0, cyan.1, cyan.2),
                style_header_text(header).truecolor(cyan.0, cyan.1, cyan.2).bold(),
                "─".repeat(30usize.saturating_sub(header.len())).truecolor(dim.0, dim.1, dim.2)
            )
            .expect("write to string should not fail");
        } else if let Some(header) = line.strip_prefix("## ") {
            let purple = colors::accent_primary();
            let dim = colors::text_dim();
            writeln!(
                output,
                "\n{} {} {}",
                "─".truecolor(purple.0, purple.1, purple.2),
                style_header_text(header).truecolor(purple.0, purple.1, purple.2).bold(),
                "─".repeat(32usize.saturating_sub(header.len())).truecolor(dim.0, dim.1, dim.2)
            )
            .expect("write to string should not fail");
        } else if let Some(header) = line.strip_prefix("# ") {
            // Main title - big and bold
            let purple = colors::accent_primary();
            let cyan = colors::accent_secondary();
            writeln!(
                output,
                "{}  {}  {}",
                "━━━".truecolor(purple.0, purple.1, purple.2),
                style_header_text(header).truecolor(cyan.0, cyan.1, cyan.2).bold(),
                "━━━".truecolor(purple.0, purple.1, purple.2)
            )
            .expect("write to string should not fail");
        }
        // Handle bullet points
        else if let Some(content) = line.strip_prefix("- ") {
            let coral = colors::accent_tertiary();
            let styled = style_line_content(content);
            writeln!(output, "  {} {}", "•".truecolor(coral.0, coral.1, coral.2), styled)
                .expect("write to string should not fail");
        } else if let Some(content) = line.strip_prefix("* ") {
            let coral = colors::accent_tertiary();
            let styled = style_line_content(content);
            writeln!(output, "  {} {}", "•".truecolor(coral.0, coral.1, coral.2), styled)
                .expect("write to string should not fail");
        }
        // Handle numbered lists
        else if line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". ") {
            if let Some((num, rest)) = line.split_once(". ") {
                let coral = colors::accent_tertiary();
                let styled = style_line_content(rest);
                writeln!(
                    output,
                    "  {} {}",
                    format!("{}.", num).truecolor(coral.0, coral.1, coral.2).bold(),
                    styled
                )
                .expect("write to string should not fail");
            }
        }
        // Handle empty lines
        else if line.trim().is_empty() {
            output.push('\n');
        }
        // Regular paragraph text
        else {
            let styled = style_line_content(line);
            writeln!(output, "{styled}").expect("write to string should not fail");
        }
    }

    output
}

/// Style header text - uppercase and clean
fn style_header_text(text: &str) -> String {
    text.to_uppercase()
}

/// Style inline content - handles bold, code, severity badges
#[allow(clippy::too_many_lines)]
fn style_line_content(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut current_text = String::new();

    // Get theme colors once for efficiency
    let text_color = colors::text_secondary();
    let error_color = colors::error();
    let warning_color = colors::warning();
    let coral_color = colors::accent_tertiary();
    let cyan_color = colors::accent_secondary();

    while let Some(ch) = chars.next() {
        match ch {
            // Handle severity badges [CRITICAL], [HIGH], [MEDIUM], [LOW]
            '[' => {
                // Flush current text
                if !current_text.is_empty() {
                    result.push_str(
                        &current_text.truecolor(text_color.0, text_color.1, text_color.2).to_string(),
                    );
                    current_text.clear();
                }

                // Collect badge content
                let mut badge = String::new();
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                    badge.push(c);
                }

                // Style based on severity
                let badge_upper = badge.to_uppercase();
                let styled_badge = match badge_upper.as_str() {
                    "CRITICAL" => format!(
                        "[{}]",
                        "CRITICAL".truecolor(error_color.0, error_color.1, error_color.2).bold()
                    ),
                    "HIGH" => format!(
                        "[{}]",
                        "HIGH".truecolor(error_color.0, error_color.1, error_color.2).bold()
                    ),
                    "MEDIUM" => format!(
                        "[{}]",
                        "MEDIUM".truecolor(warning_color.0, warning_color.1, warning_color.2).bold()
                    ),
                    "LOW" => format!(
                        "[{}]",
                        "LOW".truecolor(coral_color.0, coral_color.1, coral_color.2).bold()
                    ),
                    _ => format!(
                        "[{}]",
                        badge.truecolor(cyan_color.0, cyan_color.1, cyan_color.2)
                    ),
                };
                result.push_str(&styled_badge);
            }
            // Handle bold text **text**
            '*' if chars.peek() == Some(&'*') => {
                // Flush current text
                if !current_text.is_empty() {
                    result.push_str(
                        &current_text.truecolor(text_color.0, text_color.1, text_color.2).to_string(),
                    );
                    current_text.clear();
                }

                chars.next(); // consume second *

                // Collect bold content
                let mut bold = String::new();
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'*') {
                        chars.next(); // consume closing **
                        break;
                    }
                    bold.push(c);
                }

                result.push_str(
                    &bold.truecolor(cyan_color.0, cyan_color.1, cyan_color.2).bold().to_string(),
                );
            }
            // Handle inline code `code`
            '`' => {
                // Flush current text
                if !current_text.is_empty() {
                    result.push_str(
                        &current_text.truecolor(text_color.0, text_color.1, text_color.2).to_string(),
                    );
                    current_text.clear();
                }

                // Collect code content
                let mut code = String::new();
                for c in chars.by_ref() {
                    if c == '`' {
                        break;
                    }
                    code.push(c);
                }

                result.push_str(
                    &code.truecolor(warning_color.0, warning_color.1, warning_color.2).to_string(),
                );
            }
            _ => {
                current_text.push(ch);
            }
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        result.push_str(
            &current_text.truecolor(text_color.0, text_color.1, text_color.2).to_string(),
        );
    }

    result
}
