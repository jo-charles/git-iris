//! Common utilities for agent tools
//!
//! This module provides shared functionality used across multiple tools:
//! - Schema generation for OpenAI-compatible tool definitions
//! - Error type macros
//! - Repository initialization helpers

use serde_json::{Map, Value};

use crate::git::GitRepo;

/// Generate a JSON schema for tool parameters that's `OpenAI`-compatible.
/// `OpenAI` tool schemas require the `required` array to list every property.
pub fn parameters_schema<T: schemars::JsonSchema>() -> Value {
    use schemars::schema_for;

    let schema = schema_for!(T);
    let mut value = serde_json::to_value(schema).expect("tool schema should serialize");
    enforce_required_properties(&mut value);
    value
}

/// Ensure all properties are listed in the `required` array.
/// This is needed for `OpenAI` tool compatibility.
fn enforce_required_properties(value: &mut Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };

    let props_entry = obj
        .entry("properties")
        .or_insert_with(|| Value::Object(Map::new()));
    let props_obj = props_entry.as_object().expect("properties must be object");
    let required_keys: Vec<Value> = props_obj.keys().cloned().map(Value::String).collect();

    obj.insert("required".to_string(), Value::Array(required_keys));
}

/// Get the current repository from the working directory.
/// This is a common operation used by most tools.
pub fn get_current_repo() -> anyhow::Result<GitRepo> {
    let current_dir = std::env::current_dir()?;
    GitRepo::new(&current_dir)
}

/// Macro to define a tool error type with standard From implementations.
///
/// This creates a newtype wrapper around String that implements:
/// - `Debug`, `thiserror::Error`
/// - `From<anyhow::Error>`
/// - `From<std::io::Error>`
///
/// # Example
/// ```ignore
/// define_tool_error!(GitError);
/// // Creates: pub struct GitError(String);
/// // With Display showing: "GitError: {message}"
/// ```
#[macro_export]
macro_rules! define_tool_error {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::error::Error for $name {}

        impl From<anyhow::Error> for $name {
            fn from(err: anyhow::Error) -> Self {
                $name(err.to_string())
            }
        }

        impl From<std::io::Error> for $name {
            fn from(err: std::io::Error) -> Self {
                $name(err.to_string())
            }
        }
    };
}

// Re-export the macro at the module level
pub use define_tool_error;
