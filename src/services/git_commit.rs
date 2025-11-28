//! Git commit service
//!
//! Focused service for git commit operations. This extracts the commit-specific
//! functionality from the monolithic `IrisCommitService`.

use anyhow::Result;
use std::sync::Arc;

use crate::git::{CommitResult, GitRepo};
use crate::gitmoji::process_commit_message;
use crate::log_debug;

/// Service for performing git commit operations
///
/// This service handles:
/// - Creating commits with optional hook verification
/// - Pre-commit hook execution
/// - Remote repository detection
///
/// It does NOT handle:
/// - LLM operations (handled by `IrisAgentService`)
/// - Context gathering (handled by agents)
/// - Message generation (handled by agents)
pub struct GitCommitService {
    repo: Arc<GitRepo>,
    use_gitmoji: bool,
    verify: bool,
}

impl GitCommitService {
    /// Create a new `GitCommitService`
    ///
    /// # Arguments
    /// * `repo` - The git repository to operate on
    /// * `use_gitmoji` - Whether to apply gitmoji to commit messages
    /// * `verify` - Whether to run pre/post-commit hooks
    pub fn new(repo: Arc<GitRepo>, use_gitmoji: bool, verify: bool) -> Self {
        Self {
            repo,
            use_gitmoji,
            verify,
        }
    }

    /// Create from an existing `GitRepo` (convenience constructor)
    pub fn from_repo(repo: GitRepo, use_gitmoji: bool, verify: bool) -> Self {
        Self::new(Arc::new(repo), use_gitmoji, verify)
    }

    /// Check if the repository is a remote repository
    pub fn is_remote(&self) -> bool {
        self.repo.is_remote()
    }

    /// Execute the pre-commit hook if verification is enabled
    ///
    /// Returns Ok(()) if:
    /// - verify is false (hooks disabled)
    /// - repository is remote (hooks don't apply)
    /// - pre-commit hook succeeds
    pub fn pre_commit(&self) -> Result<()> {
        if self.is_remote() {
            log_debug!("Skipping pre-commit hook for remote repository");
            return Ok(());
        }

        if self.verify {
            self.repo.execute_hook("pre-commit")
        } else {
            Ok(())
        }
    }

    /// Perform a commit with the given message
    ///
    /// This method:
    /// 1. Validates the repository is not remote
    /// 2. Processes the message (applies gitmoji if enabled)
    /// 3. Runs pre-commit hook (if verify is enabled)
    /// 4. Creates the commit
    /// 5. Runs post-commit hook (if verify is enabled)
    ///
    /// # Arguments
    /// * `message` - The commit message to use
    ///
    /// # Returns
    /// The result of the commit operation
    pub fn perform_commit(&self, message: &str) -> Result<CommitResult> {
        if self.is_remote() {
            return Err(anyhow::anyhow!("Cannot commit to a remote repository"));
        }

        let processed_message = process_commit_message(message.to_string(), self.use_gitmoji);
        log_debug!("Performing commit with message: {}", processed_message);

        if !self.verify {
            log_debug!("Skipping pre-commit hook (verify=false)");
            return self.repo.commit(&processed_message);
        }

        // Execute pre-commit hook
        log_debug!("Executing pre-commit hook");
        if let Err(e) = self.repo.execute_hook("pre-commit") {
            log_debug!("Pre-commit hook failed: {}", e);
            return Err(e);
        }
        log_debug!("Pre-commit hook executed successfully");

        // Perform the commit
        match self.repo.commit(&processed_message) {
            Ok(result) => {
                // Execute post-commit hook (failure doesn't fail the commit)
                log_debug!("Executing post-commit hook");
                if let Err(e) = self.repo.execute_hook("post-commit") {
                    log_debug!("Post-commit hook failed: {}", e);
                }
                log_debug!("Commit performed successfully");
                Ok(result)
            }
            Err(e) => {
                log_debug!("Commit failed: {}", e);
                Err(e)
            }
        }
    }

    /// Amend the previous commit with staged changes and a new message
    ///
    /// This method:
    /// 1. Validates the repository is not remote
    /// 2. Processes the message (applies gitmoji if enabled)
    /// 3. Runs pre-commit hook (if verify is enabled)
    /// 4. Amends the commit (replaces HEAD)
    /// 5. Runs post-commit hook (if verify is enabled)
    ///
    /// # Arguments
    /// * `message` - The new commit message
    ///
    /// # Returns
    /// The result of the amend operation
    pub fn perform_amend(&self, message: &str) -> Result<CommitResult> {
        if self.is_remote() {
            return Err(anyhow::anyhow!(
                "Cannot amend a commit in a remote repository"
            ));
        }

        let processed_message = process_commit_message(message.to_string(), self.use_gitmoji);
        log_debug!("Performing amend with message: {}", processed_message);

        if !self.verify {
            log_debug!("Skipping pre-commit hook (verify=false)");
            return self.repo.amend_commit(&processed_message);
        }

        // Execute pre-commit hook
        log_debug!("Executing pre-commit hook");
        if let Err(e) = self.repo.execute_hook("pre-commit") {
            log_debug!("Pre-commit hook failed: {}", e);
            return Err(e);
        }
        log_debug!("Pre-commit hook executed successfully");

        // Perform the amend
        match self.repo.amend_commit(&processed_message) {
            Ok(result) => {
                // Execute post-commit hook (failure doesn't fail the amend)
                log_debug!("Executing post-commit hook");
                if let Err(e) = self.repo.execute_hook("post-commit") {
                    log_debug!("Post-commit hook failed: {}", e);
                }
                log_debug!("Amend performed successfully");
                Ok(result)
            }
            Err(e) => {
                log_debug!("Amend failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get the message of the HEAD commit
    ///
    /// Useful for amend operations to provide original context
    pub fn get_head_commit_message(&self) -> Result<String> {
        self.repo.get_head_commit_message()
    }

    /// Get a reference to the underlying repository
    pub fn repo(&self) -> &GitRepo {
        &self.repo
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_git_commit_service_construction() {
        // This test just verifies the API compiles correctly
        // Real tests would need a mock GitRepo
    }
}
