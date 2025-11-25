//! Project metadata tool for Rig-based agents
//!
//! This tool extracts project metadata like language, framework, dependencies,
//! build system, and test framework from the project files.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

use crate::context::ProjectMetadata;
use crate::git::{GitRepo, extract_project_metadata};

#[derive(Debug, thiserror::Error)]
#[error("Project metadata error: {0}")]
pub struct MetadataError(String);

impl From<anyhow::Error> for MetadataError {
    fn from(err: anyhow::Error) -> Self {
        MetadataError(err.to_string())
    }
}

impl From<std::io::Error> for MetadataError {
    fn from(err: std::io::Error) -> Self {
        MetadataError(err.to_string())
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

/// Tool for extracting project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadataTool;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProjectMetadataArgs {
    /// Whether to analyze staged files only (default: true) or all project files
    #[serde(default = "default_staged_only")]
    pub staged_only: bool,
}

fn default_staged_only() -> bool {
    true
}

impl Tool for ProjectMetadataTool {
    const NAME: &'static str = "project_metadata";
    type Error = MetadataError;
    type Args = ProjectMetadataArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "project_metadata".to_string(),
            description:
                "Get project metadata including language, framework, dependencies, and build system"
                    .to_string(),
            parameters: parameters_schema::<ProjectMetadataArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(MetadataError::from)?;
        let repo = GitRepo::new(&current_dir).map_err(MetadataError::from)?;

        // Get files to analyze
        let files = if args.staged_only {
            let files_info = repo
                .extract_files_info(false)
                .map_err(MetadataError::from)?;
            files_info.file_paths
        } else {
            // Get all tracked files in the repo
            get_tracked_files(&current_dir)?
        };

        if files.is_empty() {
            return Ok(
                "No files to analyze. Stage some files or use staged_only=false.".to_string(),
            );
        }

        // Extract metadata using the existing infrastructure
        let metadata = extract_project_metadata(&files, 10)
            .await
            .map_err(MetadataError::from)?;

        Ok(format_metadata(&metadata))
    }
}

/// Get all tracked files in the repository
fn get_tracked_files(repo_path: &Path) -> Result<Vec<String>, MetadataError> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| MetadataError(format!("Failed to run git ls-files: {e}")))?;

    if !output.status.success() {
        return Err(MetadataError("git ls-files failed".to_string()));
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(ToString::to_string)
        .collect();

    Ok(files)
}

/// Format metadata for output
fn format_metadata(metadata: &ProjectMetadata) -> String {
    let mut output = String::new();
    output.push_str("=== PROJECT METADATA ===\n\n");

    if let Some(lang) = &metadata.language {
        output.push_str(&format!("Language: {lang}\n"));
    }

    if let Some(framework) = &metadata.framework {
        output.push_str(&format!("Framework: {framework}\n"));
    }

    if let Some(build_system) = &metadata.build_system {
        output.push_str(&format!("Build System: {build_system}\n"));
    }

    if let Some(test_framework) = &metadata.test_framework {
        output.push_str(&format!("Test Framework: {test_framework}\n"));
    }

    if let Some(version) = &metadata.version {
        output.push_str(&format!("Version: {version}\n"));
    }

    if !metadata.dependencies.is_empty() {
        output.push_str(&format!(
            "\nDependencies ({}):\n",
            metadata.dependencies.len()
        ));
        for dep in metadata.dependencies.iter().take(20) {
            output.push_str(&format!("  - {dep}\n"));
        }
        if metadata.dependencies.len() > 20 {
            output.push_str(&format!(
                "  ... and {} more\n",
                metadata.dependencies.len() - 20
            ));
        }
    }

    if !metadata.plugins.is_empty() {
        output.push_str(&format!("\nPlugins ({}):\n", metadata.plugins.len()));
        for plugin in metadata.plugins.iter().take(10) {
            output.push_str(&format!("  - {plugin}\n"));
        }
        if metadata.plugins.len() > 10 {
            output.push_str(&format!("  ... and {} more\n", metadata.plugins.len() - 10));
        }
    }

    output
}
