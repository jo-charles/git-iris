use crate::config::Config;
use crate::context::{CommitContext, ProjectMetadata, RecentCommit, StagedFile};
use crate::git::commit::{self, CommitResult};
use crate::git::files::{RepoFilesInfo, get_file_statuses, get_unstaged_file_statuses};
use crate::git::metadata;
use crate::git::utils::is_inside_work_tree;
use crate::log_debug;
use anyhow::{Context as AnyhowContext, Result, anyhow};
use git2::{Repository, Tree};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use url::Url;

/// Represents a Git repository and provides methods for interacting with it.
pub struct GitRepo {
    repo_path: PathBuf,
    /// Optional temporary directory for cloned repositories
    #[allow(dead_code)] // This field is needed to maintain ownership of temp directories
    temp_dir: Option<TempDir>,
    /// Whether this is a remote repository
    is_remote: bool,
    /// Original remote URL if this is a cloned repository
    remote_url: Option<String>,
}

impl GitRepo {
    /// Creates a new `GitRepo` instance from a local path.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The path to the Git repository.
    ///
    /// # Returns
    ///
    /// A Result containing the `GitRepo` instance or an error.
    pub fn new(repo_path: &Path) -> Result<Self> {
        Ok(Self {
            repo_path: repo_path.to_path_buf(),
            temp_dir: None,
            is_remote: false,
            remote_url: None,
        })
    }

    /// Creates a new `GitRepo` instance, handling both local and remote repositories.
    ///
    /// # Arguments
    ///
    /// * `repository_url` - Optional URL for a remote repository.
    ///
    /// # Returns
    ///
    /// A Result containing the `GitRepo` instance or an error.
    pub fn new_from_url(repository_url: Option<String>) -> Result<Self> {
        if let Some(url) = repository_url {
            Self::clone_remote_repository(&url)
        } else {
            let current_dir = env::current_dir()?;
            Self::new(&current_dir)
        }
    }

    /// Clones a remote repository and creates a `GitRepo` instance for it.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the remote repository to clone.
    ///
    /// # Returns
    ///
    /// A Result containing the `GitRepo` instance or an error.
    pub fn clone_remote_repository(url: &str) -> Result<Self> {
        log_debug!("Cloning remote repository from URL: {}", url);

        // Validate URL
        let _ = Url::parse(url).map_err(|e| anyhow!("Invalid repository URL: {}", e))?;

        // Create a temporary directory for the clone
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        log_debug!("Created temporary directory for clone: {:?}", temp_path);

        // Clone the repository into the temporary directory
        let repo = match Repository::clone(url, temp_path) {
            Ok(repo) => repo,
            Err(e) => return Err(anyhow!("Failed to clone repository: {}", e)),
        };

        log_debug!("Successfully cloned repository to {:?}", repo.path());

        Ok(Self {
            repo_path: temp_path.to_path_buf(),
            temp_dir: Some(temp_dir),
            is_remote: true,
            remote_url: Some(url.to_string()),
        })
    }

    /// Open the repository at the stored path
    pub fn open_repo(&self) -> Result<Repository, git2::Error> {
        Repository::open(&self.repo_path)
    }

    /// Returns whether this `GitRepo` instance is working with a remote repository
    pub fn is_remote(&self) -> bool {
        self.is_remote
    }

    /// Returns the original remote URL if this is a cloned repository
    pub fn get_remote_url(&self) -> Option<&str> {
        self.remote_url.as_deref()
    }

    /// Returns the repository path
    pub fn repo_path(&self) -> &PathBuf {
        &self.repo_path
    }

    /// Updates the remote repository by fetching the latest changes
    pub fn update_remote(&self) -> Result<()> {
        if !self.is_remote {
            return Err(anyhow!("Not a remote repository"));
        }

        log_debug!("Updating remote repository");
        let repo = self.open_repo()?;

        // Find the default remote (usually "origin")
        let remotes = repo.remotes()?;
        let remote_name = remotes
            .iter()
            .flatten()
            .next()
            .ok_or_else(|| anyhow!("No remote found"))?;

        // Fetch updates from the remote
        let mut remote = repo.find_remote(remote_name)?;
        remote.fetch(&["master", "main"], None, None)?;

        log_debug!("Successfully updated remote repository");
        Ok(())
    }

