//! Agent system for Git-Iris
//!
//! This module provides the foundation for AI agent orchestration in Git-Iris,
//! including setup, core types, and execution management.

// Core agent components
pub mod core;
pub mod iris;
pub mod prompts;

// Agent services
pub mod services;

// Agent tools
pub mod tools;

// Setup and configuration
pub mod setup;

// Status and reporting
pub mod status;

// Re-exports for public API
pub use core::{AgentBackend, AgentContext, TaskResult};
pub use iris::{IrisAgent, IrisAgentBuilder, StreamingCallback};
pub use setup::{AgentSetupService, handle_with_agent};
pub use tools::{GitChangedFiles, GitDiff, GitLog, GitRepoInfo, GitStatus};
