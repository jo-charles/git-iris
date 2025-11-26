//! Agent system for Git-Iris
//!
//! This module provides the foundation for AI agent orchestration in Git-Iris,
//! including setup, core types, and execution management.

// Core agent components
pub mod context;
pub mod core;
pub mod iris;
pub mod prompts;

// Agent tools
pub mod tools;

// Setup and configuration
pub mod setup;

// Status and reporting
pub mod status;

// Debug observability
pub mod debug;
pub mod debug_tool;

// Output validation and recovery
pub mod output_validator;

// Re-exports for public API
pub use context::TaskContext;
pub use core::{AgentBackend, AgentContext, TaskResult};
pub use iris::{IrisAgent, IrisAgentBuilder, StreamingCallback, StructuredResponse};
pub use setup::{AgentSetupService, IrisAgentService, handle_with_agent};
pub use tools::{GitChangedFiles, GitDiff, GitLog, GitRepoInfo, GitStatus};
