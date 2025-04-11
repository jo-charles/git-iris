use git_iris::common::CommonParams;
use git_iris::config::{Config, ProviderConfig};
use git_iris::git::GitRepo;

use git2::Repository;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// Helper function to set up a test Git repository
fn setup_git_repo() -> (TempDir, GitRepo) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    // Explicitly initialize git in the directory
    let init_output = Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    assert!(
        init_output.status.success(),
        "Failed to initialize git repository: {init_output:?}"
    );

    // Now use git2 to configure and make an initial commit
    let repo = Repository::open(temp_dir.path()).expect("Failed to open repository");

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

    // Print current directory for debugging
    println!("Current directory: {:?}", env::current_dir().unwrap());
    println!("Temp directory: {:?}", temp_dir.path());

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

    let git_repo = GitRepo::new(temp_dir.path()).expect("Failed to create GitRepo");
    (temp_dir, git_repo)
}

// Helper to verify git repo status
fn is_git_repo(dir: &Path) -> bool {
    let status = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(dir)
        .output()
        .expect("Failed to execute git command");

    status.status.success()
}

#[test]
fn test_project_config_security() {
    // Set up a git repository in a temporary directory
    let (temp_dir, _git_repo) = setup_git_repo();

    // Save current directory so we can restore it later
    let original_dir = env::current_dir().unwrap();

    // Change to the test repository directory for the test
    env::set_current_dir(temp_dir.path()).unwrap();

    // Verify we're in a git repo
    assert!(
        is_git_repo(Path::new(".")),
        "Current directory is not a git repository"
    );

    // 1. Test API key security in project config
    // Create a config with API keys
    let mut config = Config::default();

    // Add API keys to multiple providers
    for provider_name in &["openai", "anthropic", "cohere"] {
        let mut provider_config = ProviderConfig::default();
        provider_config.api_key = format!("secret_{provider_name}_api_key");
        provider_config.model = format!("{provider_name}_model");

        config
            .providers
            .insert((*provider_name).to_string(), provider_config);
    }

    // Save as project config
    config
        .save_as_project_config()
        .expect("Failed to save project config");

    // Verify the file exists
    let config_path = Path::new(".irisconfig");
    assert!(config_path.exists(), "Project config file not created");

    // Read the file content
    let content = fs::read_to_string(config_path).expect("Failed to read project config file");

    // Verify no API keys are in the file
    for provider_name in &["openai", "anthropic", "cohere"] {
        let api_key = format!("secret_{provider_name}_api_key");
        assert!(
            !content.contains(&api_key),
            "API key was found in project config file"
        );
    }

    // 2. Test merging project config with personal config
    // Create a personal config with API keys
    let mut personal_config = Config::default();

    let mut personal_provider_config = ProviderConfig::default();
    personal_provider_config.api_key = "personal_api_key".to_string();
    personal_provider_config.model = "gpt-3.5-turbo".to_string();
    personal_config
        .providers
        .insert("openai".to_string(), personal_provider_config);

    // Create a project config
    let mut project_config = Config::default();
    let mut project_provider_config = ProviderConfig::default();
    project_provider_config.api_key = String::new(); // Empty API key
    project_provider_config.model = "gpt-4".to_string();
    project_config
        .providers
        .insert("openai".to_string(), project_provider_config);

    // Merge configs
    personal_config.merge_with_project_config(project_config);

    // Verify API key from personal config is preserved
    let provider_config = personal_config.providers.get("openai").unwrap();
    assert_eq!(
        provider_config.api_key, "personal_api_key",
        "Personal API key was lost during merge"
    );

    // Verify model from project config is used
    assert_eq!(
        provider_config.model, "gpt-4",
        "Project model setting was not applied"
    );

    // 3. Test CLI command integration
    // Set up common parameters similar to CLI arguments
    let common = CommonParams {
        provider: Some("openai".to_string()),
        instructions: Some("Test instructions".to_string()),
        preset: Some("default".to_string()),
        gitmoji: Some(true),
        detail_level: "standard".to_string(),
        repository_url: None,
    };

    // Create a config and apply common parameters
    let mut config = Config::default();
    common
        .apply_to_config(&mut config)
        .expect("Failed to apply common params");

    // Set an API key
    let provider_config = config.providers.get_mut("openai").unwrap();
    provider_config.api_key = "cli_integration_api_key".to_string();

    // Save as project config
    config
        .save_as_project_config()
        .expect("Failed to save project config with CLI params");

    // Read the file content
    let content = fs::read_to_string(config_path).expect("Failed to read project config file");

    // Verify the API key is not in the file
    assert!(
        !content.contains("cli_integration_api_key"),
        "API key from CLI integration was found in project config file"
    );

    // Clean up - restore original directory
    env::set_current_dir(original_dir).unwrap();
}
