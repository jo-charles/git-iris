//! Tests for review functionality
//!
//! Note: Legacy GeneratedReview tests removed. MarkdownReview is now the active code path.

// Use our centralized test infrastructure
#[path = "test_utils.rs"]
mod test_utils;

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
