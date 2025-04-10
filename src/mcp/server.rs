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
use rmcp::tool;
use rmcp::transport::sse_server::SseServer;
use rmcp::{ServerHandler, ServiceExt, model::ServerInfo};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{stdin, stdout};

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
        ui::print_info(&format!(
            "Starting Git-Iris MCP server with {:?} transport",
            config.transport
        ));
        if let Some(port) = config.port {
            ui::print_info(&format!("Port: {}", port));
        }
        if let Some(addr) = &config.listen_address {
            ui::print_info(&format!("Listening on: {}", addr));
        }
        ui::print_info(&format!(
            "Development mode: {}",
            if config.dev_mode {
                "Enabled"
            } else {
                "Disabled"
            }
        ));
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
            // Get socket address for the server
            let socket_addr = get_socket_addr(&config)?;
            serve_sse(toolbox, socket_addr).await
        }
        MCPTransportType::WebSocket => {
            let port = config
                .port
                .context("Port is required for WebSocket transport")?;
            Err(anyhow::anyhow!("WebSocket transport not yet implemented"))
        }
    }
}

/// Start the MCP server using StdIO transport
async fn serve_stdio(toolbox: GitIrisToolbox, _dev_mode: bool) -> Result<()> {
    log_debug!("Starting MCP server with StdIO transport");

    let transport = (stdin(), stdout());

    let server = toolbox.serve(transport).await?;

    // Wait for the server to finish
    log_debug!("MCP server initialized, waiting for completion");
    let quit_reason = server.waiting().await?;
    log_debug!("MCP server finished: {:?}", quit_reason);

    Ok(())
}

/// Start the MCP server using SSE transport
async fn serve_sse(toolbox: GitIrisToolbox, socket_addr: SocketAddr) -> Result<()> {
    log_debug!("Starting MCP server with SSE transport on {}", socket_addr);

    // Create and start the SSE server
    let server = SseServer::serve(socket_addr).await?;

    // Set up the service with our toolbox
    let control = server.with_service(move || {
        // Return a clone of the toolbox directly as it implements ServerHandler
        toolbox.clone()
    });

    // Wait for Ctrl+C signal
    log_debug!("SSE server initialized, waiting for interrupt signal");
    tokio::signal::ctrl_c()
        .await
        .context("Failed to listen for ctrl+c signal")?;

    // Cancel the server gracefully
    log_debug!("Interrupt signal received, shutting down SSE server");
    control.cancel();

    Ok(())
}

/// Helper function to get a socket address from the configuration
fn get_socket_addr(config: &MCPServerConfig) -> Result<SocketAddr> {
    // Get listen address, or use default
    let listen_address = config.listen_address.as_deref().unwrap_or("127.0.0.1");
    let port = config.port.context("Port is required for SSE transport")?;

    // Parse the socket address
    let socket_addr: SocketAddr = format!("{}:{}", listen_address, port)
        .parse()
        .context("Failed to parse socket address")?;

    Ok(socket_addr)
}