    /// Retrieves the current branch name.
    ///
    /// # Returns
    ///
    /// A Result containing the branch name as a String or an error.
    pub fn get_current_branch(&self) -> Result<String> {
        let repo = self.open_repo()?;
        let head = repo.head()?;
        let branch_name = head.shorthand().unwrap_or("HEAD detached").to_string();
        log_debug!("Current branch: {}", branch_name);
        Ok(branch_name)
    }

    /// Executes a Git hook.
    ///
    /// # Arguments
    ///
    /// * `hook_name` - The name of the hook to execute.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error.
    pub fn execute_hook(&self, hook_name: &str) -> Result<()> {
        if self.is_remote {
            log_debug!("Skipping hook execution for remote repository");
            return Ok(());
        }

        let repo = self.open_repo()?;
        let hook_path = repo.path().join("hooks").join(hook_name);

        if hook_path.exists() {
            log_debug!("Executing hook: {}", hook_name);
            log_debug!("Hook path: {:?}", hook_path);

            // Get the repository's working directory (top level)
            let repo_workdir = repo
                .workdir()
                .context("Repository has no working directory")?;
            log_debug!("Repository working directory: {:?}", repo_workdir);

            // Create a command with the proper environment and working directory
            let mut command = Command::new(&hook_path);
            command
                .current_dir(repo_workdir) // Use the repository's working directory, not .git
                .env("GIT_DIR", repo.path()) // Set GIT_DIR to the .git directory
                .env("GIT_WORK_TREE", repo_workdir) // Set GIT_WORK_TREE to the working directory
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            log_debug!("Executing hook command: {:?}", command);

            let mut child = command.spawn()?;

            let stdout = child.stdout.take().context("Could not get stdout")?;
            let stderr = child.stderr.take().context("Could not get stderr")?;

            std::thread::spawn(move || {
                std::io::copy(&mut std::io::BufReader::new(stdout), &mut std::io::stdout())
                    .expect("Failed to copy data to stdout");
            });
            std::thread::spawn(move || {
                std::io::copy(&mut std::io::BufReader::new(stderr), &mut std::io::stderr())
                    .expect("Failed to copy data to stderr");
            });

            let status = child.wait()?;

            if !status.success() {
                return Err(anyhow!(
                    "Hook '{}' failed with exit code: {:?}",
                    hook_name,
                    status.code()
                ));
            }

            log_debug!("Hook '{}' executed successfully", hook_name);
        } else {
            log_debug!("Hook '{}' not found at {:?}", hook_name, hook_path);
        }

        Ok(())
    }

    /// Get the root directory of the current git repository
    pub fn get_repo_root() -> Result<PathBuf> {
        // Check if we're in a git repository
        if !is_inside_work_tree()? {
            return Err(anyhow!(
                "Not in a Git repository. Please run this command from within a Git repository."
            ));
        }

        // Use git rev-parse to find the repository root
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to execute git command")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to get repository root: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Convert the output to a path
        let root = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 output from git command")?
            .trim()
            .to_string();

        Ok(PathBuf::from(root))
    }

    /// Retrieves the README content at a specific commit.
    ///
    /// # Arguments
    ///
    /// * `commit_ish` - A string that resolves to a commit.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<String> with the README content or an error.
    pub fn get_readme_at_commit(&self, commit_ish: &str) -> Result<Option<String>> {
        let repo = self.open_repo()?;
        let obj = repo.revparse_single(commit_ish)?;
        let tree = obj.peel_to_tree()?;

        Self::find_readme_in_tree(&repo, &tree)
            .context("Failed to find and read README at specified commit")
    }

    /// Finds a README file in the given tree.
    ///
    /// # Arguments
    ///
    /// * `tree` - A reference to a Git tree.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<String> with the README content or an error.
    fn find_readme_in_tree(repo: &Repository, tree: &Tree) -> Result<Option<String>> {
        log_debug!("Searching for README file in the repository");

        let readme_patterns = [
            "README.md",
            "README.markdown",
            "README.txt",
            "README",
            "Readme.md",
            "readme.md",
        ];

        for entry in tree {
            let name = entry.name().unwrap_or("");
            if readme_patterns
                .iter()
                .any(|&pattern| name.eq_ignore_ascii_case(pattern))
            {
                let object = entry.to_object(repo)?;
                if let Some(blob) = object.as_blob() {
                    if let Ok(content) = std::str::from_utf8(blob.content()) {
                        log_debug!("README file found: {}", name);
                        return Ok(Some(content.to_string()));
                    }
                }
            }
        }

        log_debug!("No README file found");
        Ok(None)
    }

