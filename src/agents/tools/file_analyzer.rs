//! File analyzer tool
//!
//! This tool provides Iris with the ability to analyze file contents,
//! extract metadata, identify patterns, and understand code structure.

use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

use crate::context::{ProjectMetadata, StagedFile};
use crate::file_analyzers::get_analyzer;

#[derive(Debug, thiserror::Error)]
#[error("File analyzer error: {0}")]
pub struct FileAnalyzerError(String);

impl From<anyhow::Error> for FileAnalyzerError {
    fn from(err: anyhow::Error) -> Self {
        FileAnalyzerError(err.to_string())
    }
}

impl From<std::io::Error> for FileAnalyzerError {
    fn from(err: std::io::Error) -> Self {
        FileAnalyzerError(err.to_string())
    }
}

/// File analyzer tool for understanding code structure and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalyzer;

impl Default for FileAnalyzer {
    fn default() -> Self {
        Self
    }
}

impl FileAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a single file using the appropriate analyzer
    async fn analyze_file(&self, file_path: &str, repo_path: &Path) -> Result<FileAnalysis> {
        let analyzer = get_analyzer(file_path);

        // Read file content
        let full_path = if Path::new(file_path).is_absolute() {
            Path::new(file_path).to_path_buf()
        } else {
            repo_path.join(file_path)
        };

        let content = if full_path.exists() && full_path.is_file() {
            tokio::fs::read_to_string(&full_path)
                .await
                .unwrap_or_else(|_| String::new())
        } else {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        };

        // Create a mock StagedFile for analysis (since we're not in a commit context)
        let staged_file = StagedFile {
            path: file_path.to_string(),
            change_type: crate::context::ChangeType::Modified,
            diff: content.clone(),
            analysis: vec![],
            content: Some(content.clone()),
            content_excluded: false,
        };

        // Perform analysis
        let analysis_results = analyzer.analyze(file_path, &staged_file);
        let file_type = analyzer.get_file_type();
        let metadata = analyzer.extract_metadata(file_path, &content);

        // Calculate basic metrics
        let lines_of_code = content.lines().count();
        let complexity_score = Self::calculate_complexity(&content, file_type);

        Ok(FileAnalysis {
            file_path: file_path.to_string(),
            file_type: file_type.to_string(),
            summary: if analysis_results.is_empty() {
                format!("Analyzed {file_type} file with {lines_of_code} lines")
            } else {
                analysis_results.join("; ")
            },
            key_components: Self::extract_key_components(&content, file_type),
            dependencies: metadata.dependencies.clone(),
            complexity_score,
            lines_of_code,
            security_issues: Self::check_security_issues(&content, file_type),
            performance_notes: Self::check_performance_issues(&content, file_type),
            architectural_insights: Self::extract_architectural_insights(&metadata),
            extracted_metadata: Self::metadata_to_json(&metadata),
        })
    }

    /// Calculate basic complexity score
    fn calculate_complexity(content: &str, file_type: &str) -> usize {
        let mut complexity = 0;

        // Count various complexity indicators based on file type
        match file_type {
            "Rust source file" => {
                complexity += content.matches("fn ").count() * 2;
                complexity += content.matches("if ").count();
                complexity += content.matches("match ").count() * 2;
                complexity += content.matches("loop ").count();
                complexity += content.matches("while ").count();
            }
            "JavaScript source file" | "TypeScript source file" => {
                complexity += content.matches("function ").count() * 2;
                complexity += content.matches("if (").count();
                complexity += content.matches("switch (").count() * 2;
                complexity += content.matches("for (").count();
                complexity += content.matches("while (").count();
            }
            "Python source file" => {
                complexity += content.matches("def ").count() * 2;
                complexity += content.matches("if ").count();
                complexity += content.matches("elif ").count();
                complexity += content.matches("for ").count();
                complexity += content.matches("while ").count();
            }
            _ => {
                // Generic complexity based on line count and nesting
                complexity = content.lines().count() / 10;
            }
        }

        complexity
    }

    /// Extract key components from content
    fn extract_key_components(content: &str, file_type: &str) -> Vec<String> {
        let mut components = Vec::new();

        match file_type {
            "Rust source file" => {
                // Extract function names
                for line in content.lines() {
                    if let Some(fn_match) = line.find("fn ") {
                        if let Some(name_start) = line[fn_match + 3..].find(char::is_alphabetic) {
                            let name_part = &line[fn_match + 3 + name_start..];
                            if let Some(name_end) =
                                name_part.find(|c: char| c == '(' || c.is_whitespace())
                            {
                                components.push(format!("fn {}", &name_part[..name_end]));
                            }
                        }
                    }
                }
            }
            "JavaScript source file" | "TypeScript source file" => {
                // Extract function names
                for line in content.lines() {
                    if line.contains("function ") {
                        components.push("function".to_string());
                    }
                    if line.contains("class ") {
                        components.push("class".to_string());
                    }
                }
            }
            _ => {
                // Generic extraction based on common patterns
                if content.contains("main") {
                    components.push("main".to_string());
                }
            }
        }

        components
    }

    /// Check for basic security issues
    fn check_security_issues(content: &str, _file_type: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Basic security pattern detection
        if content.to_lowercase().contains("password") && content.contains('=') {
            issues.push("Potential hardcoded password detected".to_string());
        }
        if content.contains("TODO") || content.contains("FIXME") {
            issues.push("TODO/FIXME comments indicate incomplete security measures".to_string());
        }
        if content.to_lowercase().contains("unsafe") {
            issues.push("Unsafe code detected".to_string());
        }

        issues
    }

    /// Check for basic performance issues
    fn check_performance_issues(content: &str, file_type: &str) -> Vec<String> {
        let mut issues = Vec::new();

        match file_type {
            "Rust source file" => {
                if content.contains(".clone()") {
                    issues.push("Frequent cloning detected - consider borrowing".to_string());
                }
                if content.contains("Vec::new()") && content.contains("push") {
                    issues.push("Vector growth pattern - consider with_capacity".to_string());
                }
            }
            "JavaScript source file" | "TypeScript source file" => {
                if content.contains("console.log") {
                    issues.push(
                        "Debug logging statements should be removed in production".to_string(),
                    );
                }
            }
            _ => {}
        }

        issues
    }

    /// Extract architectural insights from metadata
    fn extract_architectural_insights(metadata: &ProjectMetadata) -> Vec<String> {
        let mut insights = Vec::new();

        if let Some(framework) = &metadata.framework {
            insights.push(format!("Uses {framework} framework"));
        }
        if let Some(build_system) = &metadata.build_system {
            insights.push(format!("Build system: {build_system}"));
        }
        if !metadata.dependencies.is_empty() {
            insights.push(format!("Has {} dependencies", metadata.dependencies.len()));
        }

        insights
    }

    /// Convert metadata to JSON
    fn metadata_to_json(metadata: &ProjectMetadata) -> HashMap<String, serde_json::Value> {
        let mut json_map = HashMap::new();

        if let Some(ref language) = metadata.language {
            json_map.insert(
                "language".to_string(),
                serde_json::Value::String(language.clone()),
            );
        }
        if let Some(ref version) = metadata.version {
            json_map.insert(
                "version".to_string(),
                serde_json::Value::String(version.clone()),
            );
        }
        if let Some(ref framework) = metadata.framework {
            json_map.insert(
                "framework".to_string(),
                serde_json::Value::String(framework.clone()),
            );
        }
        if let Some(ref build_system) = metadata.build_system {
            json_map.insert(
                "build_system".to_string(),
                serde_json::Value::String(build_system.clone()),
            );
        }

        json_map.insert(
            "dependencies".to_string(),
            serde_json::Value::Array(
                metadata
                    .dependencies
                    .iter()
                    .map(|d| serde_json::Value::String(d.clone()))
                    .collect(),
            ),
        );

        json_map
    }

    /// Analyze multiple files in batch
    async fn analyze_files_batch(
        &self,
        file_paths: &[String],
        repo_path: &Path,
    ) -> Result<Vec<FileAnalysis>> {
        let mut results = Vec::new();

        for file_path_str in file_paths {
            match self.analyze_file(file_path_str, repo_path).await {
                Ok(analysis) => results.push(analysis),
                Err(e) => {
                    // Continue with other files on error
                    eprintln!("Failed to analyze {file_path_str}: {e}");
                    results.push(FileAnalysis {
                        file_path: file_path_str.clone(),
                        file_type: "unknown".to_string(),
                        summary: format!("Analysis failed: {e}"),
                        key_components: vec![],
                        dependencies: vec![],
                        complexity_score: 0,
                        lines_of_code: 0,
                        security_issues: vec![],
                        performance_notes: vec![],
                        architectural_insights: vec![],
                        extracted_metadata: HashMap::new(),
                    });
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file_path: String,
    pub file_type: String,
    pub summary: String,
    pub key_components: Vec<String>,
    pub dependencies: Vec<String>,
    pub complexity_score: usize,
    pub lines_of_code: usize,
    pub security_issues: Vec<String>,
    pub performance_notes: Vec<String>,
    pub architectural_insights: Vec<String>,
    pub extracted_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct FileAnalyzerArgs {
    pub file_paths: Vec<String>,
    pub analysis_depth: Option<String>, // "basic", "detailed", "comprehensive"
    pub include_metrics: Option<bool>,
    pub include_dependencies: Option<bool>,
}

impl Tool for FileAnalyzer {
    const NAME: &'static str = "file_analyzer";
    type Error = FileAnalyzerError;
    type Args = FileAnalyzerArgs;
    type Output = String;

    async fn definition(&self, _: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "file_analyzer",
            "description": "Analyze file contents, extract metadata, identify patterns, and understand code structure. Provides complexity metrics, security issues, and architectural insights.",
            "parameters": {
                "type": "object",
                "properties": {
                    "file_paths": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of file paths to analyze (relative to repository root)",
                        "minItems": 1,
                        "maxItems": 50
                    },
                    "analysis_depth": {
                        "type": "string",
                        "enum": ["basic", "detailed", "comprehensive"],
                        "description": "Depth of analysis to perform (default: detailed)"
                    },
                    "include_metrics": {
                        "type": "boolean",
                        "description": "Include complexity and performance metrics (default: true)"
                    },
                    "include_dependencies": {
                        "type": "boolean",
                        "description": "Include dependency analysis (default: true)"
                    }
                },
                "required": ["file_paths"]
            }
        }))
        .unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let current_dir = std::env::current_dir().map_err(FileAnalyzerError::from)?;

        let analyses = self
            .analyze_files_batch(&args.file_paths, &current_dir)
            .await
            .map_err(FileAnalyzerError::from)?;

        // Calculate aggregate metrics
        let total_loc: usize = analyses.iter().map(|a| a.lines_of_code).sum();
        let avg_complexity: f64 = if analyses.is_empty() {
            0.0
        } else {
            let sum: usize = analyses.iter().map(|a| a.complexity_score).sum();
            f64::from(u32::try_from(sum).unwrap_or(u32::MAX))
                / f64::from(u32::try_from(analyses.len()).unwrap_or(u32::MAX))
        };
        let total_security_issues: usize = analyses.iter().map(|a| a.security_issues.len()).sum();

        let result = serde_json::json!({
            "file_analyses": analyses,
            "summary": {
                "total_files": analyses.len(),
                "total_lines_of_code": total_loc,
                "average_complexity": avg_complexity,
                "total_security_issues": total_security_issues,
                "file_types": analyses.iter().map(|a| &a.file_type).collect::<std::collections::HashSet<_>>(),
            },
            "analysis_depth": args.analysis_depth.unwrap_or_else(|| "detailed".to_string()),
            "include_metrics": args.include_metrics.unwrap_or(true),
            "include_dependencies": args.include_dependencies.unwrap_or(true),
        });

        serde_json::to_string_pretty(&result).map_err(|e| FileAnalyzerError(e.to_string()))
    }
}
