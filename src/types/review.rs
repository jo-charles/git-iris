//! Code review types and formatting
//!
//! This module provides markdown-based review output that lets the LLM drive
//! the review structure while we beautify it for terminal display.

use crate::ui::rgb::{
    CORAL, DIM_SEPARATOR, DIM_WHITE, ELECTRIC_PURPLE, ELECTRIC_YELLOW, ERROR_RED, NEON_CYAN,
    SUCCESS_GREEN,
};
use colored::Colorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use textwrap;

/// Width in characters for wrapping explanations in code reviews
const EXPLANATION_WRAP_WIDTH: usize = 80;

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: Markdown-based review (LLM-driven structure)
// ═══════════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════════
// LEGACY: Structured review types (kept for backwards compatibility)
// ═══════════════════════════════════════════════════════════════════════════════

/// Represents a specific issue found during code review
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CodeIssue {
    /// Brief description of the issue
    pub description: String,
    /// Severity level of the issue (Critical, High, Medium, Low)
    pub severity: String,
    /// Location of the issue, preferably in "`filename.rs:line_numbers`" format
    /// or "`path/to/file.rs:line_numbers`" format for better readability
    pub location: String,
    /// Detailed explanation of why this is problematic
    pub explanation: String,
    /// Specific suggestion to address the issue
    pub recommendation: String,
}

/// Represents analysis for a specific code quality dimension
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct DimensionAnalysis {
    /// Whether issues were found in this dimension
    pub issues_found: bool,
    /// List of specific issues identified in this dimension
    pub issues: Vec<CodeIssue>,
}

/// Represents the different dimensions of code quality analysis
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum QualityDimension {
    /// Unnecessary complexity in algorithms, abstractions, or control flow
    Complexity,
    /// Poor or inappropriate abstractions, design patterns or separation of concerns
    Abstraction,
    /// Unintended deletion of code or functionality without proper replacement
    Deletion,
    /// References to non-existent components, APIs, or behaviors
    Hallucination,
    /// Inconsistencies in code style, naming, or formatting
    Style,
    /// Security vulnerabilities or insecure coding practices
    Security,
    /// Inefficient algorithms, operations, or resource usage
    Performance,
    /// Repeated logic, functionality, or copy-pasted code
    Duplication,
    /// Insufficient or improper error handling and recovery
    ErrorHandling,
    /// Gaps in test coverage or tests that miss critical scenarios
    Testing,
    /// Violations of established best practices or coding standards
    BestPractices,
}