    /// Extract files info without crossing async boundaries
    pub fn extract_files_info(&self, include_unstaged: bool) -> Result<RepoFilesInfo> {
        let repo = self.open_repo()?;

        // Get basic repo info
        let branch = self.get_current_branch()?;
        let recent_commits = self.get_recent_commits(5)?;

        // Get staged and unstaged files
        let mut staged_files = get_file_statuses(&repo)?;
        if include_unstaged {
            let unstaged_files = self.get_unstaged_files()?;
            staged_files.extend(unstaged_files);
            log_debug!("Combined {} files (staged + unstaged)", staged_files.len());
        }

        // Extract file paths for metadata
        let file_paths: Vec<String> = staged_files.iter().map(|file| file.path.clone()).collect();

        Ok(RepoFilesInfo {
            branch,
            recent_commits,
            staged_files,
            file_paths,
        })
    }

    /// Gets unstaged file changes from the repository
    pub fn get_unstaged_files(&self) -> Result<Vec<StagedFile>> {
        let repo = self.open_repo()?;
        get_unstaged_file_statuses(&repo)
    }

    /// Retrieves project metadata for changed files.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - A slice of Strings representing the changed file paths.
    ///
    /// # Returns
    ///
    /// A Result containing the `ProjectMetadata` or an error.
    pub async fn get_project_metadata(&self, changed_files: &[String]) -> Result<ProjectMetadata> {
        // Default batch size of 10 files at a time to limit concurrency
        metadata::extract_project_metadata(changed_files, 10).await
    }

    /// Helper method for creating `CommitContext`
    ///
    /// # Arguments
    ///
    /// * `branch` - Branch name
    /// * `recent_commits` - List of recent commits
    /// * `staged_files` - List of staged files
    /// * `project_metadata` - Project metadata
    ///
    /// # Returns
    ///
    /// A Result containing the `CommitContext` or an error.
    fn create_commit_context(
        &self,
        branch: String,
        recent_commits: Vec<RecentCommit>,
        staged_files: Vec<StagedFile>,
        project_metadata: ProjectMetadata,
    ) -> Result<CommitContext> {
        // Get user info
        let repo = self.open_repo()?;
        let user_name = repo.config()?.get_string("user.name").unwrap_or_default();
        let user_email = repo.config()?.get_string("user.email").unwrap_or_default();

        // Create and return the context
        Ok(CommitContext::new(
            branch,
            recent_commits,
            staged_files,
            project_metadata,
            user_name,
            user_email,
        ))
    }

    /// Retrieves Git information for the repository.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object.
    ///
    /// # Returns
    ///
    /// A Result containing the `CommitContext` or an error.
    pub async fn get_git_info(&self, _config: &Config) -> Result<CommitContext> {
        // Get data that doesn't cross async boundaries
        let repo = self.open_repo()?;
        log_debug!("Getting git info for repo path: {:?}", repo.path());

        let branch = self.get_current_branch()?;
        let recent_commits = self.get_recent_commits(5)?;
        let staged_files = get_file_statuses(&repo)?;

        let changed_files: Vec<String> =
            staged_files.iter().map(|file| file.path.clone()).collect();

        log_debug!("Changed files for metadata extraction: {:?}", changed_files);

        // Get project metadata (async operation)
        let project_metadata = self.get_project_metadata(&changed_files).await?;
        log_debug!("Extracted project metadata: {:?}", project_metadata);

        // Create and return the context
        self.create_commit_context(branch, recent_commits, staged_files, project_metadata)
    }

    /// Get Git information including unstaged changes
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object
    /// * `include_unstaged` - Whether to include unstaged changes
    ///
    /// # Returns
    ///
    /// A Result containing the `CommitContext` or an error.
    pub async fn get_git_info_with_unstaged(
        &self,
        _config: &Config,
        include_unstaged: bool,
    ) -> Result<CommitContext> {
        log_debug!("Getting git info with unstaged flag: {}", include_unstaged);

        // Extract all git2 data before crossing async boundaries
        let files_info = self.extract_files_info(include_unstaged)?;

        // Now perform async operations
        let project_metadata = self.get_project_metadata(&files_info.file_paths).await?;

        // Create and return the context
        self.create_commit_context(
            files_info.branch,
            files_info.recent_commits,
            files_info.staged_files,
            project_metadata,
        )
    }

