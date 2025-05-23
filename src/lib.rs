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
pub mod mcp;
pub mod messages;
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
pub use commit::types::{GeneratedMessage, format_commit_message};
