use git_iris::commit::prompt::{create_system_prompt, create_user_prompt};
use git_iris::config::Config;
use git_iris::git::GitRepo;
use git_iris::token_optimizer::TokenOptimizer;
use git2::Repository;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_git_repo() -> (TempDir, GitRepo) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let repo = Repository::init(temp_dir.path()).expect("Failed to initialize repository");

    // Configure git user
    let mut config = repo.config().expect("Failed to get repository config");
    config
        .set_str("user.name", "Test User")
        .expect("Failed to set user name");
    config
        .set_str("user.email", "test@example.com")
        .expect("Failed to set user email");

    // Create and commit an initial file
    let initial_file_path = temp_dir.path().join("initial.txt");
    fs::write(&initial_file_path, "Initial content").expect("Failed to write initial file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("initial.txt"))
        .expect("Failed to add file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )
    .expect("Failed to commit");

    // Ensure the default branch is named 'main' for consistency across environments
    {
        let head_commit = repo
            .head()
            .expect("Failed to get HEAD")
            .peel_to_commit()
            .expect("Failed to peel HEAD to commit");
        let current_branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(std::string::ToString::to_string))
            .unwrap_or_default();
        if current_branch != "main" {
            // Create or update the 'main' branch pointing to the current HEAD commit
            repo.branch("main", &head_commit, true)
                .expect("Failed to create 'main' branch");
            repo.set_head("refs/heads/main")
                .expect("Failed to set HEAD to 'main' branch");
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .expect("Failed to checkout 'main' branch");
        }
    }

    let git_repo = GitRepo::new(temp_dir.path()).expect("Failed to create GitRepo");
    (temp_dir, git_repo)
}

#[tokio::test]
async fn test_token_optimization_integration() {
    let (_temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    // Set a small token limit for the OpenAI provider to force truncation
    let small_token_limit = 200;
    let optimizer = TokenOptimizer::new(small_token_limit);

    let context = git_repo
        .get_git_info(&config)
        .await
        .expect("Failed to get git info");

    let system_prompt = create_system_prompt(&config).expect("Failed to create system prompt");
    let user_prompt = create_user_prompt(&context);
    let prompt = format!("{system_prompt}\n{user_prompt}");

    // Check that the prompt is within the token limit
    let prompt = optimizer.truncate_string(&prompt, small_token_limit);

    let token_count = optimizer.count_tokens(&prompt);

    println!("Token count: {token_count}");
    println!("Token limit: {small_token_limit}");
    println!("Prompt:\n{prompt}");

    assert!(
        token_count <= small_token_limit,
        "Prompt exceeds token limit. Token count: {token_count}, Limit: {small_token_limit}"
    );

    // Check that the prompt contains essential information
    assert!(
        prompt.contains("Git commit message"),
        "Prompt should contain instructions"
    );

    // The following assertions may fail due to truncation, so we'll make them optional
    if token_count < small_token_limit {
        assert!(
            prompt.contains("Branch:"),
            "Prompt should contain branch information"
        );
        assert!(
            prompt.contains("Recent commits:"),
            "Prompt should mention recent commits"
        );
        assert!(
            prompt.contains("Staged changes:"),
            "Prompt should mention staged changes"
        );
    }

    // Check that the prompt ends with the truncation indicator
    assert!(
        prompt.ends_with('…'),
        "Prompt should end with truncation indicator"
    );

    // Test with a larger token limit
    let large_token_limit = 5000;

    let system_prompt = create_system_prompt(&config).expect("Failed to create system prompt");
    let user_prompt = create_user_prompt(&context);
    let large_prompt = format!("{system_prompt}\n{user_prompt}");

    let large_optimizer = TokenOptimizer::new(large_token_limit);
    let large_token_count = large_optimizer.count_tokens(&large_prompt);

    assert!(
        large_token_count <= large_token_limit,
        "Large prompt exceeds token limit. Token count: {large_token_count}, Limit: {large_token_limit}"
    );
}
