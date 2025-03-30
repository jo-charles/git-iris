use colored::Colorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a specific issue found during code review
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct CodeIssue {
    /// Brief description of the issue
    pub description: String,
    /// Severity level of the issue (Critical, High, Medium, Low)
    pub severity: String,
    /// Line numbers or location references for the issue
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

impl From<String> for GeneratedReview {
    fn from(s: String) -> Self {
        match serde_json::from_str(&s) {
            Ok(review) => review,
            Err(e) => {
                crate::log_debug!("Failed to parse review JSON: {}", e);
                crate::log_debug!("Input was: {}", s);
                Self {
                    summary: "Error parsing code review".to_string(),
                    code_quality: "There was an error parsing the code review from the AI."
                        .to_string(),
                    suggestions: vec!["Please try again.".to_string()],
                    issues: vec![],
                    positive_aspects: vec![],
                    complexity: None,
                    abstraction: None,
                    deletion: None,
                    hallucination: None,
                    style: None,
                    security: None,
                    performance: None,
                    duplication: None,
                    error_handling: None,
                    testing: None,
                    best_practices: None,
                }
            }
        }
    }
}

impl GeneratedReview {
    /// Formats the review into a readable string with colors and emojis for terminal display
    pub fn format(&self) -> String {
        let mut formatted = String::new();

        formatted.push_str(&format!(
            "{}\n\n{}\n\n",
            "✨ Code Review Summary ✨".bright_magenta().bold(),
            self.summary.bright_white()
        ));

        formatted.push_str(&format!(
            "{}\n\n{}\n\n",
            "🔍 Code Quality Assessment".bright_cyan().bold(),
            self.code_quality.bright_white()
        ));

        if !self.positive_aspects.is_empty() {
            formatted.push_str(&format!("{}\n\n", "✅ Positive Aspects".green().bold()));
            for (i, aspect) in self.positive_aspects.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, aspect.green()));
            }
            formatted.push('\n');
        }

        if !self.issues.is_empty() {
            formatted.push_str(&format!("{}\n\n", "❌ Issues Identified".yellow().bold()));
            for (i, issue) in self.issues.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, issue.yellow()));
            }
            formatted.push('\n');
        }

        // Format the dimension-specific analyses if they exist
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Complexity.display_name(),
            self.complexity.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Abstraction.display_name(),
            self.abstraction.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Deletion.display_name(),
            self.deletion.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Hallucination.display_name(),
            self.hallucination.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Style.display_name(),
            self.style.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Security.display_name(),
            self.security.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Performance.display_name(),
            self.performance.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Duplication.display_name(),
            self.duplication.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::ErrorHandling.display_name(),
            self.error_handling.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::Testing.display_name(),
            self.testing.as_ref(),
        );
        Self::format_dimension_analysis(
            &mut formatted,
            QualityDimension::BestPractices.display_name(),
            self.best_practices.as_ref(),
        );

        if !self.suggestions.is_empty() {
            formatted.push_str(&format!(
                "{}\n\n",
                "💡 Suggestions for Improvement".bright_blue().bold()
            ));
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, suggestion.bright_blue()));
            }
        }

        formatted
    }

    /// Helper method to format a single dimension analysis
    fn format_dimension_analysis(
        formatted: &mut String,
        title: &str,
        analysis: Option<&DimensionAnalysis>,
    ) {
        if let Some(dim) = analysis {
            if dim.issues_found && !dim.issues.is_empty() {
                formatted.push_str(&format!("{}\n\n", format!("🔎 {title}").yellow().bold()));

                for (i, issue) in dim.issues.iter().enumerate() {
                    let severity_color = match issue.severity.as_str() {
                        "Critical" => issue.description.bright_red(),
                        "High" => issue.description.red(),
                        "Medium" => issue.description.yellow(),
                        "Low" => issue.description.bright_yellow(),
                        _ => issue.description.normal(),
                    };

                    formatted.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        severity_color,
                        issue.severity
                    ));
                    formatted
                        .push_str(&format!("   Location: {}\n", issue.location.bright_white()));
                    formatted.push_str(&format!("   {}\n", issue.explanation));
                    formatted.push_str(&format!(
                        "   Recommendation: {}\n\n",
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
use crate::instruction_presets::PresetType;
use crate::messages;
use crate::ui;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Handles the review command which generates an AI code review of staged changes
/// with comprehensive analysis across multiple dimensions of code quality
pub async fn handle_review_command(common: CommonParams, _print: bool) -> Result<()> {
    // Check if the preset is appropriate for code reviews
    if !common.is_valid_preset_for_type(PresetType::Review) {
        ui::print_warning(
            "The specified preset may not be suitable for code reviews. Consider using a review or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for reviews.");
    }

    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;
    let current_dir = std::env::current_dir()?;

    let provider_name = &config.default_provider;

    let service = Arc::new(
        IrisCommitService::new(
            config.clone(),
            &current_dir.clone(),
            provider_name,
            false, // gitmoji not needed for review
            false, // verification not needed for review
        )
        .context("Failed to create IrisCommitService")?,
    );

    // Check environment prerequisites
    if let Err(e) = service.check_environment() {
        ui::print_error(&format!("Error: {e}"));
        ui::print_info("\nPlease ensure the following:");
        ui::print_info("1. Git is installed and accessible from the command line.");
        ui::print_info("2. You are running this command from within a Git repository.");
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
    spinner.set_message(format!("{}", random_message.text));

    // Generate the code review
    let review = service
        .generate_review(preset_str, &effective_instructions)
        .await?;

    // Stop the spinner
    spinner.finish_and_clear();

    // Print information about the enhanced review
    ui::print_info("\n✨ Enhanced Code Review ✨");
    ui::print_info("This review analyzes your code across multiple dimensions:");

    // Show all dimensions using the enum
    for dimension in QualityDimension::all() {
        ui::print_info(&format!(" • {}", dimension.display_name()));
    }
    println!();

    // Print the review to stdout or save to file if requested
    println!("{}", review.format());

    Ok(())
}
