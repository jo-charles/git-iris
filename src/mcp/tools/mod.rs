//! MCP tools module for Git-Iris
//!
//! This module contains the implementation of the MCP tools
//! that expose Git-Iris functionality to MCP clients.

pub mod releasenotes;

use crate::changes::ReleaseNotesGenerator;
use crate::common::DetailLevel;
use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;

use rmcp::{ServerHandler, model::ServerInfo};
use rmcp::service::RequestContext;
use rmcp::RoleServer;
use rmcp::Error;
use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam, Tool, Content, RawTextContent, RawContent, Annotated, ServerCapabilities};

use serde_json::Value;
use std::sync::Arc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Mutex;
use std::future::Future;

/// The main toolbox for Git-Iris, providing all MCP tools
#[derive(Clone)]
pub struct GitIrisToolbox {
    /// Git repository instance
    pub git_repo: Arc<GitRepo>,
    /// Git-Iris configuration
    pub config: GitIrisConfig,
    /// Workspace roots registered by the client
    pub workspace_roots: Arc<Mutex<Vec<PathBuf>>>,
}

impl GitIrisToolbox {
    /// Create a new Git-Iris toolbox with the provided dependencies
    pub fn new(git_repo: Arc<GitRepo>, config: GitIrisConfig) -> Self {
        Self { 
            git_repo, 
            config,
            workspace_roots: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get the current workspace root, if available
    fn get_workspace_root(&self) -> Option<PathBuf> {
        let roots = self.workspace_roots.lock().unwrap();
        // Use the first workspace root if available
        roots.first().cloned()
    }
    
    /// Generate release notes between two Git references
    async fn git_iris_release_notes(
        &self,
        request: releasenotes::ReleaseNotesRequest,
    ) -> anyhow::Result<String> {
        log_debug!("Generating release notes with request: {:?}", request);
        
        // Parse detail level with robust empty string handling
        let detail_level = if request.detail_level.trim().is_empty() {
            log_debug!("Empty detail level, using Standard");
            DetailLevel::Standard
        } else {
            match request.detail_level.trim().to_lowercase().as_str() {
                "minimal" => DetailLevel::Minimal,
                "detailed" => DetailLevel::Detailed,
                "standard" => DetailLevel::Standard,
                other => {
                    log_debug!("Unknown detail level '{}', defaulting to Standard", other);
                    DetailLevel::Standard
                }
            }
        };
        
        // Set up config with custom instructions if provided and not empty
        let mut config = self.config.clone();
        if !request.custom_instructions.trim().is_empty() {
            config.set_temp_instructions(Some(request.custom_instructions));
        }
        
        // Default to HEAD if to is empty
        let to = if request.to.trim().is_empty() {
            "HEAD".to_string()
        } else {
            request.to
        };
        
        // Generate the release notes using the publicly re-exported generator
        ReleaseNotesGenerator::generate(
            self.git_repo.clone(),
            &request.from,
            &to,
            &config,
            detail_level,
        )
        .await
    }
}

impl ServerHandler for GitIrisToolbox {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Git-Iris is an AI-powered Git workflow assistant. You can use it to generate commit messages, review code, create changelogs and release notes.".to_string()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                // Note: We can't enable roots directly in the server capabilities
                // as that's a client-side feature. The client provides workspace info to us.
                .build(),
            ..Default::default()
        }
    }
    
    // Handle notification when client workspace roots change
    fn on_roots_list_changed(&self) -> impl Future<Output = ()> + Send + '_ {
        log_debug!("Client workspace roots changed");
        // We could retrieve information about the workspace from the client
        // and update our git repository path here if needed
        std::future::ready(())
    }
    
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, Error> {
        // Create tool for release notes
        let tool = Tool {
            name: Cow::<'static, str>::Borrowed("git_iris_release_notes"),
            description: Some(Cow::<'static, str>::Borrowed("Generate comprehensive release notes between two Git references")),
            input_schema: rmcp::handler::server::tool::cached_schema_for_type::<releasenotes::ReleaseNotesRequest>().into(),
            annotations: None,
        };
        
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![tool],
        })
    }
    
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, Error> {
        // Compare the name field directly
        if request.name == "git_iris_release_notes" {
            let params: releasenotes::ReleaseNotesRequest = match &request.arguments {
                Some(args) => serde_json::from_value(Value::Object(args.clone()))
                    .map_err(|e| Error::invalid_params(format!("Invalid parameters: {}", e), None))?,
                None => return Err(Error::invalid_params(String::from("Missing arguments"), None)),
            };
            
            match self.git_iris_release_notes(params).await {
                Ok(content) => {
                    // Create content
                    let raw_text = RawTextContent { text: content };
                    let raw_content = RawContent::Text(raw_text);
                    let annotated_content = Annotated { 
                        raw: raw_content,
                        annotations: None,
                    };
                    let content_item = Content::from(annotated_content);
                    
                    Ok(CallToolResult {
                        content: vec![content_item],
                        is_error: None,
                    })
                },
                Err(e) => {
                    // Create error content
                    let error_msg = format!("Error generating release notes: {}", e);
                    let error_text = RawTextContent { text: error_msg };
                    let error_raw = RawContent::Text(error_text);
                    let annotated_error = Annotated {
                        raw: error_raw,
                        annotations: None,
                    };
                    let error_content = Content::from(annotated_error);
                    
                    Ok(CallToolResult {
                        content: vec![error_content],
                        is_error: Some(true),
                    })
                }
            }
        } else {
            Err(Error::invalid_params(format!("Unknown tool: {}", request.name), None))
        }
    }
}