use crate::context::{ChangeType, RecentCommit, StagedFile};
use crate::git::utils::is_binary_diff;
use crate::log_debug;
use anyhow::{Context, Result, anyhow};
use chrono;
use git2::{FileMode, Repository, Status};

/// Results from a commit operation
#[derive(Debug)]
pub struct CommitResult {
    pub branch: String,
    pub commit_hash: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub new_files: Vec<(String, FileMode)>,
}

/// Collects information about a specific commit
#[derive(Debug)]
pub struct CommitInfo {
    pub branch: String,
    pub commit: RecentCommit,
    pub file_paths: Vec<String>,
}

/// Commits changes to the repository.
///
/// # Arguments
///
/// * `repo` - The git repository
/// * `message` - The commit message.
/// * `is_remote` - Whether the repository is remote.
///
/// # Returns
///
/// A Result containing the `CommitResult` or an error.
pub fn commit(repo: &Repository, message: &str, is_remote: bool) -> Result<CommitResult> {
    if is_remote {
        return Err(anyhow!(
            "Cannot commit to a remote repository in read-only mode"
        ));
    }

    let signature = repo.signature()?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    let commit_oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    let branch_name = repo.head()?.shorthand().unwrap_or("HEAD").to_string();
    let commit = repo.find_commit(commit_oid)?;
    let short_hash = commit.id().to_string()[..7].to_string();

    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;
    let mut new_files = Vec::new();

    let diff = repo.diff_tree_to_tree(Some(&parent_commit.tree()?), Some(&tree), None)?;

    diff.print(git2::DiffFormat::NameStatus, |_, _, line| {
        files_changed += 1;
        if line.origin() == '+' {
            insertions += 1;
        } else if line.origin() == '-' {
            deletions += 1;
        }
        true
    })?;

    let statuses = repo.statuses(None)?;
    for entry in statuses.iter() {
        if entry.status().contains(Status::INDEX_NEW) {
            new_files.push((
                entry.path().context("Could not get path")?.to_string(),
                entry
                    .index_to_workdir()
                    .context("Could not get index to workdir")?
                    .new_file()
                    .mode(),
            ));
        }
    }

    Ok(CommitResult {
        branch: branch_name,
        commit_hash: short_hash,
        files_changed,
        insertions,
        deletions,
        new_files,
    })
}

/// Retrieves commits between two Git references.
///
/// # Arguments
///
/// * `repo` - The git repository
/// * `from` - The starting Git reference.
/// * `to` - The ending Git reference.
/// * `callback` - A callback function to process each commit.
///
/// # Returns
///
/// A Result containing a Vec of processed commits or an error.
pub fn get_commits_between_with_callback<T, F>(
    repo: &Repository,
    from: &str,
    to: &str,
    mut callback: F,
) -> Result<Vec<T>>
where
    F: FnMut(&RecentCommit) -> Result<T>,
{
    let from_commit = repo.revparse_single(from)?.peel_to_commit()?;
    let to_commit = repo.revparse_single(to)?.peel_to_commit()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(to_commit.id())?;
    revwalk.hide(from_commit.id())?;

    revwalk
        .filter_map(std::result::Result::ok)
        .map(|id| {
            let commit = repo.find_commit(id)?;
            let recent_commit = RecentCommit {
                hash: commit.id().to_string(),
                message: commit.message().unwrap_or_default().to_string(),
                author: commit.author().name().unwrap_or_default().to_string(),
                timestamp: commit.time().seconds().to_string(),
            };
            callback(&recent_commit)
        })
        .collect()
}

