//! MCP tools module for Git-Iris
//!
//! This module contains the implementation of the MCP tools
//! that expose Git-Iris functionality to MCP clients.

pub mod changelog;
pub mod codereview;
pub mod commit;
pub mod releasenotes;
pub mod utils;

use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;
use crate::mcp::tools::utils::GitIrisTool;

use rmcp::Error;
use rmcp::RoleServer;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam,
    ServerCapabilities, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ServerHandler, model::ServerInfo};

use serde_json::{Map, Value};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

// Re-export all tools for easy importing
pub use self::changelog::ChangelogTool;
pub use self::codereview::CodeReviewTool;
pub use self::commit::CommitTool;
pub use self::releasenotes::ReleaseNotesTool;

// Define our tools for the Git-Iris toolbox
#[derive(Debug)]
pub enum GitIrisTools {
    ReleaseNotesTool(ReleaseNotesTool),
    ChangelogTool(ChangelogTool),
    CommitTool(CommitTool),
    CodeReviewTool(CodeReviewTool),
}

impl GitIrisTools {
    /// Get all tools available in Git-Iris
    pub fn get_tools() -> Vec<Tool> {
        vec![
            ReleaseNotesTool::get_tool_definition(),
            ChangelogTool::get_tool_definition(),
            CommitTool::get_tool_definition(),
            CodeReviewTool::get_tool_definition(),
        ]
    }

    /// Try to convert a parameter map into a `GitIrisTools` enum
    pub fn try_from(params: Map<String, Value>) -> Result<Self, Error> {
        // Check the tool name and convert to the appropriate variant
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_params("Tool name not specified", None))?;

        match tool_name {
            "git_iris_release_notes" => {
                // Convert params to ReleaseNotesTool
                let tool: ReleaseNotesTool = serde_json::from_value(Value::Object(params))
                    .map_err(|e| Error::invalid_params(format!("Invalid parameters: {e}"), None))?;
                Ok(GitIrisTools::ReleaseNotesTool(tool))
            }
            "git_iris_changelog" => {
                // Convert params to ChangelogTool
                let tool: ChangelogTool = serde_json::from_value(Value::Object(params))
                    .map_err(|e| Error::invalid_params(format!("Invalid parameters: {e}"), None))?;
                Ok(GitIrisTools::ChangelogTool(tool))
            }
            "git_iris_commit" => {
                // Convert params to CommitTool
                let tool: CommitTool = serde_json::from_value(Value::Object(params))
                    .map_err(|e| Error::invalid_params(format!("Invalid parameters: {e}"), None))?;
                Ok(GitIrisTools::CommitTool(tool))
            }
            "git_iris_code_review" => {
                // Convert params to CodeReviewTool
                let tool: CodeReviewTool = serde_json::from_value(Value::Object(params))
                    .map_err(|e| Error::invalid_params(format!("Invalid parameters: {e}"), None))?;
                Ok(GitIrisTools::CodeReviewTool(tool))
            }
            _ => Err(Error::invalid_params(
                format!("Unknown tool: {tool_name}"),
                None,
            )),
        }
    }
}

/// Common error handling for Git-Iris tools
pub fn handle_tool_error(e: &anyhow::Error) -> Error {
    Error::invalid_params(format!("Tool execution failed: {e}"), None)
}

/// The main handler for Git-Iris, providing all MCP tools
#[derive(Clone)]
pub struct GitIrisHandler {
    /// Git repository instance
    pub git_repo: Arc<GitRepo>,
    /// Git-Iris configuration
    pub config: GitIrisConfig,
    /// Workspace roots registered by the client
    pub workspace_roots: Arc<Mutex<Vec<PathBuf>>>,
}

impl GitIrisHandler {
    /// Create a new Git-Iris handler with the provided dependencies
    pub fn new(git_repo: Arc<GitRepo>, config: GitIrisConfig) -> Self {
        Self {
            git_repo,
            config,
            workspace_roots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the current workspace root, if available
    pub fn get_workspace_root(&self) -> Option<PathBuf> {
        let roots = self
            .workspace_roots
            .lock()
            .expect("Failed to lock workspace roots mutex");
        // Use the first workspace root if available
        roots.first().cloned()
    }
}

impl ServerHandler for GitIrisHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Git-Iris is an AI-powered Git workflow assistant. You can use it to generate commit messages, review code, create changelogs and release notes.".to_string()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    // Handle notification when client workspace roots change
    fn on_roots_list_changed(&self) -> impl Future<Output = ()> + Send + '_ {
        log_debug!("Client workspace roots changed");
        std::future::ready(())
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, Error> {
        // Use our custom method to get all tools
        let tools = GitIrisTools::get_tools();

        Ok(ListToolsResult {
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, Error> {
        // Get the arguments as a Map
        let args = match &request.arguments {
            Some(args) => args.clone(),
            None => {
                return Err(Error::invalid_params(
                    String::from("Missing arguments"),
                    None,
                ));
            }
        };

        // Add the tool name to the parameters
        let mut params = args.clone();
        params.insert("name".to_string(), Value::String(request.name.to_string()));

        // Try to convert to our GitIrisTools enum
        let tool_params = GitIrisTools::try_from(params)?;

        // Make a clone of the repository path before executing the tool
        // This prevents git2 objects from crossing async boundaries
        let git_repo_path = self.git_repo.repo_path().clone();

        // Clone config to avoid sharing it across async boundaries
        let config = self.config.clone();

        // Create a new git repo instance - handle any errors here before async code
        let git_repo = match GitRepo::new(&git_repo_path) {
            Ok(repo) => Arc::new(repo),
            Err(e) => return Err(handle_tool_error(&e)),
        };

        // Use the GitIrisTool trait to execute any tool without matching on specific types
        match tool_params {
            GitIrisTools::ReleaseNotesTool(tool) => tool
                .execute(git_repo.clone(), config.clone())
                .await
                .map_err(|e| handle_tool_error(&e)),
            GitIrisTools::ChangelogTool(tool) => tool
                .execute(git_repo.clone(), config.clone())
                .await
                .map_err(|e| handle_tool_error(&e)),
            GitIrisTools::CommitTool(tool) => tool
                .execute(git_repo.clone(), config.clone())
                .await
                .map_err(|e| handle_tool_error(&e)),
            GitIrisTools::CodeReviewTool(tool) => tool
                .execute(git_repo, config)
                .await
                .map_err(|e| handle_tool_error(&e)),
        }
    }
}
