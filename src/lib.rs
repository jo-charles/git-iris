//! Git-Iris - AI-powered Git workflow assistant
//!
//! This library provides intelligent assistance for Git workflows including commit message generation,
//! code reviews, pull request descriptions, changelogs, and release notes.

// Allow certain clippy warnings that are either stylistic or from external dependencies
#![allow(clippy::uninlined_format_args)] // Style preference
#![allow(clippy::format_push_string)] // Performance improvement but stylistic
#![allow(clippy::future_not_send)] // From Rig framework internals, can't fix
#![allow(clippy::return_self_not_must_use)] // Builder pattern is clear enough
#![allow(clippy::items_after_statements)] // Locally-scoped use statements are fine
#![allow(clippy::too_many_arguments)] // Some functions legitimately need many params
#![allow(clippy::option_as_ref_cloned)] // .as_ref().cloned() is sometimes clearer
#![allow(clippy::redundant_clone)] // Sometimes more explicit is clearer

pub mod agents;
pub mod changes;
pub mod cli;
pub mod commands;
pub mod commit;
pub mod common;
pub mod config;
pub mod context;
pub mod file_analyzers;
pub mod git;
pub mod gitmoji;
pub mod instruction_presets;
pub mod llm;
pub mod logger;
pub mod messages;
pub mod services;
pub mod token_optimizer;
pub mod tui;
pub mod ui;

// Re-export important structs and functions for easier testing
pub use config::Config;
pub use config::ProviderConfig;
// Re-export the LLMProvider trait from the external llm crate
pub use ::llm::LLMProvider;

// Re-exports from the new types organization
pub use commit::review::{CodeIssue, DimensionAnalysis, GeneratedReview, QualityDimension};
pub use commit::types::{
    GeneratedMessage, GeneratedPullRequest, format_commit_message, format_pull_request,
};
