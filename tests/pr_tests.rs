use anyhow::Result;
use git_iris::config::Config;
use git_iris::context::ChangeType;
use git_iris::git::GitRepo;
use git_iris::types::{GeneratedPullRequest, format_pull_request};
use std::sync::Arc;
use tempfile::TempDir;

// Use our centralized test infrastructure
#[path = "test_utils.rs"]
mod test_utils;
use test_utils::{MockDataBuilder, setup_git_repo_with_commits};

fn create_mock_generated_pr() -> GeneratedPullRequest {
    MockDataBuilder::generated_pull_request()
}

fn setup_test_repo_with_commits_arc() -> Result<(TempDir, Arc<GitRepo>)> {
    let (temp_dir, git_repo) = setup_git_repo_with_commits()?;
    Ok((temp_dir, Arc::new(git_repo)))
}

// Tests for PR type formatting
#[test]
fn test_format_pull_request() {
    let pr = create_mock_generated_pr();
    let formatted = format_pull_request(&pr);

    assert!(formatted.contains("# Add JWT authentication with user registration"));
    assert!(formatted.contains("## Summary"));
    assert!(formatted.contains("## Description"));
    assert!(formatted.contains("## Commits"));
    assert!(formatted.contains("- abc1234: Add JWT authentication middleware"));
    assert!(formatted.contains("- def5678: Implement user registration endpoint"));
    assert!(formatted.contains("## Breaking Changes"));
    assert!(formatted.contains("- All protected endpoints now require authentication headers"));
    assert!(formatted.contains("## Testing"));
    assert!(formatted.contains("Test user registration flow"));
    assert!(formatted.contains("## Notes"));
    assert!(formatted.contains("Requires JWT_SECRET environment variable"));
}

#[test]
fn test_format_pull_request_minimal() {
    let pr = GeneratedPullRequest {
        emoji: None,
        title: "Fix bug in user authentication".to_string(),
        summary: "Fixes a critical bug in the authentication flow".to_string(),
        description: "This PR fixes an issue where users couldn't log in properly.".to_string(),
        commits: vec!["abc1234: Fix authentication bug".to_string()],
        breaking_changes: vec![],
        testing_notes: None,
        notes: None,
    };

    let formatted = format_pull_request(&pr);

    assert!(formatted.contains("# Fix bug in user authentication"));
    assert!(formatted.contains("## Summary"));
    assert!(formatted.contains("## Description"));
    assert!(formatted.contains("## Commits"));
    assert!(formatted.contains("- abc1234: Fix authentication bug"));
    // Should not contain empty sections
    assert!(!formatted.contains("## Breaking Changes"));
    assert!(!formatted.contains("## Testing"));
    assert!(!formatted.contains("## Notes"));
}

// Tests for Git operations (using public API)
#[tokio::test]
async fn test_git_repo_get_commits_for_pr() -> Result<()> {
    let (temp_dir, git_repo) = setup_test_repo_with_commits_arc()?;
    let repo = git2::Repository::open(temp_dir.path())?;

    // Get commits between the initial commit and HEAD
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let commits: Vec<_> = revwalk.collect::<std::result::Result<Vec<_>, _>>()?;

    if commits.len() >= 2 {
        let from_commit = repo.find_commit(commits[1])?; // Second commit (older)
        let to_commit = repo.find_commit(commits[0])?; // First commit (newer)

        let commit_messages = git_repo
            .get_commits_for_pr(&from_commit.id().to_string(), &to_commit.id().to_string())?;

        assert!(!commit_messages.is_empty());
        assert!(commit_messages[0].contains("Add main function"));
    }

    Ok(())
}

#[tokio::test]
async fn test_git_repo_get_commit_range_files() -> Result<()> {
    let (temp_dir, git_repo) = setup_test_repo_with_commits_arc()?;
    let repo = git2::Repository::open(temp_dir.path())?;

    // Get commits
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let commits: Vec<_> = revwalk.collect::<std::result::Result<Vec<_>, _>>()?;

    if commits.len() >= 2 {
        let from_commit = repo.find_commit(commits[1])?; // Second commit (older)
        let to_commit = repo.find_commit(commits[0])?; // First commit (newer)

        let files = git_repo
            .get_commit_range_files(&from_commit.id().to_string(), &to_commit.id().to_string())?;

        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.path == "src/main.rs"));
        assert!(
            files
                .iter()
                .any(|f| matches!(f.change_type, ChangeType::Added))
        );
    }

    Ok(())
}

