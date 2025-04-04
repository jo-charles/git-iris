use colored::Colorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use textwrap;

/// Width in characters for wrapping explanations in code reviews
const EXPLANATION_WRAP_WIDTH: usize = 80;

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
    /// Formats a location string to ensure it includes file reference when possible
    ///
    /// Intelligently formats location strings by detecting whether they already
    /// contain a file reference or just line numbers.
    pub fn format_location(location: &str) -> String {
        if location.contains(':')
            || location.to_lowercase().contains(".rs")
            || location.to_lowercase().contains(".ts")
            || location.to_lowercase().contains(".js")
            || location.to_lowercase().contains("file")
        {
            // This is likely a file reference
            location.to_string()
        } else if location.to_lowercase().contains("line") {
            // This already mentions line numbers
            location.to_string()
        } else {
            // Treat as just line numbers - explicitly mention it's line numbers
            format!("Line(s) {location}")
        }
    }

    /// Formats the review into a readable string with colors and emojis for terminal display
    pub fn format(&self) -> String {
        let mut formatted = String::new();

        formatted.push_str(&format!(
            "{}\n\n{}\n\n",
            "✧･ﾟ: *✧･ﾟ CODE REVIEW ✧･ﾟ: *✧･ﾟ".bright_magenta().bold(),
            self.summary.bright_white()
        ));

        formatted.push_str(&format!(
            "{}\n\n{}\n\n",
            "◤ QUALITY ASSESSMENT ◢".bright_cyan().bold(),
            self.code_quality.bright_white()
        ));

        if !self.positive_aspects.is_empty() {
            formatted.push_str(&format!("{}\n\n", "✅ STRENGTHS //".green().bold()));
            for aspect in &self.positive_aspects {
                formatted.push_str(&format!("  {} {}\n", "•".bright_green(), aspect.green()));
            }
            formatted.push('\n');
        }

        if !self.issues.is_empty() {
            formatted.push_str(&format!("{}\n\n", "⚠️ CORE ISSUES //".yellow().bold()));
            for issue in &self.issues {
                formatted.push_str(&format!("  {} {}\n", "•".bright_yellow(), issue.yellow()));
            }
            formatted.push('\n');
        }

        // Format the dimension-specific analyses if they exist
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Complexity,
            self.complexity.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Abstraction,
            self.abstraction.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Deletion,
            self.deletion.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Hallucination,
            self.hallucination.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Style,
            self.style.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Security,
            self.security.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Performance,
            self.performance.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Duplication,
            self.duplication.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::ErrorHandling,
            self.error_handling.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Testing,
            self.testing.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::BestPractices,
            self.best_practices.as_ref(),
        );

        if !self.suggestions.is_empty() {
            formatted.push_str(&format!("{}\n\n", "💡 SUGGESTIONS //".bright_blue().bold()));
            for suggestion in &self.suggestions {
                formatted.push_str(&format!(
                    "  {} {}\n",
                    "•".bright_cyan(),
                    suggestion.bright_blue()
                ));
            }
        }

        formatted
    }

    /// Helper method to format a single dimension analysis
    fn format_dimension_analysis(
        formatted: &mut String,
        dimension: QualityDimension,
        analysis: Option<&DimensionAnalysis>,
    ) {
        if let Some(dim) = analysis {
            if dim.issues_found && !dim.issues.is_empty() {
                // Choose emoji based on the dimension
                let (emoji, color_fn) = match dimension {
                    QualityDimension::Complexity => ("🧠", "bright_magenta"),
                    QualityDimension::Abstraction => ("🏗️", "bright_cyan"),
                    QualityDimension::Deletion => ("🗑️", "bright_white"),
                    QualityDimension::Hallucination => ("👻", "bright_magenta"),
                    QualityDimension::Style => ("🎨", "bright_blue"),
                    QualityDimension::Security => ("🔒", "bright_red"),
                    QualityDimension::Performance => ("⚡", "bright_yellow"),
                    QualityDimension::Duplication => ("🔄", "bright_cyan"),
                    QualityDimension::ErrorHandling => ("🧯", "bright_red"),
                    QualityDimension::Testing => ("🧪", "bright_green"),
                    QualityDimension::BestPractices => ("📐", "bright_blue"),
                };

                let title = dimension.display_name();
                let header = match color_fn {
                    "bright_magenta" => format!("◤ {emoji} {title} ◢").bright_magenta().bold(),
                    "bright_cyan" => format!("◤ {emoji} {title} ◢").bright_cyan().bold(),
                    "bright_white" => format!("◤ {emoji} {title} ◢").bright_white().bold(),
                    "bright_blue" => format!("◤ {emoji} {title} ◢").bright_blue().bold(),
                    "bright_red" => format!("◤ {emoji} {title} ◢").bright_red().bold(),
                    "bright_yellow" => format!("◤ {emoji} {title} ◢").bright_yellow().bold(),
                    "bright_green" => format!("◤ {emoji} {title} ◢").bright_green().bold(),
                    _ => format!("◤ {emoji} {title} ◢").normal().bold(),
                };

                formatted.push_str(&format!("{header}\n\n"));

                for (i, issue) in dim.issues.iter().enumerate() {
                    // Severity badge with custom styling based on severity
                    let severity_badge = match issue.severity.as_str() {
                        "Critical" => format!("[{}]", "CRITICAL".bright_red().bold()),
                        "High" => format!("[{}]", "HIGH".red().bold()),
                        "Medium" => format!("[{}]", "MEDIUM".yellow().bold()),
                        "Low" => format!("[{}]", "LOW".bright_yellow().bold()),
                        _ => format!("[{}]", issue.severity.normal().bold()),
                    };

                    formatted.push_str(&format!(
                        "  {} {} {}\n",
                        format!("{:02}", i + 1).bright_white().bold(),
                        severity_badge,
                        issue.description.bright_white()
                    ));

                    let formatted_location = Self::format_location(&issue.location).bright_white();
                    formatted.push_str(&format!(
                        "     {}: {}\n",
                        "LOCATION".bright_cyan().bold(),
                        formatted_location
                    ));

                    // Format explanation with some spacing for readability
                    let explanation_lines =
                        textwrap::wrap(&issue.explanation, EXPLANATION_WRAP_WIDTH);
                    formatted.push_str(&format!("     {}: ", "DETAIL".bright_cyan().bold()));
                    for (i, line) in explanation_lines.iter().enumerate() {
                        if i == 0 {
                            formatted.push_str(&format!("{line}\n"));
                        } else {
                            formatted.push_str(&format!("            {line}\n"));
                        }
                    }

                    // Format recommendation with a different style
                    formatted.push_str(&format!(
                        "     {}: {}\n\n",
                        "FIX".bright_green().bold(),
                        issue.recommendation.bright_green()
                    ));
                }
            }
        }
    }
}

