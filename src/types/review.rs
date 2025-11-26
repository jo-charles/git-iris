//! Code review types and formatting
//!
//! This module provides markdown-based review output that lets the LLM drive
//! the review structure while we beautify it for terminal display.

use crate::ui::rgb::{
    CORAL, DIM_SEPARATOR, DIM_WHITE, ELECTRIC_PURPLE, ELECTRIC_YELLOW, ERROR_RED, NEON_CYAN,
};
use colored::Colorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

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
                for code_line in code_block_content.lines() {
                    writeln!(
                        output,
                        "  {}",
                        code_line.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                    )
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
            writeln!(
                output,
                "\n{} {} {}",
                "─".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2),
                style_header_text(header)
                    .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                    .bold(),
                "─".repeat(30usize.saturating_sub(header.len())).truecolor(
                    DIM_SEPARATOR.0,
                    DIM_SEPARATOR.1,
                    DIM_SEPARATOR.2
                )
            )
            .expect("write to string should not fail");
        } else if let Some(header) = line.strip_prefix("## ") {
            writeln!(
                output,
                "\n{} {} {}",
                "─".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
                style_header_text(header)
                    .truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2)
                    .bold(),
                "─".repeat(32usize.saturating_sub(header.len())).truecolor(
                    DIM_SEPARATOR.0,
                    DIM_SEPARATOR.1,
                    DIM_SEPARATOR.2
                )
            )
            .expect("write to string should not fail");
        } else if let Some(header) = line.strip_prefix("# ") {
            // Main title - big and bold
            writeln!(
                output,
                "{}  {}  {}",
                "━━━".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
                style_header_text(header)
                    .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                    .bold(),
                "━━━".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2)
            )
            .expect("write to string should not fail");
        }
        // Handle bullet points
        else if let Some(content) = line.strip_prefix("- ") {
            let styled = style_line_content(content);
            writeln!(
                output,
                "  {} {}",
                "•".truecolor(CORAL.0, CORAL.1, CORAL.2),
                styled
            )
            .expect("write to string should not fail");
        } else if let Some(content) = line.strip_prefix("* ") {
            let styled = style_line_content(content);
            writeln!(
                output,
                "  {} {}",
                "•".truecolor(CORAL.0, CORAL.1, CORAL.2),
                styled
            )
            .expect("write to string should not fail");
        }
        // Handle numbered lists
        else if line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". ") {
            if let Some((num, rest)) = line.split_once(". ") {
                let styled = style_line_content(rest);
                writeln!(
                    output,
                    "  {} {}",
                    format!("{}.", num)
                        .truecolor(CORAL.0, CORAL.1, CORAL.2)
                        .bold(),
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

    while let Some(ch) = chars.next() {
        match ch {
            // Handle severity badges [CRITICAL], [HIGH], [MEDIUM], [LOW]
            '[' => {
                // Flush current text
                if !current_text.is_empty() {
                    result.push_str(
                        &current_text
                            .truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                            .to_string(),
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
                        "CRITICAL"
                            .truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2)
                            .bold()
                    ),
                    "HIGH" => format!(
                        "[{}]",
                        "HIGH"
                            .truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2)
                            .bold()
                    ),
                    "MEDIUM" => format!(
                        "[{}]",
                        "MEDIUM"
                            .truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
                            .bold()
                    ),
                    "LOW" => format!("[{}]", "LOW".truecolor(CORAL.0, CORAL.1, CORAL.2).bold()),
                    _ => format!(
                        "[{}]",
                        badge.truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                    ),
                };
                result.push_str(&styled_badge);
            }
            // Handle bold text **text**
            '*' if chars.peek() == Some(&'*') => {
                // Flush current text
                if !current_text.is_empty() {
                    result.push_str(
                        &current_text
                            .truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                            .to_string(),
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
                    &bold
                        .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                        .bold()
                        .to_string(),
                );
            }
            // Handle inline code `code`
            '`' => {
                // Flush current text
                if !current_text.is_empty() {
                    result.push_str(
                        &current_text
                            .truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                            .to_string(),
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
                    &code
                        .truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
                        .to_string(),
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
            &current_text
                .truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                .to_string(),
        );
    }

    result
}
