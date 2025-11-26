use anyhow::Result;
use git_iris::git::GitRepo;
use git_iris::services::GitCommitService;
use std::sync::Arc;
use tempfile::TempDir;

// Use our centralized test infrastructure
#[path = "test_utils.rs"]
mod test_utils;
use test_utils::setup_git_repo_with_commits;

fn setup_test_repo() -> Result<(TempDir, Arc<GitRepo>)> {
    let (temp_dir, git_repo) = setup_git_repo_with_commits()?;
    Ok((temp_dir, Arc::new(git_repo)))
}

#[tokio::test]
async fn test_perform_commit() -> Result<()> {
    let (temp_dir, _git_repo) = setup_test_repo()?;
    let use_gitmoji = true;
    let verify = true;

    // Create a new GitRepo for the service
    let service_repo = Arc::new(GitRepo::new(temp_dir.path())?);

    let service = GitCommitService::new(service_repo, use_gitmoji, verify);

    let result = service.perform_commit("Test commit message")?;
    println!("Perform commit result: {result:?}");

    // Verify the commit was made
    let repo = git2::Repository::open(temp_dir.path())?;
    let head_commit = repo.head()?.peel_to_commit()?;
    assert_eq!(
        head_commit.message().expect("Failed to get commit message"),
        "Test commit message"
    );

    Ok(())
}