use super::service::IrisCommitService;
use crate::common::CommonParams;
use crate::config::Config;
use crate::git::GitRepo;
use crate::instruction_presets::PresetType;
use crate::messages;
use crate::ui;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Handles the review command which generates an AI code review of staged changes
/// with comprehensive analysis across multiple dimensions of code quality
pub async fn handle_review_command(
    common: CommonParams,
    _print: bool,
    repository_url: Option<String>,
) -> Result<()> {
    // Check if the preset is appropriate for code reviews
    if !common.is_valid_preset_for_type(PresetType::Review) {
        ui::print_warning(
            "The specified preset may not be suitable for code reviews. Consider using a review or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for reviews.");
    }

    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    // Combine repository URL from CLI and CommonParams
    let repo_url = repository_url.or(common.repository_url.clone());

    // Create the git repository
    let git_repo = GitRepo::new_from_url(repo_url).context("Failed to create GitRepo")?;

    let repo_path = git_repo.repo_path().clone();
    let provider_name = &config.default_provider;

    let service = Arc::new(
        IrisCommitService::new(
            config.clone(),
            &repo_path,
            provider_name,
            false, // gitmoji not needed for review
            false, // verification not needed for review
            git_repo,
        )
        .context("Failed to create IrisCommitService")?,
    );

    // Check environment prerequisites
    if let Err(e) = service.check_environment() {
        ui::print_error(&format!("Error: {e}"));
        ui::print_info("\nPlease ensure the following:");
        ui::print_info("1. Git is installed and accessible from the command line.");
        ui::print_info(
            "2. You are running this command from within a Git repository or provide a repository URL with --repo.",
        );
        ui::print_info("3. You have set up your configuration using 'git-iris config'.");
        return Err(e);
    }

    let git_info = service.get_git_info().await?;

    if git_info.staged_files.is_empty() {
        ui::print_warning(
            "No staged changes. Please stage your changes before generating a review.",
        );
        ui::print_info("You can stage changes using 'git add <file>' or 'git add .'");
        return Ok(());
    }

    let effective_instructions = common
        .instructions
        .unwrap_or_else(|| config.instructions.clone());
    let preset_str = common.preset.as_deref().unwrap_or("");

    // Create and start the spinner
    let spinner = ui::create_spinner("");
    let random_message = messages::get_review_waiting_message();
    spinner.set_message(random_message.text.to_string());

    // Generate the code review
    let review = service
        .generate_review(preset_str, &effective_instructions)
        .await?;

    // Stop the spinner
    spinner.finish_and_clear();

    // Print the review to stdout or save to file if requested
    println!("{}", review.format());

    Ok(())
}
