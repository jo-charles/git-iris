//! Project documentation tool for Rig-based agents
//!
//! This tool fetches documentation files like README.md, CONTRIBUTING.md,
//! CHANGELOG.md, etc. from the project root.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::common::parameters_schema;

// Use standard tool error macro for consistency
crate::define_tool_error!(DocsError);

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
    /// Agent/AI instructions (AGENTS.md, CLAUDE.md, .github/copilot-instructions.md)
    Agents,
    /// Project context: README + agent instructions (recommended for all operations)
    Context,
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
                "Fetch project documentation for context. Types: readme, contributing, changelog, license, codeofconduct, agents (AGENTS.md/CLAUDE.md), context (readme + agent instructions - RECOMMENDED), all"
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
            DocType::Agents => vec![
                "AGENTS.md",
                "CLAUDE.md",
                ".github/copilot-instructions.md",
                ".cursor/rules",
                "CODING_GUIDELINES.md",
            ],
            DocType::Context => vec![
                "README.md",
                "AGENTS.md",
                "CLAUDE.md",
                ".github/copilot-instructions.md",
            ],
            DocType::All => vec![
                "README.md",
                "AGENTS.md",
                "CLAUDE.md",
                "CONTRIBUTING.md",
                "CHANGELOG.md",
                "CODE_OF_CONDUCT.md",
            ],
        };

        let mut output = String::new();
        let mut found_any = false;
        // Track if we found an agent instructions file (AGENTS.md often symlinks to CLAUDE.md)
        let mut found_agent_doc = false;

        for filename in files_to_check {
            // Skip CLAUDE.md if we already found AGENTS.md (avoid duplicate from symlink)
            if filename == "CLAUDE.md" && found_agent_doc {
                continue;
            }

            let path: PathBuf = current_dir.join(filename);
            if path.exists() {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        found_any = true;

                        // Mark that we found an agent doc file
                        if filename == "AGENTS.md" {
                            found_agent_doc = true;
                        }

                        output.push_str(&format!("=== {} ===\n", filename));

                        // Truncate if too long (use char boundary-safe truncation)
                        let char_count = content.chars().count();
                        if char_count > max_chars {
                            let truncated: String = content.chars().take(max_chars).collect();
                            output.push_str(&truncated);
                            output.push_str(&format!(
                                "\n\n[... truncated, {} more chars ...]\n",
                                char_count - max_chars
                            ));
                        } else {
                            output.push_str(&content);
                        }
                        output.push_str("\n\n");

                        // For single doc types, return after finding first match
                        // Context and All gather multiple files
                        if !matches!(args.doc_type, DocType::All | DocType::Context) {
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
