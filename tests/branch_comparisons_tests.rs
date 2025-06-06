use git_iris::config::Config;
use git_iris::context::ChangeType;
use git_iris::git::GitRepo;
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
async fn test_branch_comparison_basic() {
    let (temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    // Get the repository to work with branches
    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");

    // Create a feature branch
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
    let feature_branch = repo
        .branch("feature-branch", &head_commit, false)
        .expect("Failed to create feature branch");

    // Switch to feature branch
    repo.set_head(feature_branch.get().name().unwrap())
        .expect("Failed to set head to feature branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("Failed to checkout feature branch");

    // Add a new file in feature branch
    let feature_file_path = temp_dir.path().join("feature.txt");
    fs::write(&feature_file_path, "Feature content").expect("Failed to write feature file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("feature.txt"))
        .expect("Failed to add feature file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add feature file",
        &tree,
        &[&head_commit],
    )
    .expect("Failed to commit feature");

    // Test branch comparison
    let context = git_repo
        .get_git_info_for_branch_diff(&config, "main", "feature-branch")
        .await
        .expect("Failed to get git info for branch diff");

    // Test branch name formatting
    assert_eq!(context.branch, "main -> feature-branch");

    // Test that we have the feature commit
    assert_eq!(context.recent_commits.len(), 1);
    assert!(
        context.recent_commits[0]
            .message
            .contains("Add feature file")
    );

    // Test staged files (should contain the feature file)
    assert_eq!(context.staged_files.len(), 1);
    assert_eq!(context.staged_files[0].path, "feature.txt");
    assert!(matches!(
        context.staged_files[0].change_type,
        ChangeType::Added
    ));
    assert!(context.staged_files[0].content.is_some());
    assert_eq!(
        context.staged_files[0].content.as_ref().unwrap(),
        "Feature content"
    );
}

#[tokio::test]
async fn test_branch_comparison_multiple_files() {
    let (temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();

    // Create feature branch
    let feature_branch = repo
        .branch("feature-multiple", &head_commit, false)
        .expect("Failed to create feature branch");
    repo.set_head(feature_branch.get().name().unwrap())
        .expect("Failed to set head to feature branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("Failed to checkout feature branch");

    // Add multiple files
    for i in 1..=3 {
        let file_path = temp_dir.path().join(format!("feature{i}.txt"));
        fs::write(&file_path, format!("Feature {i} content"))
            .expect("Failed to write feature file");
    }

    // Modify the initial file
    let initial_file_path = temp_dir.path().join("initial.txt");
    fs::write(&initial_file_path, "Modified in feature branch")
        .expect("Failed to modify initial file");

    // Stage all changes
    let mut index = repo.index().expect("Failed to get repository index");
    for i in 1..=3 {
        index
            .add_path(Path::new(&format!("feature{i}.txt")))
            .expect("Failed to add feature file to index");
    }
    index
        .add_path(Path::new("initial.txt"))
        .expect("Failed to add modified file to index");
    index.write().expect("Failed to write index");

    // Commit changes
    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add multiple features and modify initial",
        &tree,
        &[&head_commit],
    )
    .expect("Failed to commit multiple features");

    // Test branch comparison
    let context = git_repo
        .get_git_info_for_branch_diff(&config, "main", "feature-multiple")
        .await
        .expect("Failed to get git info for branch diff");

    // Should have 4 files: 3 new + 1 modified
    assert_eq!(context.staged_files.len(), 4);

    // Count file types
    let added_files: Vec<_> = context
        .staged_files
        .iter()
        .filter(|f| matches!(f.change_type, ChangeType::Added))
        .collect();
    let modified_files: Vec<_> = context
        .staged_files
        .iter()
        .filter(|f| matches!(f.change_type, ChangeType::Modified))
        .collect();

    assert_eq!(added_files.len(), 3);
    assert_eq!(modified_files.len(), 1);

    // Verify the modified file
    let modified_file = modified_files[0];
    assert_eq!(modified_file.path, "initial.txt");
    assert_eq!(
        modified_file.content.as_ref().unwrap(),
        "Modified in feature branch"
    );

    // Verify added files
    for i in 1..=3 {
        assert!(
            added_files
                .iter()
                .any(|f| f.path == format!("feature{i}.txt"))
        );
    }
}

#[tokio::test]
async fn test_branch_comparison_with_deletions() {
    let (temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();

    // Add a file to main branch first
    let file_to_delete_path = temp_dir.path().join("will_be_deleted.txt");
    fs::write(&file_to_delete_path, "This will be deleted").expect("Failed to write file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("will_be_deleted.txt"))
        .expect("Failed to add file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    let main_commit = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add file to be deleted",
            &tree,
            &[&head_commit],
        )
        .expect("Failed to commit file to delete");

    // Create feature branch from this new commit
    let main_commit_obj = repo
        .find_commit(main_commit)
        .expect("Failed to find main commit");
    let feature_branch = repo
        .branch("feature-delete", &main_commit_obj, false)
        .expect("Failed to create feature branch");
    repo.set_head(feature_branch.get().name().unwrap())
        .expect("Failed to set head to feature branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("Failed to checkout feature branch");

    // Delete the file
    fs::remove_file(&file_to_delete_path).expect("Failed to remove file");
    let mut index = repo.index().expect("Failed to get repository index");
    index
        .remove_path(Path::new("will_be_deleted.txt"))
        .expect("Failed to remove file from index");
    index.write().expect("Failed to write index");

    // Commit deletion
    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Delete file",
        &tree,
        &[&main_commit_obj],
    )
    .expect("Failed to commit deletion");

    // Test branch comparison
    let context = git_repo
        .get_git_info_for_branch_diff(&config, "main", "feature-delete")
        .await
        .expect("Failed to get git info for branch diff");

    // Should have 1 deleted file
    assert_eq!(context.staged_files.len(), 1);
    assert_eq!(context.staged_files[0].path, "will_be_deleted.txt");
    assert!(matches!(
        context.staged_files[0].change_type,
        ChangeType::Deleted
    ));

    // Deleted files shouldn't have content
    assert!(context.staged_files[0].content.is_none());
}

#[tokio::test]
async fn test_branch_comparison_default_from_main() {
    let (temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();

    // Create feature branch
    let feature_branch = repo
        .branch("feature-default", &head_commit, false)
        .expect("Failed to create feature branch");
    repo.set_head(feature_branch.get().name().unwrap())
        .expect("Failed to set head to feature branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("Failed to checkout feature branch");

    // Add a file
    let feature_file_path = temp_dir.path().join("default_test.txt");
    fs::write(&feature_file_path, "Default from main test").expect("Failed to write feature file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("default_test.txt"))
        .expect("Failed to add feature file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add file for default test",
        &tree,
        &[&head_commit],
    )
    .expect("Failed to commit feature");

    // Test branch comparison using main as default base
    let context = git_repo
        .get_git_info_for_branch_diff(&config, "main", "feature-default")
        .await
        .expect("Failed to get git info for branch diff");

    // Verify we get the expected results
    assert_eq!(context.branch, "main -> feature-default");
    assert_eq!(context.staged_files.len(), 1);
    assert_eq!(context.staged_files[0].path, "default_test.txt");
    assert!(matches!(
        context.staged_files[0].change_type,
        ChangeType::Added
    ));
}

#[tokio::test]
async fn test_branch_comparison_with_binary_files() {
    let (temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();

    // Create feature branch
    let feature_branch = repo
        .branch("feature-binary", &head_commit, false)
        .expect("Failed to create feature branch");
    repo.set_head(feature_branch.get().name().unwrap())
        .expect("Failed to set head to feature branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("Failed to checkout feature branch");

    // Add a binary file
    let binary_file_path = temp_dir.path().join("binary.png");
    let binary_content = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(&binary_file_path, binary_content).expect("Failed to write binary file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("binary.png"))
        .expect("Failed to add binary file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add binary file",
        &tree,
        &[&head_commit],
    )
    .expect("Failed to commit binary file");

    // Test branch comparison
    let context = git_repo
        .get_git_info_for_branch_diff(&config, "main", "feature-binary")
        .await
        .expect("Failed to get git info for branch diff");

    // Should have 1 binary file
    assert_eq!(context.staged_files.len(), 1);
    assert_eq!(context.staged_files[0].path, "binary.png");
    assert!(matches!(
        context.staged_files[0].change_type,
        ChangeType::Added
    ));

    // Binary files should be detected and marked appropriately
    assert_eq!(context.staged_files[0].diff, "[Binary file changed]");
    assert!(context.staged_files[0].content.is_none());
}

#[tokio::test]
async fn test_branch_comparison_nonexistent_branches() {
    let (_temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    // Test with nonexistent base branch
    let result = git_repo
        .get_git_info_for_branch_diff(&config, "nonexistent-base", "main")
        .await;
    assert!(result.is_err(), "Should fail with nonexistent base branch");

    // Test with nonexistent target branch
    let result = git_repo
        .get_git_info_for_branch_diff(&config, "main", "nonexistent-target")
        .await;
    assert!(
        result.is_err(),
        "Should fail with nonexistent target branch"
    );

    // Test with both branches nonexistent
    let result = git_repo
        .get_git_info_for_branch_diff(&config, "nonexistent-base", "nonexistent-target")
        .await;
    assert!(
        result.is_err(),
        "Should fail with both branches nonexistent"
    );
}

#[tokio::test]
async fn test_branch_comparison_with_merge_base() {
    let (temp_dir, git_repo) = setup_git_repo();
    let config = Config::default();

    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");
    let initial_commit = repo.head().unwrap().peel_to_commit().unwrap();

    // Create feature branch from initial commit
    let feature_branch = repo
        .branch("feature-mergeback", &initial_commit, false)
        .expect("Failed to create feature branch");

    // Add commits to main branch (simulating main moving forward)
    let main_file_path = temp_dir.path().join("main_progress.txt");
    fs::write(&main_file_path, "Progress in main").expect("Failed to write main file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("main_progress.txt"))
        .expect("Failed to add main file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let signature = repo.signature().expect("Failed to create signature");
    let main_new_commit = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Progress in main branch",
            &tree,
            &[&initial_commit],
        )
        .expect("Failed to commit to main");

    // Add another commit to main
    let main_file2_path = temp_dir.path().join("main_progress2.txt");
    fs::write(&main_file2_path, "More progress in main").expect("Failed to write main file 2");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("main_progress2.txt"))
        .expect("Failed to add main file 2 to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let main_commit_obj = repo
        .find_commit(main_new_commit)
        .expect("Failed to find main commit");
    let _main_final_commit = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "More progress in main branch",
            &tree,
            &[&main_commit_obj],
        )
        .expect("Failed to commit to main again");

    // Now switch to feature branch and add commits there
    repo.set_head(feature_branch.get().name().unwrap())
        .expect("Failed to set head to feature branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .expect("Failed to checkout feature branch");

    // Add feature files
    let feature_file_path = temp_dir.path().join("feature_work.txt");
    fs::write(&feature_file_path, "Feature work").expect("Failed to write feature file");

    let mut index = repo.index().expect("Failed to get repository index");
    index
        .add_path(Path::new("feature_work.txt"))
        .expect("Failed to add feature file to index");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add feature work",
        &tree,
        &[&initial_commit],
    )
    .expect("Failed to commit feature");

    // Test branch comparison - should only show feature changes, not main changes
    let context = git_repo
        .get_git_info_for_branch_diff(&config, "main", "feature-mergeback")
        .await
        .expect("Failed to get git info for branch diff");

    // Should only have 1 file from the feature branch, not the 2 files from main
    assert_eq!(context.staged_files.len(), 1);
    assert_eq!(context.staged_files[0].path, "feature_work.txt");
    assert!(matches!(
        context.staged_files[0].change_type,
        ChangeType::Added
    ));

    // Should not include main branch files
    assert!(
        !context
            .staged_files
            .iter()
            .any(|f| f.path == "main_progress.txt")
    );
    assert!(
        !context
            .staged_files
            .iter()
            .any(|f| f.path == "main_progress2.txt")
    );

    // Should only show commits from the feature branch
    assert_eq!(context.recent_commits.len(), 1);
    assert!(
        context.recent_commits[0]
            .message
            .contains("Add feature work")
    );

    // Should not include main branch commits
    assert!(
        !context
            .recent_commits
            .iter()
            .any(|c| c.message.contains("Progress in main"))
    );
}
