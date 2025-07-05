//! File analysis tool
//!
//! This tool provides Iris with the ability to analyze files for content,
//! structure, and language-specific information.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AgentTool;
use crate::agents::core::AgentContext;
use crate::log_debug;

/// File reading and analysis tool
pub struct FileAnalyzerTool {
    id: String,
}

impl Default for FileAnalyzerTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FileAnalyzerTool {
    pub fn new() -> Self {
        Self {
            id: "file_analyzer".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct FileAnalyzerArgs {
    pub path: String,
    pub analysis_type: Option<String>,
    pub include_content: Option<bool>,
}

#[async_trait]
impl AgentTool for FileAnalyzerTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "File Analyzer"
    }

    fn description(&self) -> &'static str {
        "Analyze files for content, structure, and language-specific information"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "file_analysis".to_string(),
            "code_review".to_string(),
            "commit".to_string(),
        ]
    }

    async fn execute(
        &self,
        _context: &AgentContext,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        log_debug!("🔍 FileAnalyzerTool executing with params: {:?}", params);

        let args: FileAnalyzerArgs = serde_json::from_value(serde_json::Value::Object(
            params.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ))?;

        log_debug!(
            "📄 FileAnalyzerTool: Analyzing file: {} (type: {:?}, include_content: {})",
            args.path,
            args.analysis_type,
            args.include_content.unwrap_or(false)
        );

        let analyzer = crate::file_analyzers::get_analyzer(&args.path);
        let file_type = analyzer.get_file_type();
        let include_content = args.include_content.unwrap_or(false);

        log_debug!("🔬 FileAnalyzerTool: File type detected: {}", file_type);

        let mut result = serde_json::json!({
            "path": args.path,
            "file_type": file_type,
        });

        if include_content {
            log_debug!("📖 FileAnalyzerTool: Reading file content for analysis");
            if let Ok(content) = tokio::fs::read_to_string(&args.path).await {
                let line_count = content.lines().count();
                result["content"] = serde_json::Value::String(content.clone());
                result["lines"] = serde_json::Value::Number(serde_json::Number::from(line_count));

                log_debug!(
                    "✅ FileAnalyzerTool: Content read - {} lines, {} chars",
                    line_count,
                    content.len()
                );
            } else {
                log_debug!(
                    "⚠️  FileAnalyzerTool: Failed to read file content for: {}",
                    args.path
                );
            }
        }

        log_debug!("✅ FileAnalyzerTool: Analysis complete for {}", args.path);
        Ok(result)
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to analyze"
                },
                "analysis_type": {
                    "type": "string",
                    "enum": ["basic", "detailed", "security"],
                    "description": "Type of analysis to perform"
                },
                "include_content": {
                    "type": "boolean",
                    "description": "Whether to include file content in the response"
                }
            },
            "required": ["path"]
        })
    }

    fn as_rig_tool_placeholder(&self) -> String {
        format!("FileAnalyzerTool: {}", self.name())
    }
}