impl QualityDimension {
    /// Get all quality dimensions
    pub fn all() -> &'static [QualityDimension] {
        &[
            QualityDimension::Complexity,
            QualityDimension::Abstraction,
            QualityDimension::Deletion,
            QualityDimension::Hallucination,
            QualityDimension::Style,
            QualityDimension::Security,
            QualityDimension::Performance,
            QualityDimension::Duplication,
            QualityDimension::ErrorHandling,
            QualityDimension::Testing,
            QualityDimension::BestPractices,
        ]
    }

    /// Get the display name for a dimension
    pub fn display_name(&self) -> &'static str {
        match self {
            QualityDimension::Complexity => "Complexity",
            QualityDimension::Abstraction => "Abstraction",
            QualityDimension::Deletion => "Unintended Deletion",
            QualityDimension::Hallucination => "Hallucinated Components",
            QualityDimension::Style => "Style Inconsistencies",
            QualityDimension::Security => "Security Vulnerabilities",
            QualityDimension::Performance => "Performance Issues",
            QualityDimension::Duplication => "Code Duplication",
            QualityDimension::ErrorHandling => "Error Handling",
            QualityDimension::Testing => "Test Coverage",
            QualityDimension::BestPractices => "Best Practices",
        }
    }

    /// Get the description for a dimension
    #[allow(clippy::too_many_lines)]
    pub fn description(&self) -> &'static str {
        match self {
            QualityDimension::Complexity => {
                "
            **Unnecessary Complexity**
            - Overly complex algorithms or functions
            - Unnecessary abstraction layers
            - Convoluted control flow
            - Functions/methods that are too long or have too many parameters
            - Nesting levels that are too deep
            "
            }
            QualityDimension::Abstraction => {
                "
            **Poor Abstractions**
            - Inappropriate use of design patterns
            - Missing abstractions where needed
            - Leaky abstractions that expose implementation details
            - Overly generic abstractions that add complexity
            - Unclear separation of concerns
            "
            }
            QualityDimension::Deletion => {
                "
            **Unintended Code Deletion**
            - Critical functionality removed without replacement
            - Incomplete removal of deprecated code
            - Breaking changes to public APIs
            - Removed error handling or validation
            - Missing edge case handling present in original code
            "
            }
            QualityDimension::Hallucination => {
                "
            **Hallucinated Components**
            - References to non-existent functions, classes, or modules
            - Assumptions about available libraries or APIs
            - Inconsistent or impossible behavior expectations
            - References to frameworks or patterns not used in the project
            - Creation of interfaces that don't align with the codebase
            "
            }
            QualityDimension::Style => {
                "
            **Style Inconsistencies**
            - Deviation from project coding standards
            - Inconsistent naming conventions
            - Inconsistent formatting or indentation
            - Inconsistent comment styles or documentation
            - Mixing of different programming paradigms
            "
            }
            QualityDimension::Security => {
                "
            **Security Vulnerabilities**
            - Injection vulnerabilities (SQL, Command, etc.)
            - Insecure data handling or storage
            - Authentication or authorization flaws
            - Exposure of sensitive information
            - Unsafe dependencies or API usage
            "
            }
            QualityDimension::Performance => {
                "
            **Performance Issues**
            - Inefficient algorithms or data structures
            - Unnecessary computations or operations
            - Resource leaks (memory, file handles, etc.)
            - Excessive network or disk operations
            - Blocking operations in asynchronous code
            "
            }
            QualityDimension::Duplication => {
                "
            **Code Duplication**
            - Repeated logic or functionality
            - Copy-pasted code with minor variations
            - Duplicate functionality across different modules
            - Redundant validation or error handling
            - Parallel hierarchies or structures
            "
            }
            QualityDimension::ErrorHandling => {
                "
            **Incomplete Error Handling**
            - Missing try-catch blocks for risky operations
            - Overly broad exception handling
            - Swallowed exceptions without proper logging
            - Unclear error messages or codes
            - Inconsistent error recovery strategies
            "
            }
            QualityDimension::Testing => {
                "
            **Test Coverage Gaps**
            - Missing unit tests for critical functionality
            - Uncovered edge cases or error paths
            - Brittle tests that make inappropriate assumptions
            - Missing integration or system tests
            - Tests that don't verify actual requirements
            "
            }
            QualityDimension::BestPractices => {
                "
            **Best Practices Violations**
            - Not following language-specific idioms and conventions
            - Violation of SOLID principles or other design guidelines
            - Anti-patterns or known problematic implementation approaches
            - Ignored compiler/linter warnings
            - Outdated or deprecated APIs and practices
            "
            }
        }
    }
}

/// Model for code review generation results
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct GeneratedReview {
    /// Brief summary of the code changes and overall review
    pub summary: String,
    /// Detailed assessment of the overall code quality
    pub code_quality: String,
    /// List of specific suggestions for improving the code
    pub suggestions: Vec<String>,
    /// List of identified issues or problems in the code
    pub issues: Vec<String>,
    /// List of positive aspects or good practices in the code
    pub positive_aspects: Vec<String>,
    /// Analysis of unnecessary complexity issues
    pub complexity: Option<DimensionAnalysis>,
    /// Analysis of abstraction quality issues
    pub abstraction: Option<DimensionAnalysis>,
    /// Analysis of unintended code deletion
    pub deletion: Option<DimensionAnalysis>,
    /// Analysis of hallucinated components that don't exist
    pub hallucination: Option<DimensionAnalysis>,
    /// Analysis of style inconsistencies
    pub style: Option<DimensionAnalysis>,
    /// Analysis of security vulnerabilities
    pub security: Option<DimensionAnalysis>,
    /// Analysis of performance issues
    pub performance: Option<DimensionAnalysis>,
    /// Analysis of code duplication
    pub duplication: Option<DimensionAnalysis>,
    /// Analysis of error handling completeness
    pub error_handling: Option<DimensionAnalysis>,
    /// Analysis of test coverage gaps
    pub testing: Option<DimensionAnalysis>,
    /// Analysis of best practices violations
    pub best_practices: Option<DimensionAnalysis>,
}

