//! Output validation and error recovery for agent responses
//!
//! This module provides robust JSON validation and recovery mechanisms
//! for handling malformed or partially correct LLM responses.

use anyhow::{Context, Result};
use schemars::JsonSchema;
use schemars::schema_for;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::agents::debug;

/// Validation result with recovery information
#[derive(Debug)]
pub struct ValidationResult<T> {
    /// The parsed value (if successful)
    pub value: Option<T>,
    /// Warnings encountered during parsing (non-fatal issues)
    pub warnings: Vec<String>,
    /// Whether recovery was needed
    pub recovered: bool,
}

impl<T> ValidationResult<T> {
    fn success(value: T) -> Self {
        Self {
            value: Some(value),
            warnings: vec![],
            recovered: false,
        }
    }

    fn recovered(value: T, warnings: Vec<String>) -> Self {
        Self {
            value: Some(value),
            warnings,
            recovered: true,
        }
    }
}

/// Validate and parse JSON with schema validation and error recovery
pub fn validate_and_parse<T>(json_str: &str) -> Result<ValidationResult<T>>
where
    T: JsonSchema + DeserializeOwned,
{
    let mut warnings = Vec::new();

    // First, try direct parsing
    match serde_json::from_str::<T>(json_str) {
        Ok(value) => {
            debug::debug_json_parse_success(std::any::type_name::<T>());
            return Ok(ValidationResult::success(value));
        }
        Err(e) => {
            debug::debug_json_parse_error(&format!("Initial parse failed: {}", e));
            warnings.push(format!("Initial parse failed: {}", e));
        }
    }

    // Parse as generic Value for recovery attempts
    let mut json_value: Value = serde_json::from_str(json_str)
        .context("Response is not valid JSON - cannot attempt recovery")?;

    // Get the expected schema
    let schema = schema_for!(T);
    let schema_value = serde_json::to_value(&schema).unwrap_or(Value::Null);

    // Attempt recovery based on schema
    if let Some(obj) = json_value.as_object_mut() {
        recover_missing_fields(obj, &schema_value, &mut warnings);
        recover_type_mismatches(obj, &schema_value, &mut warnings);
        recover_null_to_defaults(obj, &schema_value, &mut warnings);
    }

    // Try parsing again after recovery
    match serde_json::from_value::<T>(json_value.clone()) {
        Ok(value) => {
            debug::debug_context_management(
                "JSON recovery successful",
                &format!("{} warnings", warnings.len()),
            );
            Ok(ValidationResult::recovered(value, warnings))
        }
        Err(e) => {
            // Final attempt: try to extract just the required fields
            let final_value = extract_required_fields(&json_value, &schema_value);
            match serde_json::from_value::<T>(final_value) {
                Ok(value) => {
                    warnings.push(format!("Extracted required fields only: {}", e));
                    Ok(ValidationResult::recovered(value, warnings))
                }
                Err(final_e) => Err(anyhow::anyhow!(
                    "Failed to parse JSON even after recovery attempts: {}",
                    final_e
                )),
            }
        }
    }
}

/// Recover missing required fields by adding defaults
fn recover_missing_fields(
    obj: &mut Map<String, Value>,
    schema: &Value,
    warnings: &mut Vec<String>,
) {
    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return;
    };

    let required = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    for field_name in required {
        if !obj.contains_key(field_name)
            && let Some(prop_schema) = properties.get(field_name)
        {
            let default_value = get_default_for_type(prop_schema);
            warnings.push(format!(
                "Added missing required field '{}' with default value",
                field_name
            ));
            obj.insert(field_name.to_string(), default_value);
        }
    }
}

/// Recover type mismatches by attempting conversion
fn recover_type_mismatches(
    obj: &mut Map<String, Value>,
    schema: &Value,
    warnings: &mut Vec<String>,
) {
    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return;
    };

    for (field_name, prop_schema) in properties {
        if let Some(current_value) = obj.get(field_name).cloned() {
            let expected_type = prop_schema
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("any");

            let converted = match expected_type {
                "string" => convert_to_string(&current_value),
                "array" => Some(convert_to_array(&current_value)),
                "boolean" => convert_to_boolean(&current_value),
                "integer" | "number" => convert_to_number(&current_value),
                _ => None,
            };

            if let Some(new_value) = converted
                && new_value != current_value
            {
                warnings.push(format!(
                    "Converted field '{}' from {:?} to {}",
                    field_name,
                    type_name(&current_value),
                    expected_type
                ));
                obj.insert(field_name.clone(), new_value);
            }
        }
    }
}

/// Recover null values by replacing with appropriate defaults
fn recover_null_to_defaults(
    obj: &mut Map<String, Value>,
    schema: &Value,
    warnings: &mut Vec<String>,
) {
    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return;
    };

    for (field_name, prop_schema) in properties {
        if let Some(Value::Null) = obj.get(field_name) {
            // Check if field is nullable (has "anyOf" with null type)
            let is_nullable = prop_schema
                .get("anyOf")
                .and_then(|a| a.as_array())
                .is_some_and(|arr| {
                    arr.iter()
                        .any(|v| v.get("type") == Some(&Value::String("null".to_string())))
                });

            if !is_nullable {
                let default_value = get_default_for_type(prop_schema);
                warnings.push(format!(
                    "Replaced null value in non-nullable field '{}' with default",
                    field_name
                ));
                obj.insert(field_name.clone(), default_value);
            }
        }
    }
}

