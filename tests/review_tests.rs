use git_iris::types::{CodeIssue, DimensionAnalysis, GeneratedReview};

// Use our centralized test infrastructure
#[path = "test_utils.rs"]
mod test_utils;

#[test]
fn test_review_format() {
    // Test that the review formatting works as expected
    let review = GeneratedReview {
        summary: "Test summary".to_string(),
        code_quality: "Good quality".to_string(),
        suggestions: vec!["Suggestion 1".to_string(), "Suggestion 2".to_string()],
        issues: vec!["Issue 1".to_string()],
        positive_aspects: vec!["Positive 1".to_string(), "Positive 2".to_string()],
        complexity: Some(DimensionAnalysis {
            issues_found: true,
            issues: vec![CodeIssue {
                description: "Complex function".to_string(),
                severity: "Medium".to_string(),
                location: "src/main.rs:42".to_string(),
                explanation: "This function has too many nested conditionals".to_string(),
                recommendation: "Extract nested logic into separate functions".to_string(),
            }],
        }),
        abstraction: None,
        deletion: None,
        hallucination: None,
        style: None,
        security: None,
        performance: None,
        duplication: None,
        error_handling: None,
        testing: None,
        best_practices: Some(DimensionAnalysis {
            issues_found: true,
            issues: vec![CodeIssue {
                description: "SOLID principle violation".to_string(),
                severity: "High".to_string(),
                location: "src/user_service.rs:105-120".to_string(),
                explanation: "This class violates the Single Responsibility Principle by handling both authentication and user data management".to_string(),
                recommendation: "Split into separate UserService and AuthenticationService classes".to_string(),
            }],
        }),
    };

    let formatted = review.format();

    // Check that the formatted output contains all the important parts
    // We can't match exact strings because of color codes, so we'll check for key substrings
    assert!(formatted.contains("CODE REVIEW"));
    assert!(formatted.contains("Test summary"));
    assert!(formatted.contains("QUALITY")); // SilkCircuit styling uses "QUALITY" not "QUALITY ASSESSMENT"
    assert!(formatted.contains("Good quality"));
    assert!(formatted.contains("STRENGTHS"));
    assert!(formatted.contains("Positive 1"));
    assert!(formatted.contains("Positive 2"));
    assert!(formatted.contains("ISSUES")); // SilkCircuit styling uses "ISSUES" not "CORE ISSUES"
    assert!(formatted.contains("Issue 1"));
    assert!(formatted.contains("SUGGESTIONS"));
    assert!(formatted.contains("Suggestion 1"));
    assert!(formatted.contains("Suggestion 2"));

    // Check the complexity dimension was formatted (SilkCircuit uses uppercase headers)
    assert!(formatted.contains("COMPLEXITY"));
    assert!(formatted.contains("Complex function"));
    assert!(formatted.contains("MEDIUM"));
    assert!(formatted.contains("src/main.rs:42"));
    assert!(formatted.contains("This function has too many nested conditionals"));
    assert!(formatted.contains("Extract nested logic into separate functions"));

    // Check the best practices dimension was formatted (SilkCircuit uses uppercase headers)
    assert!(formatted.contains("BEST PRACTICES"));
    assert!(formatted.contains("SOLID principle violation"));
    assert!(formatted.contains("HIGH"));
    assert!(formatted.contains("src/user_service.rs:105-120"));
    assert!(formatted.contains("This class violates the Single Responsibility Principle"));
    assert!(
        formatted.contains("Split into separate UserService and AuthenticationService classes")
    );
}

#[test]
fn test_format_location() {
    // Test with a path containing filename and line numbers
    let location = "src/commit/review.rs:45-67";
    let formatted = GeneratedReview::format_location(location);
    assert_eq!(formatted, location);

    // Test with just line numbers
    let location = "45-67";
    let formatted = GeneratedReview::format_location(location);
    assert_eq!(formatted, "Line(s) 45-67");

    // Test with line keyword
    let location = "Line 45";
    let formatted = GeneratedReview::format_location(location);
    assert_eq!(formatted, location);

    // Test with file keyword
    let location = "in file helpers.js";
    let formatted = GeneratedReview::format_location(location);
    assert_eq!(formatted, location);

    // Test with just file extension
    let location = "helpers.js";
    let formatted = GeneratedReview::format_location(location);
    assert_eq!(formatted, location);
}

// Helper function to validate branch comparison parameters
fn validate_branch_parameters(
    from: Option<&String>,
    to: Option<&String>,
    commit_id: Option<&String>,
    include_unstaged: bool,
) -> Result<(), String> {
    // Validate branch parameters
    if from.is_some() && to.is_none() {
        return Err(
            "When using --from, you must also specify --to for branch comparison reviews"
                .to_string(),
        );
    }

    if commit_id.is_some() && (from.is_some() || to.is_some()) {
        return Err(
            "Cannot use --commit with --from/--to. These are mutually exclusive options"
                .to_string(),
        );
    }

    if include_unstaged && (from.is_some() || to.is_some()) {
        return Err("Cannot use --include-unstaged with --from/--to. Branch comparison reviews don't include working directory changes".to_string());
    }

    Ok(())
}

#[test]
fn test_branch_parameter_validation() {
    // Test valid combinations
    assert!(validate_branch_parameters(None, None, None, false).is_ok());
    assert!(
        validate_branch_parameters(
            Some(&"main".to_string()),
            Some(&"feature".to_string()),
            None,
            false
        )
        .is_ok()
    );
    assert!(validate_branch_parameters(None, Some(&"feature".to_string()), None, false).is_ok()); // --to only (defaults to main)
    assert!(validate_branch_parameters(None, None, Some(&"abc123".to_string()), false).is_ok());
    assert!(validate_branch_parameters(None, None, None, true).is_ok());

    // Test invalid combinations
    assert!(validate_branch_parameters(Some(&"main".to_string()), None, None, false).is_err()); // --from without --to
    assert!(
        validate_branch_parameters(
            Some(&"main".to_string()),
            Some(&"feature".to_string()),
            Some(&"abc123".to_string()),
            false
        )
        .is_err()
    ); // branches + commit
    assert!(
        validate_branch_parameters(
            None,
            Some(&"feature".to_string()),
            Some(&"abc123".to_string()),
            false
        )
        .is_err()
    ); // --to + commit
    assert!(
        validate_branch_parameters(
            Some(&"main".to_string()),
            Some(&"feature".to_string()),
            None,
            true
        )
        .is_err()
    ); // branches + unstaged
    assert!(validate_branch_parameters(None, Some(&"feature".to_string()), None, true).is_err()); // --to + unstaged

    // Test error messages
    let result = validate_branch_parameters(Some(&"main".to_string()), None, None, false);
    assert!(result.is_err());
    assert!(
        result
            .expect_err("Should have error")
            .contains("When using --from, you must also specify --to")
    );

    let result = validate_branch_parameters(
        Some(&"main".to_string()),
        Some(&"feature".to_string()),
        Some(&"abc123".to_string()),
        false,
    );
    assert!(result.is_err());
    assert!(
        result
            .expect_err("Should have error")
            .contains("Cannot use --commit with --from/--to")
    );

    let result = validate_branch_parameters(
        Some(&"main".to_string()),
        Some(&"feature".to_string()),
        None,
        true,
    );
    assert!(result.is_err());
    assert!(
        result
            .expect_err("Should have error")
            .contains("Cannot use --include-unstaged with --from/--to")
    );
}
