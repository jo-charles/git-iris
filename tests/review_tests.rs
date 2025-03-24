use git_iris::context::GeneratedReview;

#[test]
fn test_review_format() {
    // Test that the review formatting works as expected
    let review = GeneratedReview {
        summary: "Test summary".to_string(),
        code_quality: "Good quality".to_string(),
        suggestions: vec!["Suggestion 1".to_string(), "Suggestion 2".to_string()],
        issues: vec!["Issue 1".to_string()],
        positive_aspects: vec!["Positive 1".to_string(), "Positive 2".to_string()],
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