/// Extract only required fields from a JSON value
fn extract_required_fields(value: &Value, schema: &Value) -> Value {
    let Some(obj) = value.as_object() else {
        return value.clone();
    };

    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return value.clone();
    };

    let required: Vec<&str> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut result = Map::new();

    for field_name in required {
        if let Some(field_value) = obj.get(field_name) {
            result.insert(field_name.to_string(), field_value.clone());
        } else if let Some(prop_schema) = properties.get(field_name) {
            result.insert(field_name.to_string(), get_default_for_type(prop_schema));
        }
    }

    // Also include optional fields that are present
    for (field_name, field_value) in obj {
        if !result.contains_key(field_name) {
            result.insert(field_name.clone(), field_value.clone());
        }
    }

    Value::Object(result)
}

/// Get a sensible default value for a JSON schema type
fn get_default_for_type(schema: &Value) -> Value {
    // Check for explicit default first
    if let Some(default) = schema.get("default") {
        return default.clone();
    }

    // Check for anyOf (nullable types)
    if let Some(any_of) = schema.get("anyOf").and_then(|a| a.as_array()) {
        for variant in any_of {
            if variant.get("type") == Some(&Value::String("null".to_string())) {
                return Value::Null;
            }
        }
        // Use first non-null type's default
        if let Some(first) = any_of.first() {
            return get_default_for_type(first);
        }
    }

    match schema.get("type").and_then(|t| t.as_str()) {
        Some("string") => Value::String(String::new()),
        Some("array") => Value::Array(vec![]),
        Some("object") => Value::Object(Map::new()),
        Some("boolean") => Value::Bool(false),
        Some("integer" | "number") => Value::Number(0.into()),
        _ => Value::Null,
    }
}

/// Convert a value to string if possible
fn convert_to_string(value: &Value) -> Option<Value> {
    match value {
        Value::String(_) => Some(value.clone()),
        Value::Number(n) => Some(Value::String(n.to_string())),
        Value::Bool(b) => Some(Value::String(b.to_string())),
        Value::Null => Some(Value::String(String::new())),
        Value::Array(arr) => {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            Some(Value::String(strings.join(", ")))
        }
        Value::Object(_) => None,
    }
}

/// Convert a value to array
fn convert_to_array(value: &Value) -> Value {
    match value {
        Value::Array(_) => value.clone(),
        Value::Null => Value::Array(vec![]),
        // Wrap single value in array
        other => Value::Array(vec![other.clone()]),
    }
}

/// Convert a value to boolean if possible
fn convert_to_boolean(value: &Value) -> Option<Value> {
    match value {
        Value::Bool(_) => Some(value.clone()),
        Value::String(s) => match s.to_lowercase().as_str() {
            "true" | "yes" | "1" => Some(Value::Bool(true)),
            "false" | "no" | "0" | "" => Some(Value::Bool(false)),
            _ => None,
        },
        Value::Number(n) => Some(Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0)),
        Value::Null => Some(Value::Bool(false)),
        Value::Array(_) | Value::Object(_) => None,
    }
}

/// Convert a value to number if possible
fn convert_to_number(value: &Value) -> Option<Value> {
    match value {
        Value::Number(_) => Some(value.clone()),
        Value::String(s) => {
            // Try parsing as integer first to preserve integer semantics
            if let Ok(i) = s.parse::<i64>() {
                return Some(Value::Number(i.into()));
            }
            // Fall back to float
            s.parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
                .map(Value::Number)
        }
        Value::Bool(b) => Some(Value::Number(i32::from(*b).into())),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

/// Get a human-readable type name for a JSON value
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
    struct TestOutput {
        title: String,
        message: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        count: i32,
    }

    #[test]
    fn test_valid_json_parses_directly() {
        let json = r#"{"title": "Test", "message": "Hello", "tags": ["a", "b"], "count": 5}"#;
        let result = validate_and_parse::<TestOutput>(json).expect("should parse");
        assert!(!result.recovered);
        assert!(result.warnings.is_empty());
        assert_eq!(result.value.expect("should have value").title, "Test");
    }

    #[test]
    fn test_recovers_missing_optional_fields() {
        let json = r#"{"title": "Test", "message": "Hello"}"#;
        let result = validate_and_parse::<TestOutput>(json).expect("should parse");
        let value = result.value.expect("should have value");
        assert_eq!(value.title, "Test");
        assert!(value.tags.is_empty());
    }

    #[test]
    fn test_converts_number_to_string() {
        let json = r#"{"title": 123, "message": "Hello"}"#;
        let result = validate_and_parse::<TestOutput>(json).expect("should parse");
        assert!(result.recovered);
        assert_eq!(result.value.expect("should have value").title, "123");
    }

    #[test]
    fn test_converts_single_value_to_array() {
        let json = r#"{"title": "Test", "message": "Hello", "tags": "single"}"#;
        let result = validate_and_parse::<TestOutput>(json).expect("should parse");
        assert!(result.recovered);
        assert_eq!(
            result.value.expect("should have value").tags,
            vec!["single"]
        );
    }

    #[test]
    fn test_converts_string_to_number() {
        let json = r#"{"title": "Test", "message": "Hello", "count": "42"}"#;
        let result = validate_and_parse::<TestOutput>(json).expect("should parse");
        assert!(result.recovered);
        assert_eq!(result.value.expect("should have value").count, 42);
    }
}