/// Retrieves the files changed in a specific commit
///
/// # Arguments
///
/// * `repo` - The git repository
/// * `commit_id` - The ID of the commit to analyze.
///
/// # Returns
///
/// A Result containing a Vec of `StagedFile` objects for the commit or an error.
pub fn get_commit_files(repo: &Repository, commit_id: &str) -> Result<Vec<StagedFile>> {
    log_debug!("Getting files for commit: {}", commit_id);

    // Parse the commit ID
    let obj = repo.revparse_single(commit_id)?;
    let commit = obj.peel_to_commit()?;

    let commit_tree = commit.tree()?;
    let parent_commit = if commit.parent_count() > 0 {
        Some(commit.parent(0)?)
    } else {
        None
    };

    let parent_tree = parent_commit.map(|c| c.tree()).transpose()?;

    let mut commit_files = Vec::new();

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;

    // Get statistics for each file and convert to our StagedFile format
    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                let change_type = match delta.status() {
                    git2::Delta::Added => ChangeType::Added,
                    git2::Delta::Modified => ChangeType::Modified,
                    git2::Delta::Deleted => ChangeType::Deleted,
                    _ => return true, // Skip other types of changes
                };

                let should_exclude = crate::file_analyzers::should_exclude_file(path);

                commit_files.push(StagedFile {
                    path: path.to_string(),
                    change_type,
                    diff: String::new(), // Will be populated later
                    analysis: Vec::new(),
                    content: None,
                    content_excluded: should_exclude,
                });
            }
            true
        },
        None,
        None,
        None,
    )?;

    // Get the diff for each file
    for file in &mut commit_files {
        if file.content_excluded {
            file.diff = String::from("[Content excluded]");
            file.analysis = vec!["[Analysis excluded]".to_string()];
            continue;
        }

        let mut diff_options = git2::DiffOptions::new();
        diff_options.pathspec(&file.path);

        let file_diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            Some(&mut diff_options),
        )?;

        let mut diff_string = String::new();
        file_diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let origin = match line.origin() {
                '+' | '-' | ' ' => line.origin(),
                _ => ' ',
            };
            diff_string.push(origin);
            diff_string.push_str(&String::from_utf8_lossy(line.content()));
            true
        })?;

        if is_binary_diff(&diff_string) {
            file.diff = "[Binary file changed]".to_string();
        } else {
            file.diff = diff_string;
        }

        let analyzer = crate::file_analyzers::get_analyzer(&file.path);
        file.analysis = analyzer.analyze(&file.path, file);
    }

    log_debug!("Found {} files in commit", commit_files.len());
    Ok(commit_files)
}

/// Extract commit info without crossing async boundaries
pub fn extract_commit_info(repo: &Repository, commit_id: &str, branch: &str) -> Result<CommitInfo> {
    // Parse the commit ID
    let obj = repo.revparse_single(commit_id)?;
    let commit = obj.peel_to_commit()?;

    // Extract commit information
    let commit_author = commit.author();
    let author_name = commit_author.name().unwrap_or_default().to_string();
    let commit_message = commit.message().unwrap_or_default().to_string();
    let commit_time = commit.time().seconds().to_string();
    let commit_hash = commit.id().to_string();

    // Create the recent commit object
    let recent_commit = RecentCommit {
        hash: commit_hash,
        message: commit_message,
        author: author_name,
        timestamp: commit_time,
    };

    // Get file paths from this commit
    let file_paths = get_file_paths_for_commit(repo, commit_id)?;

    Ok(CommitInfo {
        branch: branch.to_string(),
        commit: recent_commit,
        file_paths,
    })
}

/// Gets just the file paths for a specific commit (not the full content)
pub fn get_file_paths_for_commit(repo: &Repository, commit_id: &str) -> Result<Vec<String>> {
    // Parse the commit ID
    let obj = repo.revparse_single(commit_id)?;
    let commit = obj.peel_to_commit()?;

    let commit_tree = commit.tree()?;
    let parent_commit = if commit.parent_count() > 0 {
        Some(commit.parent(0)?)
    } else {
        None
    };

    let parent_tree = parent_commit.map(|c| c.tree()).transpose()?;

    let mut file_paths = Vec::new();

    // Create diff between trees
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;

    // Extract file paths
    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                match delta.status() {
                    git2::Delta::Added | git2::Delta::Modified | git2::Delta::Deleted => {
                        file_paths.push(path.to_string());
                    }
                    _ => {} // Skip other types of changes
                }
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(file_paths)
}

/// Gets the date of a commit in YYYY-MM-DD format
///
/// # Arguments
///
/// * `repo` - The git repository
/// * `commit_ish` - A commit-ish reference (hash, tag, branch, etc.)
///
/// # Returns
///
/// A Result containing the formatted date string or an error
pub fn get_commit_date(repo: &Repository, commit_ish: &str) -> Result<String> {
    // Resolve the commit-ish to an actual commit
    let obj = repo.revparse_single(commit_ish)?;
    let commit = obj.peel_to_commit()?;

    // Get the commit time
    let time = commit.time();

    // Convert to a chrono::DateTime for easier formatting
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(time.seconds(), 0)
        .ok_or_else(|| anyhow!("Invalid timestamp"))?;

    // Format as YYYY-MM-DD
    Ok(datetime.format("%Y-%m-%d").to_string())
}