    /// Retrieves recent commits.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of recent commits to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of `RecentCommit` objects or an error.
    pub fn get_recent_commits(&self, count: usize) -> Result<Vec<RecentCommit>> {
        let repo = self.open_repo()?;
        log_debug!("Fetching {} recent commits", count);
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let commits = revwalk
            .take(count)
            .map(|oid| {
                let oid = oid?;
                let commit = repo.find_commit(oid)?;
                let author = commit.author();
                Ok(RecentCommit {
                    hash: oid.to_string(),
                    message: commit.message().unwrap_or_default().to_string(),
                    author: author.name().unwrap_or_default().to_string(),
                    timestamp: commit.time().seconds().to_string(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        log_debug!("Retrieved {} recent commits", commits.len());
        Ok(commits)
    }

    /// Commits changes and verifies the commit.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message.
    ///
    /// # Returns
    ///
    /// A Result containing the `CommitResult` or an error.
    pub fn commit_and_verify(&self, message: &str) -> Result<CommitResult> {
        if self.is_remote {
            return Err(anyhow!(
                "Cannot commit to a remote repository in read-only mode"
            ));
        }

        let repo = self.open_repo()?;
        match commit::commit(&repo, message, self.is_remote) {
            Ok(result) => {
                if let Err(e) = self.execute_hook("post-commit") {
                    log_debug!("Post-commit hook failed: {}", e);
                }
                Ok(result)
            }
            Err(e) => {
                log_debug!("Commit failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get Git information for a specific commit
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object
    /// * `commit_id` - The ID of the commit to analyze
    ///
    /// # Returns
    ///
    /// A Result containing the `CommitContext` or an error.
    pub async fn get_git_info_for_commit(
        &self,
        _config: &Config,
        commit_id: &str,
    ) -> Result<CommitContext> {
        log_debug!("Getting git info for commit: {}", commit_id);
        let repo = self.open_repo()?;

        // Get branch name
        let branch = self.get_current_branch()?;

        // Extract commit info
        let commit_info = commit::extract_commit_info(&repo, commit_id, &branch)?;

        // Now get metadata with async operations
        let project_metadata = self.get_project_metadata(&commit_info.file_paths).await?;

        // Get the files from commit after async boundary
        let commit_files = commit::get_commit_files(&repo, commit_id)?;

        // Create and return the context
        self.create_commit_context(
            commit_info.branch,
            vec![commit_info.commit],
            commit_files,
            project_metadata,
        )
    }

    /// Get the commit date for a reference
    pub fn get_commit_date(&self, commit_ish: &str) -> Result<String> {
        let repo = self.open_repo()?;
        commit::get_commit_date(&repo, commit_ish)
    }

    /// Get commits between two references with a callback
    pub fn get_commits_between_with_callback<T, F>(
        &self,
        from: &str,
        to: &str,
        callback: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&RecentCommit) -> Result<T>,
    {
        let repo = self.open_repo()?;
        commit::get_commits_between_with_callback(&repo, from, to, callback)
    }

    /// Commit changes to the repository
    pub fn commit(&self, message: &str) -> Result<CommitResult> {
        let repo = self.open_repo()?;
        commit::commit(&repo, message, self.is_remote)
    }

    /// Check if inside a working tree
    pub fn is_inside_work_tree() -> Result<bool> {
        is_inside_work_tree()
    }

    /// Get the files changed in a specific commit
    pub fn get_commit_files(&self, commit_id: &str) -> Result<Vec<StagedFile>> {
        let repo = self.open_repo()?;
        commit::get_commit_files(&repo, commit_id)
    }

    /// Get just the file paths for a specific commit
    pub fn get_file_paths_for_commit(&self, commit_id: &str) -> Result<Vec<String>> {
        let repo = self.open_repo()?;
        commit::get_file_paths_for_commit(&repo, commit_id)
    }
}

impl Drop for GitRepo {
    fn drop(&mut self) {
        // The TempDir will be automatically cleaned up when dropped
        if self.is_remote {
            log_debug!("Cleaning up temporary repository at {:?}", self.repo_path);
        }
    }
}
