//! MCP server implementation for Git-Iris
//!
//! This module contains the implementation of the MCP server
//! that allows Git-Iris to be used directly from compatible tools.

use crate::config::Config as GitIrisConfig;
use crate::git::GitRepo;
use crate::log_debug;
use crate::mcp::config::{MCPServerConfig, MCPTransportType};
use crate::mcp::tools::GitIrisToolbox;

use anyhow::{Context, Result};
use rmcp::{ServerHandler, ServiceExt, model::ServerInfo};
use rmcp::tool;
use tokio::io::{stdin, stdout};
use std::sync::Arc;

/// Serve the MCP server with the provided configuration
pub async fn serve(config: MCPServerConfig) -> Result<()> {
    // Configure logging based on transport type and dev mode
    if config.dev_mode {
        // In dev mode, set up appropriate logging
        let log_path = format!("git-iris-mcp-{}.log", std::process::id());
        if let Err(e) = crate::logger::set_log_file(&log_path) {
            // For non-stdio transports, we can print this error
            if config.transport != MCPTransportType::StdIO {
                eprintln!("Failed to set up log file: {}", e);
            }
            // Continue without file logging
        }
        
        // For stdio transport, we must NEVER log to stdout
        if config.transport == MCPTransportType::StdIO {
            crate::logger::set_log_to_stdout(false);
        } else {
            crate::logger::set_log_to_stdout(true);
        }
        
        crate::logger::enable_logging();
    }
    
    log_debug!("Starting MCP server with config: {:?}", config);
    
    // Display configuration info if not using stdio transport
    if config.transport != MCPTransportType::StdIO {
        use crate::ui;
        ui::print_info(&format!("Starting Git-Iris MCP server with {:?} transport", config.transport));
        if let Some(port) = config.port {
            ui::print_info(&format!("Port: {}", port));
        }
        ui::print_info(&format!("Development mode: {}", if config.dev_mode { "Enabled" } else { "Disabled" }));
    }
    
    // Initialize GitRepo for use with tools
    let git_repo = Arc::new(GitRepo::new_from_url(None)?);
    
    // Load Git-Iris configuration
    let git_iris_config = GitIrisConfig::load()?;
    
    // Create the toolbox with necessary dependencies
    let toolbox = GitIrisToolbox::new(git_repo, git_iris_config);
    
    // Start the appropriate transport
    match config.transport {
        MCPTransportType::StdIO => serve_stdio(toolbox, config.dev_mode).await,
        MCPTransportType::SSE => {
            let port = config.port.context("Port is required for SSE transport")?;
            Err(anyhow::anyhow!("SSE transport not yet implemented"))
        }
        MCPTransportType::WebSocket => {
            let port = config.port.context("Port is required for WebSocket transport")?;
            Err(anyhow::anyhow!("WebSocket transport not yet implemented"))
        }
    }
}

/// Start the MCP server using StdIO transport
async fn serve_stdio(toolbox: GitIrisToolbox, dev_mode: bool) -> Result<()> {
    log_debug!("Starting MCP server with StdIO transport");
    
    let transport = (stdin(), stdout());
    
    let server = toolbox.serve(transport).await?;
    
    // Wait for the server to finish
    log_debug!("MCP server initialized, waiting for completion");
    let quit_reason = server.waiting().await?;
    log_debug!("MCP server finished: {:?}", quit_reason);
    
    Ok(())
} 