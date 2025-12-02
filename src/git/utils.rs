use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::log_debug;

/// Checks if the current directory is inside a Git work tree.
///
/// # Returns
///
/// A Result containing a boolean indicating if inside a work tree or an error.
pub fn is_inside_work_tree() -> Result<bool> {
    let status = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(exit) => Ok(exit.success()),
        Err(_) => Ok(false),
    }
}

/// Determines if the given diff represents a binary file.
pub fn is_binary_diff(diff: &str) -> bool {
    diff.contains("Binary files")
        || diff.contains("GIT binary patch")
        || diff.contains("[Binary file changed]")
}

/// Executes a git command and returns the output as a string
///
/// # Arguments
///
/// * `args` - The arguments to pass to git
///
/// # Returns
///
/// A Result containing the output as a String or an error.
pub fn run_git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout =
        String::from_utf8(output.stdout).context("Invalid UTF-8 output from git command")?;

    Ok(stdout.trim().to_string())
}

/// Checks if a file should be excluded from analysis.
///
/// Excludes common directories and files that don't contribute meaningfully
/// to commit context (build artifacts, lock files, IDE configs, etc.)
pub fn should_exclude_file(path: &str) -> bool {
    log_debug!("Checking if file should be excluded: {}", path);
    let exclude_patterns = vec![
        (String::from(r"(^|/)\.git(/|$)"), false), // Only exclude .git directory, not .github
        (String::from(r"(^|/)\.svn(/|$)"), false),
        (String::from(r"(^|/)\.hg(/|$)"), false),
        (String::from(r"(^|/)\.DS_Store$"), false),
        (String::from(r"(^|/)node_modules(/|$)"), false),
        (String::from(r"(^|/)target(/|$)"), false),
        (String::from(r"(^|/)build(/|$)"), false),
        (String::from(r"(^|/)dist(/|$)"), false),
        (String::from(r"(^|/)\.vscode(/|$)"), false),
        (String::from(r"(^|/)\.idea(/|$)"), false),
        (String::from(r"(^|/)\.vs(/|$)"), false),
        (String::from(r"package-lock\.json$"), true),
        (String::from(r"\.lock$"), true),
        (String::from(r"\.log$"), true),
        (String::from(r"\.tmp$"), true),
        (String::from(r"\.temp$"), true),
        (String::from(r"\.swp$"), true),
        (String::from(r"\.min\.js$"), true),
    ];

    let path = Path::new(path);

    for (pattern, is_extension) in exclude_patterns {
        let re = match Regex::new(&pattern) {
            Ok(re) => re,
            Err(e) => {
                log_debug!("Failed to compile regex '{}': {}", pattern, e);
                continue;
            }
        };

        if is_extension {
            if let Some(file_name) = path.file_name()
                && let Some(file_name_str) = file_name.to_str()
                && re.is_match(file_name_str)
            {
                log_debug!("File excluded: {}", path.display());
                return true;
            }
        } else if let Some(path_str) = path.to_str()
            && re.is_match(path_str)
        {
            log_debug!("File excluded: {}", path.display());
            return true;
        }
    }
    log_debug!("File not excluded: {}", path.display());
    false
}
