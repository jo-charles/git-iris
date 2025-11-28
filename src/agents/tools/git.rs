//! Git operations tools for Rig-based agents
//!
//! This module provides Git operations using Rig's tool system.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::context::ChangeType;
use crate::define_tool_error;
use crate::git::StagedFile;

use super::common::{get_current_repo, parameters_schema};

define_tool_error!(GitError);

/// Helper to add a change type if not already present
fn add_change(changes: &mut Vec<&'static str>, change: &'static str) {
    if !changes.contains(&change) {
        changes.push(change);
    }
}

/// Check for function definitions in a line based on language
fn is_function_def(line: &str, ext: &str) -> bool {
    match ext {
        "rs" => {
            line.starts_with("pub fn ")
                || line.starts_with("fn ")
                || line.starts_with("pub async fn ")
                || line.starts_with("async fn ")
        }
        "ts" | "tsx" | "js" | "jsx" => {
            line.starts_with("function ")
                || line.starts_with("async function ")
                || line.contains(" = () =>")
                || line.contains(" = async () =>")
        }
        "py" => line.starts_with("def ") || line.starts_with("async def "),
        "go" => line.starts_with("func "),
        _ => false,
    }
}

/// Check for import statements based on language
fn is_import(line: &str, ext: &str) -> bool {
    match ext {
        "rs" => line.starts_with("use ") || line.starts_with("pub use "),
        "ts" | "tsx" | "js" | "jsx" => line.starts_with("import ") || line.starts_with("export "),
        "py" => line.starts_with("import ") || line.starts_with("from "),
        "go" => line.starts_with("import "),
        _ => false,
    }
}

/// Check for type definitions based on language
fn is_type_def(line: &str, ext: &str) -> bool {
    match ext {
        "rs" => {
            line.starts_with("pub struct ")
                || line.starts_with("struct ")
                || line.starts_with("pub enum ")
                || line.starts_with("enum ")
        }
        "ts" | "tsx" | "js" | "jsx" => {
            line.starts_with("interface ")
                || line.starts_with("type ")
                || line.starts_with("class ")
        }
        "py" => line.starts_with("class "),
        "go" => line.starts_with("type "),
        _ => false,
    }
}

/// Detect semantic change types from diff content
#[allow(clippy::cognitive_complexity)]
fn detect_semantic_changes(diff: &str, path: &str) -> Vec<&'static str> {
    use std::path::Path;

    let mut changes = Vec::new();

    // Get file extension
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    // Only analyze supported languages
    let supported = matches!(
        ext.as_str(),
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go"
    );

    if supported {
        // Analyze added lines for patterns
        for line in diff
            .lines()
            .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
        {
            let line = line.trim_start_matches('+').trim();

            if is_function_def(line, &ext) {
                add_change(&mut changes, "adds function");
            }
            if is_import(line, &ext) {
                add_change(&mut changes, "modifies imports");
            }
            if is_type_def(line, &ext) {
                add_change(&mut changes, "adds type");
            }
            // Rust-specific: impl blocks
            if ext == "rs" && line.starts_with("impl ") {
                add_change(&mut changes, "adds impl");
            }
        }
    }

    // Check for general change patterns
    let has_deletions = diff
        .lines()
        .any(|l| l.starts_with('-') && !l.starts_with("---"));
    let has_additions = diff
        .lines()
        .any(|l| l.starts_with('+') && !l.starts_with("+++"));

    if has_deletions && has_additions && changes.is_empty() {
        changes.push("refactors code");
    } else if has_deletions && !has_additions {
        changes.push("removes code");
    }

    changes
}

