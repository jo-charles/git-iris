use anyhow::Result;
use git_iris::commit::IrisCommitService;
use git_iris::config::Config;
use git_iris::git::GitRepo;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn setup_test_repo() -> Result<(TempDir, Arc<GitRepo>)> {
    let temp_dir = TempDir::new()?;
    let repo = git2::Repository::init(temp_dir.path())?;

    // Configure git user
    let mut config = repo.config().expect("Failed to get repository config");
    config.set_str("user.name", "Git-Iris")?;
    config.set_str("user.email", "git-iris@is-aweso.me")?;

    // Create an initial commit
    let signature = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )?;

    let git_repo = Arc::new(GitRepo::new(temp_dir.path())?);

    Ok((temp_dir, git_repo))
}

#[tokio::test]
async fn test_perform_commit() -> Result<()> {
    let (temp_dir, _git_repo) = setup_test_repo()?;
    let config = Config::default();
    let repo_path = PathBuf::from(temp_dir.path());
    let provider_name = "test";
    let use_gitmoji = true;
    let verify = true;

    // Create a new GitRepo for the service
    let service_repo = GitRepo::new(temp_dir.path())?;

    let service = IrisCommitService::new(
        config,
        &repo_path,
        provider_name,
        use_gitmoji,
        verify,
        service_repo,
    )?;

    let result = service.perform_commit("Test commit message")?;
    println!("Perform commit result: {result:?}");

    // Verify the commit was made
    let repo = git2::Repository::open(&repo_path)?;
    let head_commit = repo.head()?.peel_to_commit()?;
    assert_eq!(
        head_commit.message().expect("Failed to get commit message"),
        "Test commit message"
    );

    Ok(())
}
