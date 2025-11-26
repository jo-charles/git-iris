//! LLM provider utilities and configuration helpers.
//!
//! This module provides utility functions for working with LLM providers.
//! The actual LLM interaction is handled by the Rig framework in the agents module.

use crate::config::Config;
use anyhow::{Result, anyhow};
use llm::builder::LLMBackend;
use std::collections::HashMap;
use std::str::FromStr;

/// Returns a list of available LLM providers as strings
pub fn get_available_provider_names() -> Vec<String> {
    vec![
        "openai".to_string(),
        "anthropic".to_string(),
        "ollama".to_string(),
        "google".to_string(),
        "groq".to_string(),
        "xai".to_string(),
        "deepseek".to_string(),
        "phind".to_string(),
    ]
}

/// Returns the default model for a given provider
pub fn get_default_model_for_provider(provider_type: &str) -> &'static str {
    match provider_type.to_lowercase().as_str() {
        "anthropic" => "claude-sonnet-4-5-20250929",
        "ollama" => "llama3",
        "google" => "gemini-2.5-pro-preview-06-05",
        "groq" => "llama-3.1-70b-versatile",
        "xai" => "grok-2-beta",
        "deepseek" => "deepseek-chat",
        "phind" => "phind-v2",
        _ => "gpt-5.1", // Latest OpenAI model
    }
}

/// Returns the default fast model for a given provider (optimized for speed over quality)
pub fn get_default_fast_model_for_provider(provider_type: &str) -> &'static str {
    match provider_type.to_lowercase().as_str() {
        "anthropic" => "claude-haiku-4-5-20251001",
        "google" => "gemini-1.5-flash",
        "groq" => "llama-3.1-8b-instant",
        "xai" => "grok-2-beta",        // No fast variant available
        "deepseek" => "deepseek-chat", // No fast variant available
        "ollama" => "llama3:8b",       // Smaller variant
        "phind" => "phind-v2",         // No fast variant available
        _ => "gpt-5-mini",             // Latest OpenAI fast model
    }
}

/// Returns the default token limit for a given provider
pub fn get_default_token_limit_for_provider(provider_type: &str) -> Result<usize> {
    let limit = match provider_type.to_lowercase().as_str() {
        "anthropic" => 200_000,
        "ollama" | "openai" | "groq" | "xai" => 128_000,
        "google" => 1_000_000,
        "deepseek" => 64_000,
        "phind" => 32_000,
        _ => 8_192, // Default token limit
    };
    Ok(limit)
}

/// Checks if a provider requires an API key
pub fn provider_requires_api_key(provider_type: &str) -> bool {
    if let Ok(backend) = LLMBackend::from_str(provider_type) {
        requires_api_key(&backend)
    } else {
        true // Default to requiring API key for unknown providers
    }
}

/// Helper function: check if `LLMBackend` requires API key
fn requires_api_key(backend: &LLMBackend) -> bool {
    !matches!(backend, LLMBackend::Ollama | LLMBackend::Phind)
}

/// Helper function: check if the model is an `OpenAI` thinking model
fn is_openai_thinking_model(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.starts_with('o')
}

/// Validates the provider configuration
pub fn validate_provider_config(config: &Config, provider_name: &str) -> Result<()> {
    if provider_requires_api_key(provider_name) {
        let provider_config = config
            .get_provider_config(provider_name)
            .ok_or_else(|| anyhow!("Provider '{}' not found in configuration", provider_name))?;

        if provider_config.api_key.is_empty() {
            return Err(anyhow!("API key required for provider: {}", provider_name));
        }
    }

    Ok(())
}

/// Combines default, saved, and command-line configurations
pub fn get_combined_config<S: ::std::hash::BuildHasher>(
    config: &Config,
    provider_name: &str,
    command_line_args: &HashMap<String, String, S>,
) -> HashMap<String, String> {
    let mut combined_params = HashMap::default();

    // Add default values
    combined_params.insert(
        "model".to_string(),
        get_default_model_for_provider(provider_name).to_string(),
    );

    // Add saved config values if available
    if let Some(provider_config) = config.get_provider_config(provider_name) {
        if !provider_config.api_key.is_empty() {
            combined_params.insert("api_key".to_string(), provider_config.api_key.clone());
        }
        if !provider_config.model.is_empty() {
            combined_params.insert("model".to_string(), provider_config.model.clone());
        }
        for (key, value) in &provider_config.additional_params {
            combined_params.insert(key.clone(), value.clone());
        }
    }

    // Add command line args (these take precedence)
    for (key, value) in command_line_args {
        if !value.is_empty() {
            combined_params.insert(key.clone(), value.clone());
        }
    }

    // Handle OpenAI thinking models: convert max_tokens to max_completion_tokens
    if provider_name.to_lowercase() == "openai"
        && let Some(model) = combined_params.get("model")
        && is_openai_thinking_model(model)
        && let Some(max_tokens) = combined_params.remove("max_tokens")
    {
        combined_params.insert("max_completion_tokens".to_string(), max_tokens);
    }

    combined_params
}
