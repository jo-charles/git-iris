#![allow(clippy::unwrap_used)]

use anyhow::Result;
use git_iris::types::MarkdownChangelog;
use git2::Repository;
use tempfile::TempDir;

// Use our centralized test infrastructure
#[path = "test_utils.rs"]
mod test_utils;
use test_utils::setup_git_repo_with_tags;

/// Sets up a temporary Git repository for testing
#[allow(dead_code)]
fn setup_test_repo() -> Result<(TempDir, Repository)> {
    setup_git_repo_with_tags()
}

#[test]
fn test_markdown_changelog_format() {
    let changelog = MarkdownChangelog {
        content: r#"## [1.0.0] - 2023-06-01

This release adds new features and fixes bugs.

### Added

- Add `new_feature` module for enhanced functionality (abc1234)
- Add support for custom configurations #123

### Changed

- Update `config` module to use TOML format

### Fixed

- Fix memory leak in `cache_handler` (def5678)

### Metrics

- Total Commits: 5
- Files Changed: 12
- Insertions: +245
- Deletions: -87
"#
        .to_string(),
    };

    // Test raw content
    let raw = changelog.raw_content();
    assert!(raw.contains("## [1.0.0] - 2023-06-01"));
    assert!(raw.contains("### Added"));
    assert!(raw.contains("### Changed"));
    assert!(raw.contains("### Fixed"));
    assert!(raw.contains("### Metrics"));
    assert!(raw.contains("`new_feature`"));
    assert!(raw.contains("abc1234"));

    // Test formatted output (terminal rendering)
    let formatted = changelog.format();
    assert!(!formatted.is_empty());
}

/// Test that the `version_name` parameter correctly overrides the changelog version
#[test]
fn test_update_changelog_file_with_version_name() -> Result<()> {
    use git_iris::changelog::ChangelogGenerator;
    use git_iris::git::GitRepo;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Set up a temporary directory for the test
    let temp_dir = TempDir::new()?;

    // Create a mock changelog content
    let changelog_content =
        "## [1.0.0] - 2023-01-01\n\n### Added\n\n- Test feature\n\n### Fixed\n\n- Test bugfix\n";

    // Create a temporary repository
    let (_, repo) = setup_test_repo()?;
    let git_repo = Arc::new(GitRepo::new(repo.path())?);

    // Path for the changelog file
    let changelog_path = temp_dir.path().join("CHANGELOG.md");

    // Update the changelog with custom version name
    let custom_version = "2.0.0-alpha";
    ChangelogGenerator::update_changelog_file(
        changelog_content,
        changelog_path
            .to_str()
            .expect("Invalid path for changelog file"),
        &git_repo,
        "HEAD",
        Some(custom_version.to_string()),
    )?;

    // Read the updated file
    let updated_content = std::fs::read_to_string(&changelog_path)?;

    // Check that the custom version name is used
    assert!(
        updated_content.contains(&format!("## [{custom_version}]")),
        "Changelog should contain the custom version name"
    );

    // Clean up
    temp_dir.close()?;
    Ok(())
}

#[test]
fn test_markdown_release_notes_format() {
    use git_iris::types::MarkdownReleaseNotes;

    let release_notes = MarkdownReleaseNotes {
        content: r#"# Release Notes v1.0.0

**Released:** 2023-06-01

This release includes new features and bug fixes.

## Highlights

### New Feature
A great new feature was added.

## Changes

- Added new capability
- Fixed critical bug

## Breaking Changes

- API endpoint changed
"#
        .to_string(),
    };

    // Test raw content
    assert!(
        release_notes
            .raw_content()
            .contains("# Release Notes v1.0.0")
    );
    assert!(release_notes.raw_content().contains("## Highlights"));
    assert!(release_notes.raw_content().contains("## Breaking Changes"));

    // Test formatted output (terminal rendering)
    let formatted = release_notes.format();
    assert!(!formatted.is_empty());
}
