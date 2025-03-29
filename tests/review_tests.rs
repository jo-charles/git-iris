use git_iris::context::{CodeIssue, DimensionAnalysis, GeneratedReview};

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
    };

    let formatted = review.format();

    // Check that the formatted output contains all the important parts
    // We can't match exact strings because of color codes, so we'll check for key substrings
    assert!(formatted.contains("Code Review Summary"));
    assert!(formatted.contains("Test summary"));
    assert!(formatted.contains("Code Quality Assessment"));
    assert!(formatted.contains("Good quality"));
    assert!(formatted.contains("Positive Aspects"));
    assert!(formatted.contains("Positive 1"));
    assert!(formatted.contains("Positive 2"));
    assert!(formatted.contains("Issues Identified"));
    assert!(formatted.contains("Issue 1"));
    assert!(formatted.contains("Suggestions for Improvement"));
    assert!(formatted.contains("Suggestion 1"));
    assert!(formatted.contains("Suggestion 2"));

    // Check the complexity dimension was formatted
    assert!(formatted.contains("Complexity"));
    assert!(formatted.contains("Complex function"));
    assert!(formatted.contains("Medium"));
    assert!(formatted.contains("src/main.rs:42"));
    assert!(formatted.contains("This function has too many nested conditionals"));
    assert!(formatted.contains("Extract nested logic into separate functions"));
}

// Note: We don't include a test for handle_review_command here because:
// 1. It would require mocking a lot of complex dependencies
// 2. The core functionality (generate_review) is already tested in service_tests.rs
// 3. The command itself primarily handles user interface concerns
//
// For thorough testing, we would need to:
// - Mock the IrisCommitService
// - Mock the git info generation
// - Mock the LLM response
// - Capture stdout to verify output
//
// These would be better suited for integration tests with a more sophisticated test harness.