// Integration tests for GitRepo PR methods
#[tokio::test]
async fn test_git_repo_pr_methods() -> Result<()> {
    let (temp_dir, git_repo) = setup_test_repo_with_commits_arc()?;
    let repo = git2::Repository::open(temp_dir.path())?;

    // Get commit IDs for testing
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let commits: Vec<_> = revwalk.collect::<std::result::Result<Vec<_>, _>>()?;

    if commits.len() >= 2 {
        let from_commit = repo.find_commit(commits[1])?;
        let to_commit = repo.find_commit(commits[0])?;
        let from_id = from_commit.id().to_string();
        let to_id = to_commit.id().to_string();

        // Test get_commits_for_pr
        let commit_messages = git_repo.get_commits_for_pr(&from_id, &to_id)?;
        assert!(!commit_messages.is_empty());

        // Test get_commit_range_files
        let files = git_repo.get_commit_range_files(&from_id, &to_id)?;
        assert!(!files.is_empty());

        // Test get_git_info_for_commit_range
        let context =
            git_repo.get_git_info_for_commit_range(&Config::default(), &from_id, &to_id)?;
        assert!(context.branch.contains(".."));
        assert!(!context.staged_files.is_empty());
    }

    Ok(())
}

#[test]
fn test_format_pull_request_with_unicode() {
    let pr = GeneratedPullRequest {
        emoji: None,
        title: "Add 🚀 deployment automation".to_string(),
        summary: "Implements automated deployment with emojis 🎉".to_string(),
        description: "This PR adds deployment automation:\n\n• Feature 1\n• Feature 2 ✅"
            .to_string(),
        commits: vec!["abc1234: Add 🔧 configuration".to_string()],
        breaking_changes: vec!["⚠️ Configuration format changed".to_string()],
        testing_notes: Some("Test with 🧪 test suite".to_string()),
        notes: Some("Deployment requires 🔑 secrets".to_string()),
    };

    let formatted = format_pull_request(&pr);

    assert!(formatted.contains("🚀 deployment automation"));
    assert!(formatted.contains("🎉"));
    assert!(formatted.contains("✅"));
    assert!(formatted.contains("🔧"));
    assert!(formatted.contains("⚠️"));
    assert!(formatted.contains("🧪"));
    assert!(formatted.contains("🔑"));
}

#[cfg(test)]
mod commitish_tests {
    /// Test helper to check if a reference looks like a commit hash
    fn is_likely_commit_hash(reference: &str) -> bool {
        reference.len() >= 7 && reference.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Test helper to check if a reference uses Git commitish syntax
    fn is_commitish_syntax(reference: &str) -> bool {
        reference.contains('~') || reference.contains('^') || reference.starts_with('@')
    }

    /// Test helper to check if a reference looks like a commit hash or commitish
    fn is_likely_commit_hash_or_commitish(reference: &str) -> bool {
        if reference.len() >= 7 && reference.chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }
        is_commitish_syntax(reference)
    }

    #[test]
    fn test_commit_hash_detection() {
        assert!(is_likely_commit_hash("abcdef1234567"));
        assert!(is_likely_commit_hash("1234567"));
        assert!(!is_likely_commit_hash("abc123")); // Too short
        assert!(!is_likely_commit_hash("abcdefg1234567")); // Contains non-hex
        assert!(!is_likely_commit_hash("HEAD~2"));
        assert!(!is_likely_commit_hash("main"));
    }

    #[test]
    fn test_commitish_syntax_detection() {
        // Test tilde syntax
        assert!(is_commitish_syntax("HEAD~2"));
        assert!(is_commitish_syntax("main~1"));
        assert!(is_commitish_syntax("origin/main~3"));

        // Test caret syntax
        assert!(is_commitish_syntax("HEAD^"));
        assert!(is_commitish_syntax("HEAD^^"));
        assert!(is_commitish_syntax("main^2"));

        // Test @ syntax
        assert!(is_commitish_syntax("@"));
        assert!(is_commitish_syntax("@~3"));
        assert!(is_commitish_syntax("@^"));

        // Test combinations
        assert!(is_commitish_syntax("HEAD~2^"));

        // Test non-commitish
        assert!(!is_commitish_syntax("main"));
        assert!(!is_commitish_syntax("feature-branch"));
        assert!(!is_commitish_syntax("abcdef1234567"));
    }

    #[test]
    fn test_combined_detection() {
        // Commit hashes should be detected
        assert!(is_likely_commit_hash_or_commitish("abcdef1234567"));
        assert!(is_likely_commit_hash_or_commitish("1234567"));

        // Commitish syntax should be detected
        assert!(is_likely_commit_hash_or_commitish("HEAD~2"));
        assert!(is_likely_commit_hash_or_commitish("HEAD^"));
        assert!(is_likely_commit_hash_or_commitish("@~3"));
        assert!(is_likely_commit_hash_or_commitish("main~1"));

        // Regular branches should not be detected
        assert!(!is_likely_commit_hash_or_commitish("main"));
        assert!(!is_likely_commit_hash_or_commitish("feature-branch"));
        assert!(!is_likely_commit_hash_or_commitish("origin/main"));
    }
}
