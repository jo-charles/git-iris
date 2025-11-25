//! Project documentation tool for Rig-based agents
//!
//! This tool fetches documentation files like README.md, CONTRIBUTING.md,
//! CHANGELOG.md, etc. from the project root.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("Docs error: {0}")]
pub struct DocsError(String);

impl From<std::io::Error> for DocsError {
    fn from(err: std::io::Error) -> Self {
        DocsError(err.to_string())
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

/// Tool for fetching project documentation files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocs;

/// Type of documentation to fetch
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum DocType {
    /// README file (README.md, README.rst, README.txt)
    #[default]
    Readme,
    /// Contributing guidelines (CONTRIBUTING.md)
    Contributing,
    /// Changelog (CHANGELOG.md, HISTORY.md)
    Changelog,
    /// License file (LICENSE, LICENSE.md)
    License,
    /// Code of conduct (`CODE_OF_CONDUCT.md`)
    CodeOfConduct,
    /// All documentation files
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProjectDocsArgs {
    /// Type of documentation to fetch
    #[serde(default)]
    pub doc_type: DocType,
    /// Maximum characters to return (default: 5000, max: 20000)
    #[serde(default = "default_max_chars")]
    pub max_chars: usize,
}

fn default_max_chars() -> usize {
    5000
}

impl Tool for ProjectDocs {
    const NAME: &'static str = "project_docs";
    type Error = DocsError;
    type Args = ProjectDocsArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: "project_docs".to_string(),
            description:
                "Fetch project documentation (README, CONTRIBUTING, CHANGELOG, etc.) for context"
                    .to_string(),
            parameters: parameters_schema::<ProjectDocsArgs>(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(DocsError::from)?;
        let max_chars = args.max_chars.min(20000);

        let files_to_check = match args.doc_type {
            DocType::Readme => vec![
                "README.md",
                "README.rst",
                "README.txt",
                "README",
                "readme.md",
            ],
            DocType::Contributing => vec!["CONTRIBUTING.md", "CONTRIBUTING", "contributing.md"],
            DocType::Changelog => vec![
                "CHANGELOG.md",
                "CHANGELOG",
                "HISTORY.md",
                "CHANGES.md",
                "changelog.md",
            ],
            DocType::License => vec!["LICENSE", "LICENSE.md", "LICENSE.txt", "license"],
            DocType::CodeOfConduct => vec!["CODE_OF_CONDUCT.md", "code_of_conduct.md"],
            DocType::All => vec![
                "README.md",
                "CONTRIBUTING.md",
                "CHANGELOG.md",
                "CODE_OF_CONDUCT.md",
            ],
        };

        let mut output = String::new();
        let mut found_any = false;

        for filename in files_to_check {
            let path: PathBuf = current_dir.join(filename);
            if path.exists() {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        found_any = true;
                        output.push_str(&format!("=== {} ===\n", filename));

                        // Truncate if too long
                        if content.len() > max_chars {
                            output.push_str(&content[..max_chars]);
                            output.push_str(&format!(
                                "\n\n[... truncated, {} more chars ...]\n",
                                content.len() - max_chars
                            ));
                        } else {
                            output.push_str(&content);
                        }
                        output.push_str("\n\n");

                        // For single doc types, return after finding first match
                        if !matches!(args.doc_type, DocType::All) {
                            break;
                        }
                    }
                    Err(e) => {
                        output.push_str(&format!("Error reading {}: {}\n", filename, e));
                    }
                }
            }
        }

        if !found_any {
            output = format!(
                "No {:?} documentation found in project root.",
                args.doc_type
            );
        }

        Ok(output)
    }
}
