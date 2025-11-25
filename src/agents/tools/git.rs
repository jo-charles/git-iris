//! Git operations tools for Rig-based agents
//!
//! This module provides Git operations using Rig's tool system.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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

/// `OpenAI` tool schemas require the `required` array to list every property.
fn parameters_schema<T: schemars::JsonSchema>() -> Value {
    use schemars::schema_for;

    let schema = schema_for!(T);
    let mut value = serde_json::to_value(schema).expect("tool schema should serialize");
    enforce_required_properties(&mut value);
    value
}

fn enforce_required_properties(value: &mut Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };

    let props_entry = obj
        .entry("properties")
        .or_insert_with(|| Value::Object(Map::new()));
    let props_obj = props_entry.as_object().expect("properties must be object");
    let required_keys: Vec<Value> = props_obj.keys().cloned().map(Value::String).collect();

    obj.insert("required".to_string(), Value::Array(required_keys));
}

// Git status tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitStatusArgs {
    #[serde(default)]
    pub include_unstaged: bool,
}

impl Tool for GitStatus {
    const NAME: &'static str = "git_status";
    type Error = GitError;
    type Args = GitStatusArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "git_status".to_string(),
            description: "Get current Git repository status including staged and unstaged files"
                .to_string(),
            parameters: parameters_schema::<GitStatusArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let files_info = repo
            .extract_files_info(args.include_unstaged)
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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitDiffArgs {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

impl Tool for GitDiff {
    const NAME: &'static str = "git_diff";
    type Error = GitError;
    type Args = GitDiffArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "git_diff".to_string(),
            description: "Get Git diff for file changes. Use with no args or from='staged' to get staged changes. Otherwise provide from/to commits or branches.".to_string(),
            parameters: parameters_schema::<GitDiffArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        // Handle the case where we want staged changes
        // - No args: get staged changes
        // - from="staged": get staged changes
        // - Otherwise: get commit range
        let files = match (args.from.as_deref(), args.to.as_deref()) {
            (None | Some("staged"), None) | (Some("staged"), Some("HEAD")) => {
                // Get staged changes
                let files_info = repo.extract_files_info(false).map_err(GitError::from)?;
                files_info.staged_files
            }
            (Some(from), Some(to)) => {
                // Get changes between two commits/branches
                repo.get_commit_range_files(from, to)
                    .map_err(GitError::from)?
            }
            (None, Some(_)) => {
                // Invalid: to without from
                return Err(GitError(
                    "Cannot specify 'to' without 'from'. Use both or neither.".to_string(),
                ));
            }
            (Some(from), None) => {
                // Get changes from a specific commit to HEAD (already handled "staged" above)
                repo.get_commit_range_files(from, "HEAD")
                    .map_err(GitError::from)?
            }
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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitLogArgs {
    #[serde(default)]
    pub count: Option<usize>,
}

impl Tool for GitLog {
    const NAME: &'static str = "git_log";
    type Error = GitError;
    type Args = GitLogArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "git_log".to_string(),
            description: "Get Git commit history".to_string(),
            parameters: parameters_schema::<GitLogArgs>(),
        }
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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitRepoInfoArgs {}

impl Tool for GitRepoInfo {
    const NAME: &'static str = "git_repo_info";
    type Error = GitError;
    type Args = GitRepoInfoArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "git_repo_info".to_string(),
            description: "Get general information about the Git repository".to_string(),
            parameters: parameters_schema::<GitRepoInfoArgs>(),
        }
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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitChangedFilesArgs {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

impl Tool for GitChangedFiles {
    const NAME: &'static str = "git_changed_files";
    type Error = GitError;
    type Args = GitChangedFilesArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "git_changed_files".to_string(),
            description: "Get list of files that have changed between commits or branches"
                .to_string(),
            parameters: parameters_schema::<GitChangedFilesArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(GitError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(GitError::from)?;

        let from = args.from;
        let mut to = args.to;

        // Default to HEAD when the caller provides only a starting point.
        if from.is_some() && to.is_none() {
            to = Some("HEAD".to_string());
        }

        let files = match (from, to) {
            (Some(from), Some(to)) => {
                // When both from and to are provided, get files changed between commits/branches
                let range_files = repo
                    .get_commit_range_files(&from, &to)
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
                return Err(GitError(
                    "Cannot specify 'from' without 'to' for file listing".to_string(),
                ));
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
