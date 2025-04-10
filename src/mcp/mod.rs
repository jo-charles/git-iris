//! Model Context Protocol (MCP) integration for Git-Iris
//!
//! This module contains the implementation of the MCP server
//! that allows Git-Iris to be used directly from compatible
//! LLM-powered tools and assistants.

pub mod config;
pub mod server;
pub mod tools;

#[cfg(test)]
mod tests;

// Re-export main components
pub use server::serve;
