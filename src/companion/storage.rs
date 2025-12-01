//! Persistence layer for Iris Companion
//!
//! Stores session and branch data in ~/.iris/repos/{repo-hash}/

use super::{BranchMemory, SessionState};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Storage backend for companion data
pub struct CompanionStorage {
    /// Base directory for this repo's data
    repo_dir: PathBuf,
    /// Branches subdirectory
    branches_dir: PathBuf,
}

impl CompanionStorage {
    /// Create a new storage instance for the given repository
    pub fn new(repo_path: &Path) -> Result<Self> {
        let base_dir = Self::base_dir()?;
        let repo_hash = Self::hash_path(repo_path);
        let repo_dir = base_dir.join("repos").join(&repo_hash);
        let branches_dir = repo_dir.join("branches");

        // Ensure directories exist
        fs::create_dir_all(&branches_dir).with_context(|| {
            format!(
                "Failed to create companion directory: {}",
                branches_dir.display()
            )
        })?;

        Ok(Self {
            repo_dir,
            branches_dir,
        })
    }

    /// Get the base companion directory (~/.iris/)
    fn base_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".iris"))
    }

    /// Hash a path to create a unique identifier
    fn hash_path(path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.to_string_lossy().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Sanitize branch name for filesystem
    fn sanitize_branch_name(branch: &str) -> String {
        branch.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    }

    /// Get session file path
    fn session_path(&self) -> PathBuf {
        self.repo_dir.join("session.json")
    }

    /// Get branch memory file path
    fn branch_path(&self, branch: &str) -> PathBuf {
        let safe_name = Self::sanitize_branch_name(branch);
        self.branches_dir.join(format!("{safe_name}.json"))
    }

    /// Save session state
    pub fn save_session(&self, session: &SessionState) -> Result<()> {
        let path = self.session_path();
        Self::atomic_write(&path, session)
    }

    /// Load session state
    pub fn load_session(&self) -> Result<Option<SessionState>> {
        let path = self.session_path();
        Self::load_json(&path)
    }

    /// Save branch memory
    pub fn save_branch_memory(&self, memory: &BranchMemory) -> Result<()> {
        let path = self.branch_path(&memory.branch_name);
        Self::atomic_write(&path, memory)
    }

    /// Load branch memory
    pub fn load_branch_memory(&self, branch: &str) -> Result<Option<BranchMemory>> {
        let path = self.branch_path(branch);
        Self::load_json(&path)
    }

    /// Atomic write using temp file + rename
    fn atomic_write<T: serde::Serialize>(path: &Path, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;

        // Write to temp file first
        let temp_path = path.with_extension("json.tmp");
        let mut file = fs::File::create(&temp_path)
            .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        drop(file);

        // Atomic rename
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                path.display()
            )
        })?;

        Ok(())
    }

    /// Load JSON file if it exists
    fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let data: T = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        Ok(Some(data))
    }

    /// List all branch memories for this repo
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();

        if self.branches_dir.exists() {
            for entry in fs::read_dir(&self.branches_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json")
                    && let Some(stem) = path.file_stem()
                {
                    branches.push(stem.to_string_lossy().to_string());
                }
            }
        }

        Ok(branches)
    }

    /// Delete session data
    pub fn clear_session(&self) -> Result<()> {
        let path = self.session_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}
