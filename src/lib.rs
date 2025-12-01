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
pub mod changelog;
pub mod cli;
pub mod commands;
pub mod common;
pub mod companion;
pub mod config;
pub mod context;
pub mod git;
pub mod gitmoji;
pub mod instruction_presets;
pub mod logger;
pub mod messages;
pub mod output;
pub mod providers;
pub mod services;
pub mod studio;
pub mod theme;
pub mod types;
pub mod ui;

// Re-export important structs and functions for easier testing
pub use config::Config;
pub use providers::{Provider, ProviderConfig};

// Re-exports from types module
pub use types::{
    GeneratedMessage, MarkdownPullRequest, MarkdownReleaseNotes, MarkdownReview,
    format_commit_message,
};
