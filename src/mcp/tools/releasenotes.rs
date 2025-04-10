//! MCP release notes tool implementation
//!
//! This module provides the MCP tool for generating release notes.

use crate::changes;
use crate::common::DetailLevel;
use crate::git::GitRepo;
use crate::log_debug;
use crate::config::Config as GitIrisConfig;

use rmcp::tool;
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request parameters for generating release notes
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReleaseNotesRequest {
    /// Starting reference (commit hash, tag, or branch name)
    #[schemars(description = "Starting reference (commit hash, tag, or branch name)")]
    pub from: String,
    
    /// Ending reference (commit hash, tag, or branch name). Defaults to HEAD if not specified.
    #[schemars(description = "Ending reference (commit hash, tag, or branch name). Defaults to HEAD if empty or not specified.")]
    #[serde(default)]
    pub to: String,
    
    /// Level of detail for the release notes
    #[schemars(description = "Level of detail for the release notes. Values: 'minimal', 'detailed'. Defaults to 'standard' if empty or not specified.")]
    #[serde(default)]
    pub detail_level: String,
    
    /// Custom instructions for the AI
    #[schemars(description = "Custom instructions for the AI. Empty by default.")]
    #[serde(default)]
    pub custom_instructions: String,
}

// Implementation will be handled by the tool_box macro in the parent module