impl GeneratedReview {
    /// Validates if the location string is parseable for better error handling
    pub fn format_location(location: &str) -> String {
        // If it already contains keywords like "line", "file", or "in", return as-is
        if location.to_lowercase().contains("line")
            || location.to_lowercase().contains("file")
            || location.to_lowercase().contains(" in ")
        {
            return location.to_string();
        }

        // If it looks like a file path (contains "/" or "\\" and ":"), return as-is
        if location.contains(':') && (location.contains('/') || location.contains('\\')) {
            location.to_string()
        } else if location.contains(':') {
            // Treat as file:line_numbers format without path separators
            format!("in {location}")
        } else if location.contains('.')
            && location
                .split('.')
                .next_back()
                .is_some_and(|ext| !ext.is_empty())
        {
            // Looks like a filename with extension, return as-is
            location.to_string()
        } else {
            // Treat as just line numbers - explicitly mention it's line numbers
            format!("Line(s) {location}")
        }
    }

    /// Formats the review into a readable string with colors and emojis for terminal display
    pub fn format(&self) -> String {
        let mut formatted = String::new();

        Self::format_header(&mut formatted, &self.summary, &self.code_quality);
        Self::format_positive_aspects(&mut formatted, &self.positive_aspects);
        Self::format_issues(&mut formatted, &self.issues);
        Self::format_all_dimension_analyses(&mut formatted, self);
        Self::format_suggestions(&mut formatted, &self.suggestions);

        formatted
    }

    /// Formats the header section with title, summary, and quality assessment
    fn format_header(formatted: &mut String, summary: &str, code_quality: &str) {
        // SilkCircuit header
        write!(
            formatted,
            "{}  {}  {}\n\n",
            "━━━".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
            "CODE REVIEW"
                .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                .bold(),
            "━━━".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2)
        )
        .expect("write to string should not fail");

        // Summary
        writeln!(
            formatted,
            "{}",
            summary.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
        )
        .expect("write to string should not fail");

