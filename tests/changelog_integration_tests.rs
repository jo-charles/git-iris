// tests/changelog_integration_tests.rs

#![cfg(feature = "integration")]

use anyhow::Result;
use dotenv::dotenv;
use git_iris::changes::models::{ChangelogResponse, ReleaseNotesResponse};
use git_iris::changes::{ChangelogGenerator, ReleaseNotesGenerator};
use git_iris::common::DetailLevel;
use git_iris::config::Config;
use git_iris::logger;
use git2::Repository;
use std::env;
use std::path::Path;
use tempfile::TempDir;

fn setup_test_repo() -> Result<(TempDir, Repository)> {
    let _ = logger::init(); // Initialize the logger
    logger::enable_logging(); // Enable logging
    logger::set_log_to_stdout(true);

    let temp_dir = TempDir::new()?;
    let repo = Repository::init(temp_dir.path())?;

    let signature = git2::Signature::now("Test User", "test@example.com")?;

    // Create initial commit
    {
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
    }

    // Create a tag for the initial commit (v1.0.0)
    {
        let head = repo.head()?.peel_to_commit()?;
        repo.tag(
            "v1.0.0",
            &head.into_object(),
            &signature,
            "Version 1.0.0",
            false,
        )?;
    }

    // Create a new file and commit
    std::fs::write(temp_dir.path().join("file1.txt"), "Hello, world!")?;
    {
        let mut index = repo.index()?;
        index.add_path(Path::new("file1.txt"))?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let parent = repo.head()?.peel_to_commit()?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add file1.txt",
            &tree,
            &[&parent],
        )?;
    }

    // Create another tag (v1.1.0)
    {
        let head = repo.head()?.peel_to_commit()?;
        repo.tag(
            "v1.1.0",
            &head.into_object(),
            &signature,
            "Version 1.1.0",
            false,
        )?;
    }

    Ok((temp_dir, repo))
}

fn setup_config() -> Result<Config> {
    dotenv().ok();
    let mut config = Config::default();
    config.default_provider = "openai".to_string();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    config
        .providers
        .get_mut(&config.default_provider)
        .unwrap()
        .api_key = api_key;
    Ok(config)
}

#[tokio::test]
async fn test_changelog_generation() -> Result<()> {
    let (temp_dir, _repo) = setup_test_repo()?;
    let config = setup_config()?;

    let changelog = ChangelogGenerator::generate(
        temp_dir.path(),
        "v1.0.0",
        "v1.1.0",
        &config,
        DetailLevel::Standard,
    )
    .await?;

    let changelog_response: ChangelogResponse = serde_json::from_str(&changelog)?;

    assert!(
        changelog_response.version.is_some(),
        "Changelog should have a version"
    );
    assert!(
        changelog_response.release_date.is_some(),
        "Changelog should have a release date"
    );
    assert!(
        !changelog_response.sections.is_empty(),
        "Changelog should have sections"
    );
    assert!(
        changelog_response.metrics.total_commits > 0,
        "Changelog should have commits"
    );
    assert!(
        changelog_response.metrics.files_changed > 0,
        "Changelog should have file changes"
    );

    Ok(())
}

#[tokio::test]
async fn test_release_notes_generation() -> Result<()> {
    let (temp_dir, _repo) = setup_test_repo()?;
    let config = setup_config()?;

    let release_notes = ReleaseNotesGenerator::generate(
        temp_dir.path(),
        "v1.0.0",
        "v1.1.0",
        &config,
        DetailLevel::Standard,
    )
    .await?;

    let release_notes_response: ReleaseNotesResponse = serde_json::from_str(&release_notes)?;

    assert!(
        release_notes_response.version.is_some(),
        "Release notes should have a version"
    );
    assert!(
        release_notes_response.release_date.is_some(),
        "Release notes should have a release date"
    );
    assert!(
        !release_notes_response.summary.is_empty(),
        "Release notes should have a summary"
    );
    assert!(
        release_notes_response.metrics.total_commits > 0,
        "Release notes should have commits"
    );
    assert!(
        release_notes_response.metrics.files_changed > 0,
        "Release notes should have file changes"
    );

    Ok(())
}

#[tokio::test]
async fn test_changelog_generation_with_custom_version() -> Result<()> {
    let (temp_dir, _repo) = setup_test_repo()?;
    let config = setup_config()?;
    let custom_version = "v2.0.0-beta";

    // We need to provide a path to GitRepo for this integration test
    let repo_path = std::sync::Arc::new(git_iris::git::GitRepo::new(temp_dir.path())?);

    // Generate changelog with custom version name
    let changelog = ChangelogGenerator::generate(
        repo_path.clone(),
        "v1.0.0",
        "v1.1.0",
        &config,
        DetailLevel::Standard,
    )
    .await?;

    // Generate a temporary changelog file using the custom version
    let changelog_path = temp_dir.path().join("CHANGELOG.md");
    ChangelogGenerator::update_changelog_file(
        &changelog,
        changelog_path.to_str().unwrap(),
        &repo_path,
        "HEAD",
        Some(custom_version.to_string()),
    )?;

    // Read the content to verify the custom version was used
    let content = std::fs::read_to_string(&changelog_path)?;
    assert!(
        content.contains(&format!("## [{}]", custom_version)),
        "Changelog should contain the custom version name"
    );

    Ok(())
}

#[tokio::test]
async fn test_release_notes_generation_with_custom_version() -> Result<()> {
    let (temp_dir, _repo) = setup_test_repo()?;
    let config = setup_config()?;
    let custom_version = "v2.0.0-rc1";

    // We need to provide a path to GitRepo for this integration test
    let repo_path = std::sync::Arc::new(git_iris::git::GitRepo::new(temp_dir.path())?);

    // Generate release notes with custom version name
    let release_notes = ReleaseNotesGenerator::generate(
        repo_path,
        "v1.0.0",
        "v1.1.0",
        &config,
        DetailLevel::Standard,
        Some(custom_version.to_string()),
    )
    .await?;

    // Verify the custom version was used
    assert!(
        release_notes.contains(&format!("Release Notes - v{}", custom_version)),
        "Release notes should contain the custom version name"
    );

    Ok(())
}