/// Calculate relevance score for a file (0.0 - 1.0)
/// Higher score = more important for commit message
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn calculate_relevance_score(file: &StagedFile) -> (f32, Vec<&'static str>) {
    let mut score: f32 = 0.5; // Base score
    let mut reasons = Vec::new();
    let path = file.path.to_lowercase();

    // Change type scoring
    match file.change_type {
        ChangeType::Added => {
            score += 0.15;
            reasons.push("new file");
        }
        ChangeType::Modified => {
            score += 0.1;
        }
        ChangeType::Deleted => {
            score += 0.05;
            reasons.push("deleted");
        }
    }

    // File type scoring - source code is most important
    if path.ends_with(".rs")
        || path.ends_with(".py")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".go")
        || path.ends_with(".java")
        || path.ends_with(".kt")
        || path.ends_with(".swift")
        || path.ends_with(".c")
        || path.ends_with(".cpp")
        || path.ends_with(".h")
    {
        score += 0.15;
        reasons.push("source code");
    } else if path.ends_with(".toml")
        || path.ends_with(".json")
        || path.ends_with(".yaml")
        || path.ends_with(".yml")
    {
        score += 0.1;
        reasons.push("config");
    } else if path.ends_with(".md") || path.ends_with(".txt") || path.ends_with(".rst") {
        score += 0.02;
        reasons.push("docs");
    }

    // Path-based scoring
    if path.contains("/src/") || path.starts_with("src/") {
        score += 0.1;
        reasons.push("core source");
    }
    if path.contains("/test") || path.contains("_test.") || path.contains(".test.") {
        score -= 0.1;
        reasons.push("test file");
    }
    if path.contains("generated") || path.contains(".lock") || path.contains("package-lock") {
        score -= 0.2;
        reasons.push("generated/lock");
    }
    if path.contains("/vendor/") || path.contains("/node_modules/") {
        score -= 0.3;
        reasons.push("vendored");
    }

    // Diff size scoring (estimate from diff length)
    let diff_lines = file.diff.lines().count();
    if diff_lines > 10 && diff_lines < 200 {
        score += 0.1;
        reasons.push("substantive changes");
    } else if diff_lines >= 200 {
        score += 0.05;
        reasons.push("large diff");
    }

    // Add semantic change detection
    let semantic_changes = detect_semantic_changes(&file.diff, &file.path);
    for change in semantic_changes {
        if !reasons.contains(&change) {
            // Boost score for structural changes
            if change == "adds function" || change == "adds type" || change == "adds impl" {
                score += 0.1;
            }
            reasons.push(change);
        }
    }

    // Clamp to 0.0-1.0
    score = score.clamp(0.0, 1.0);

    (score, reasons)
}

/// Scored file for output
struct ScoredFile<'a> {
    file: &'a StagedFile,
    score: f32,
    reasons: Vec<&'static str>,
}