        // Quality assessment section
        write!(
            formatted,
            "\n{} {} {}\n\n{}\n\n",
            "─".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
            "QUALITY"
                .truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2)
                .bold(),
            "─"
                .repeat(30)
                .truecolor(DIM_SEPARATOR.0, DIM_SEPARATOR.1, DIM_SEPARATOR.2),
            code_quality.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
        )
        .expect("write to string should not fail");
    }

    /// Formats the positive aspects section
    fn format_positive_aspects(formatted: &mut String, positive_aspects: &[String]) {
        if !positive_aspects.is_empty() {
            write!(
                formatted,
                "{} {} {}\n\n",
                "─".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2),
                "STRENGTHS"
                    .truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2)
                    .bold(),
                "─"
                    .repeat(28)
                    .truecolor(DIM_SEPARATOR.0, DIM_SEPARATOR.1, DIM_SEPARATOR.2)
            )
            .expect("write to string should not fail");
            for aspect in positive_aspects {
                writeln!(
                    formatted,
                    "  {} {}",
                    "•".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2),
                    aspect.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                )
                .expect("write to string should not fail");
            }
            formatted.push('\n');
        }
    }

    /// Formats the issues section
    fn format_issues(formatted: &mut String, issues: &[String]) {
        if !issues.is_empty() {
            write!(
                formatted,
                "{} {} {}\n\n",
                "─".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
                "ISSUES"
                    .truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
                    .bold(),
                "─"
                    .repeat(30)
                    .truecolor(DIM_SEPARATOR.0, DIM_SEPARATOR.1, DIM_SEPARATOR.2)
            )
            .expect("write to string should not fail");
            for issue in issues {
                writeln!(
                    formatted,
                    "  {} {}",
                    "•".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
                    issue.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                )
                .expect("write to string should not fail");
            }
            formatted.push('\n');
        }
    }

    /// Formats all dimension-specific analyses
    fn format_all_dimension_analyses(formatted: &mut String, review: &GeneratedReview) {
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Complexity,
            review.complexity.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Abstraction,
            review.abstraction.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Deletion,
            review.deletion.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Hallucination,
            review.hallucination.as_ref(),
        );
        Self::format_dimension_analysis(formatted, QualityDimension::Style, review.style.as_ref());
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Security,
            review.security.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Performance,
            review.performance.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Duplication,
            review.duplication.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::ErrorHandling,
            review.error_handling.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::Testing,
            review.testing.as_ref(),
        );
        Self::format_dimension_analysis(
            formatted,
            QualityDimension::BestPractices,
            review.best_practices.as_ref(),
        );
    }

    /// Formats the suggestions section
    fn format_suggestions(formatted: &mut String, suggestions: &[String]) {
        if !suggestions.is_empty() {
            write!(
                formatted,
                "{} {} {}\n\n",
                "─".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2),
                "SUGGESTIONS"
                    .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                    .bold(),
                "─"
                    .repeat(26)
                    .truecolor(DIM_SEPARATOR.0, DIM_SEPARATOR.1, DIM_SEPARATOR.2)
            )
            .expect("write to string should not fail");
            for suggestion in suggestions {
                writeln!(
                    formatted,
                    "  {} {}",
                    "•".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2),
                    suggestion.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                )
                .expect("write to string should not fail");
            }
        }
    }

    /// Get color for severity level
    fn severity_badge(severity: &str) -> String {
        match severity {
            "Critical" => format!(
                "[{}]",
                "CRITICAL"
                    .truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2)
                    .bold()
            ),
            "High" => format!(
                "[{}]",
                "HIGH"
                    .truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2)
                    .bold()
            ),
            "Medium" => format!(
                "[{}]",
                "MEDIUM"
                    .truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
                    .bold()
            ),
            "Low" => format!("[{}]", "LOW".truecolor(CORAL.0, CORAL.1, CORAL.2).bold()),
            other => format!(
                "[{}]",
                other
                    .truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                    .bold()
            ),
        }
    }

    /// Get color for a quality dimension
    fn dimension_color(dimension: QualityDimension) -> (u8, u8, u8) {
        match dimension {
            QualityDimension::Security | QualityDimension::ErrorHandling => ERROR_RED,
            QualityDimension::Performance => ELECTRIC_YELLOW,
            QualityDimension::Testing => SUCCESS_GREEN,
            QualityDimension::Complexity
            | QualityDimension::Hallucination
            | QualityDimension::Style => ELECTRIC_PURPLE,
            _ => NEON_CYAN,
        }
    }

    /// Format a single issue
    fn format_issue(formatted: &mut String, index: usize, issue: &CodeIssue, color: (u8, u8, u8)) {
        // Issue header with severity
        writeln!(
            formatted,
            "  {} {} {}",
            format!("{:02}", index + 1)
                .truecolor(color.0, color.1, color.2)
                .bold(),
            Self::severity_badge(&issue.severity),
            issue
                .description
                .truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
        )
        .expect("write to string should not fail");

        // Location
        let formatted_location = Self::format_location(&issue.location);
        writeln!(
            formatted,
            "     {}: {}",
            "LOCATION"
                .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                .bold(),
            formatted_location.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
        )
        .expect("write to string should not fail");

        // Explanation (wrapped)
        let explanation_lines = textwrap::wrap(&issue.explanation, EXPLANATION_WRAP_WIDTH);
        write!(
            formatted,
            "     {}: ",
            "DETAIL"
                .truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
                .bold()
        )
        .expect("write to string should not fail");
        for (i, line) in explanation_lines.iter().enumerate() {
            if i == 0 {
                writeln!(
                    formatted,
                    "{}",
                    line.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                )
                .expect("write to string should not fail");
            } else {
                writeln!(
                    formatted,
                    "            {}",
                    line.truecolor(DIM_WHITE.0, DIM_WHITE.1, DIM_WHITE.2)
                )
                .expect("write to string should not fail");
            }
        }

        // Recommendation
        write!(
            formatted,
            "     {}: {}\n\n",
            "FIX"
                .truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2)
                .bold(),
            issue
                .recommendation
                .truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2)
        )
        .expect("write to string should not fail");
    }

    /// Helper method to format a single dimension analysis
    fn format_dimension_analysis(
        formatted: &mut String,
        dimension: QualityDimension,
        analysis: Option<&DimensionAnalysis>,
    ) {
        let Some(dim) = analysis else { return };
        if !dim.issues_found || dim.issues.is_empty() {
            return;
        }

        let color = Self::dimension_color(dimension);
        let title = dimension.display_name().to_uppercase();
        let padding = 34usize.saturating_sub(title.len());

        // Section header
        write!(
            formatted,
            "{} {} {}\n\n",
            "─".truecolor(color.0, color.1, color.2),
            title.truecolor(color.0, color.1, color.2).bold(),
            "─"
                .repeat(padding)
                .truecolor(DIM_SEPARATOR.0, DIM_SEPARATOR.1, DIM_SEPARATOR.2)
        )
        .expect("write to string should not fail");

        for (i, issue) in dim.issues.iter().enumerate() {
            Self::format_issue(formatted, i, issue, color);
        }
    }
}
