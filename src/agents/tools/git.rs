//! Git operations tools for Rig-based agents
//!
//! This module provides Git operations using Rig's tool system.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::git::GitRepo;

#[derive(Debug, thiserror::Error)]
#[error("Git error: {0}")]
pub struct GitError(String);

impl From<anyhow::Error> for GitError {
    fn from(err: anyhow::Error) -> Self {
        GitError(err.to_string())
    }
}

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        GitError(err.to_string())
    }
}

// Git status tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusArgs {
    pub include_unstaged: Option<bool>,
}

impl Tool for GitStatus {
    const NAME: &'static str = "git_status";
    type Error = GitError;
    type Args = GitStatusArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_status",
            "description": "Get current Git repository status including staged and unstaged files",
            "parameters": {
                "type": "object",
                "properties": {
                    "include_unstaged": {
                        "type": "boolean",
                        "description": "Whether to include unstaged files in the output"
                    }
                },
                "required": []
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let files_info = repo
            .extract_files_info(args.include_unstaged.unwrap_or(false))
            .map_err(GitError::from)?;

        let mut output = String::new();
        output.push_str(&format!("Branch: {}\n", files_info.branch));
        output.push_str(&format!(
            "Files changed: {}\n",
            files_info.staged_files.len()
        ));

        for file in &files_info.staged_files {
            output.push_str(&format!("  {}: {:?}\n", file.path, file.change_type));
        }

        Ok(output)
    }
}

// Git diff tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffArgs {
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Tool for GitDiff {
    const NAME: &'static str = "git_diff";
    type Error = GitError;
    type Args = GitDiffArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_diff",
            "description": "Get Git diff for changes between commits or branches",
            "parameters": {
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "Starting commit, branch, or reference"
                    },
                    "to": {
                        "type": "string",
                        "description": "Ending commit, branch, or reference"
                    }
                },
                "required": []
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let files = if let (Some(from), Some(to)) = (args.from, args.to) {
            repo.get_commit_range_files(&from, &to)
                .map_err(GitError::from)?
        } else {
            let files_info = repo.extract_files_info(false).map_err(GitError::from)?;
            files_info.staged_files
        };

        let mut output = String::new();
        output.push_str("File changes:\n");

        for file in &files {
            output.push_str(&format!("--- {}\n", file.path));
            output.push_str(&file.diff);
            output.push('\n');
        }

        Ok(output)
    }
}

// Git log tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLogArgs {
    pub count: Option<usize>,
}

impl Tool for GitLog {
    const NAME: &'static str = "git_log";
    type Error = GitError;
    type Args = GitLogArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_log",
            "description": "Get Git commit history",
            "parameters": {
                "type": "object",
                "properties": {
                    "count": {
                        "type": "integer",
                        "description": "Number of commits to retrieve (default: 10)"
                    }
                },
                "required": []
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let commits = repo
            .get_recent_commits(args.count.unwrap_or(10))
            .map_err(GitError::from)?;

        let mut output = String::new();
        output.push_str("Recent commits:\n");

        for commit in commits {
            output.push_str(&format!(
                "{}: {} ({})\n",
                commit.hash, commit.message, commit.author
            ));
        }

        Ok(output)
    }
}

// Git repository info tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoInfoArgs {}

impl Tool for GitRepoInfo {
    const NAME: &'static str = "git_repo_info";
    type Error = GitError;
    type Args = GitRepoInfoArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_repo_info",
            "description": "Get general information about the Git repository",
            "parameters": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }))
        .unwrap()
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let branch = repo.get_current_branch().map_err(GitError::from)?;
        let remote_url = repo.get_remote_url().unwrap_or("None").to_string();

        let mut output = String::new();
        output.push_str("Repository Information:\n");
        output.push_str(&format!("Current Branch: {branch}\n"));
        output.push_str(&format!("Remote URL: {remote_url}\n"));
        output.push_str(&format!(
            "Repository Path: {}\n",
            repo.repo_path().display()
        ));

        Ok(output)
    }
}

// Git changed files tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitChangedFiles;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitChangedFilesArgs {
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Tool for GitChangedFiles {
    const NAME: &'static str = "git_changed_files";
    type Error = GitError;
    type Args = GitChangedFilesArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "git_changed_files",
            "description": "Get list of files that have changed between commits or branches",
            "parameters": {
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "Starting commit or branch"
                    },
                    "to": {
                        "type": "string",
                        "description": "Ending commit or branch"
                    }
                },
                "required": []
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let files = match (args.from, args.to) {
            (Some(from), Some(to)) => {
                // When both from and to are provided, get files changed between commits/branches
                let range_files = repo.get_commit_range_files(&from, &to)
                    .map_err(GitError::from)?;
                range_files.iter().map(|f| f.path.clone()).collect()
            }
            (None, Some(to)) => {
                // When only to is provided, get files changed in that single commit
                repo.get_file_paths_for_commit(&to)
                    .map_err(GitError::from)?
            }
            (Some(_from), None) => {
                // Invalid: from without to doesn't make sense for file listing
                return Err(GitError("Cannot specify 'from' without 'to' for file listing".to_string()));
            }
            (None, None) => {
                // When neither are provided, get staged files
                let files_info = repo.extract_files_info(false).map_err(GitError::from)?;
                files_info.file_paths
            }
        };

        let mut output = String::new();
        output.push_str("Changed files:\n");

        for file in files {
            output.push_str(&format!("  {file}\n"));
        }

        Ok(output)
    }
}