/// Build the diff output string from scored files
fn format_diff_output(
    scored_files: &[ScoredFile],
    total_files: usize,
    is_filtered: bool,
    include_diffs: bool,
) -> String {
    let mut output = String::new();
    let showing = scored_files.len();

    // Calculate stats
    let additions: usize = scored_files
        .iter()
        .map(|sf| sf.file.diff.lines().filter(|l| l.starts_with('+')).count())
        .sum();
    let deletions: usize = scored_files
        .iter()
        .map(|sf| sf.file.diff.lines().filter(|l| l.starts_with('-')).count())
        .sum();
    let total_lines = additions + deletions;

    // Categorize size
    let (size, guidance) = if is_filtered {
        ("Filtered", "Showing requested files only.")
    } else if total_files <= 3 && total_lines < 100 {
        ("Small", "Focus on all files equally.")
    } else if total_files <= 10 && total_lines < 500 {
        ("Medium", "Prioritize files with >60% relevance.")
    } else {
        (
            "Large",
            "Use files=['path1','path2'] with detail='standard' to analyze specific files.",
        )
    };

    // Header
    let files_info = if is_filtered {
        format!("{showing} of {total_files} files")
    } else {
        format!("{total_files} files")
    };
    output.push_str(&format!(
        "=== CHANGES SUMMARY ===\n{files_info} | +{additions} -{deletions} | Size: {size} ({total_lines} lines)\nGuidance: {guidance}\n\n"
    ));

    // File list
    output.push_str("Files by importance:\n");
    for sf in scored_files {
        let reasons = if sf.reasons.is_empty() {
            String::new()
        } else {
            format!(" ({})", sf.reasons.join(", "))
        };
        output.push_str(&format!(
            "  [{:.0}%] {:?} {}{reasons}\n",
            sf.score * 100.0,
            sf.file.change_type,
            sf.file.path
        ));
    }
    output.push('\n');

    // Diffs or hint
    if include_diffs {
        output.push_str("=== DIFFS ===\n");
        for sf in scored_files {
            output.push_str(&format!(
                "--- {} [{:.0}% relevance]\n",
                sf.file.path,
                sf.score * 100.0
            ));
            output.push_str(&sf.file.diff);
            output.push('\n');
        }
    } else if is_filtered {
        output.push_str("(Use detail='standard' to see full diffs for these files)\n");
    } else {
        output.push_str(
            "(Use detail='standard' with files=['file1','file2'] to see specific diffs)\n",
        );
    }

    output
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
        let repo = get_current_repo().map_err(GitError::from)?;

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

/// Detail level for diff output
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum DetailLevel {
    /// Summary only: file list with stats and relevance scores, no diffs (default)
    #[default]
    Summary,
    /// Standard: includes full diffs (use with `files` filter for large changesets)
    Standard,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitDiffArgs {
    /// Use "staged" or omit for staged changes, or specify commit/branch
    #[serde(default)]
    pub from: Option<String>,
    /// Target commit/branch (use with from)
    #[serde(default)]
    pub to: Option<String>,
    /// Detail level: "summary" (default) for overview, "standard" for full diffs
    #[serde(default)]
    pub detail: DetailLevel,
    /// Filter to specific files (use with detail="standard" for targeted analysis)
    #[serde(default)]
    pub files: Option<Vec<String>>,
}

impl Tool for GitDiff {
    const NAME: &'static str = "git_diff";
    type Error = GitError;
    type Args = GitDiffArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "git_diff".to_string(),
            description: "Get Git diff for file changes. Returns summary by default (file list with relevance scores). Use detail='standard' with files=['path1','path2'] to get full diffs for specific files. Progressive approach: call once for summary, then again with files filter for important ones.".to_string(),
            parameters: parameters_schema::<GitDiffArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let repo = get_current_repo().map_err(GitError::from)?;

        // Normalize empty strings to None (LLMs often send "" instead of null)
        let from = args.from.filter(|s| !s.is_empty());
        let to = args.to.filter(|s| !s.is_empty());

        // Handle the case where we want staged changes
        // - No args: get staged changes
        // - from="staged": get staged changes
        // - Otherwise: get commit range
        let files = match (from.as_deref(), to.as_deref()) {
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

        // Score and sort files by relevance
        let mut scored_files: Vec<ScoredFile> = files
            .iter()
            .map(|file| {
                let (score, reasons) = calculate_relevance_score(file);
                ScoredFile {
                    file,
                    score,
                    reasons,
                }
            })
            .collect();

        // Sort by score descending (most important first)
        scored_files.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Track total before filtering
        let total_files = scored_files.len();

        // Filter to specific files if requested
        let is_filtered = args.files.is_some();
        if let Some(ref filter) = args.files {
            scored_files.retain(|sf| filter.iter().any(|f| sf.file.path.contains(f)));
        }

        // Build output
        let include_diffs = matches!(args.detail, DetailLevel::Standard);
        Ok(format_diff_output(
            &scored_files,
            total_files,
            is_filtered,
            include_diffs,
        ))
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
        let repo = get_current_repo().map_err(GitError::from)?;

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
        let repo = get_current_repo().map_err(GitError::from)?;

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
        let repo = get_current_repo().map_err(GitError::from)?;

        // Normalize empty strings to None (LLMs often send "" instead of null)
        let from = args.from.filter(|s| !s.is_empty());
        let mut to = args.to.filter(|s| !s.is_empty());

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
