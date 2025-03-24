use crate::config::Config;
use crate::log_debug;
use anyhow::{Result, anyhow};
use llm::{
    LLMProvider,
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;

/// Generates a message using the given configuration
pub async fn get_refined_message<T>(
    config: &Config,
    provider_name: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<T>
where
    T: Serialize + DeserializeOwned + std::fmt::Debug,
    String: Into<T>,
{
    log_debug!(
        "Generating refined message using provider: {}",
        provider_name
    );
    log_debug!("System prompt: {}", system_prompt);
    log_debug!("User prompt: {}", user_prompt);

    // Parse the provider type
    let backend =
        LLMBackend::from_str(provider_name).map_err(|e| anyhow!("Invalid provider: {}", e))?;

    // Get provider configuration
    let provider_config = config
        .get_provider_config(provider_name)
        .ok_or_else(|| anyhow!("Provider '{}' not found in configuration", provider_name))?;

    // Build the provider
    let mut builder = LLMBuilder::new().backend(backend.clone());

    // Set model
    if !provider_config.model.is_empty() {
        builder = builder.model(provider_config.model.clone());
    }

    // Set system prompt
    builder = builder.system(system_prompt.to_string());

    // Set API key if needed
    if requires_api_key(&backend) && !provider_config.api_key.is_empty() {
        builder = builder.api_key(provider_config.api_key.clone());
    }

    // Set temperature if specified in additional params
    if let Some(temp) = provider_config.additional_params.get("temperature") {
        if let Ok(temp_val) = temp.parse::<f32>() {
            builder = builder.temperature(temp_val);
        }
    }

    // Set max tokens if specified in additional params, otherwise use 4096 as default
    if let Some(max_tokens) = provider_config.additional_params.get("max_tokens") {
        if let Ok(mt_val) = max_tokens.parse::<u32>() {
            builder = builder.max_tokens(mt_val);
        }
    } else {
        builder = builder.max_tokens(4096);
    }

    // Set top_p if specified in additional params
    if let Some(top_p) = provider_config.additional_params.get("top_p") {
        if let Ok(tp_val) = top_p.parse::<f32>() {
            builder = builder.top_p(tp_val);
        }
    }

    // Build the provider
    let provider = builder
        .build()
        .map_err(|e| anyhow!("Failed to build provider: {}", e))?;

    // Generate the message
    let result = get_refined_message_with_provider::<T>(provider, user_prompt).await?;

    Ok(result)
}

/// Generates a message using the given provider (mainly for testing purposes)
pub async fn get_refined_message_with_provider<T>(
    provider: Box<dyn LLMProvider + Send + Sync>,
    user_prompt: &str,
) -> Result<T>
where
    T: Serialize + DeserializeOwned + std::fmt::Debug,
    String: Into<T>,
{
    log_debug!("Entering get_refined_message_with_provider");

    let retry_strategy = ExponentialBackoff::from_millis(10).factor(2).take(2); // 2 attempts total: initial + 1 retry

    let result = Retry::spawn(retry_strategy, || async {
        log_debug!("Attempting to generate message");

        // Create chat message with user prompt
        let messages = vec![ChatMessage::user().content(user_prompt.to_string()).build()];

        match tokio::time::timeout(Duration::from_secs(30), provider.chat(&messages)).await {
            Ok(Ok(response)) => {
                log_debug!("Received response from provider");
                let response_text = response.text().unwrap_or_default();
                let cleaned_message = clean_json_from_llm(&response_text);

                if std::any::type_name::<T>() == std::any::type_name::<String>() {
                    // If T is String, return the raw string response
                    Ok(cleaned_message.into())
                } else {
                    // Attempt to deserialize the response
                    match serde_json::from_str::<T>(&cleaned_message) {
                        Ok(message) => Ok(message),
                        Err(e) => {
                            log_debug!("Deserialization error: {} message: {}", e, cleaned_message);
                            Err(anyhow!("Deserialization error: {}", e))
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                log_debug!("Provider error: {}", e);
                Err(anyhow!("Provider error: {}", e))
            }
            Err(_) => {
                log_debug!("Provider timed out");
                Err(anyhow!("Provider timed out"))
            }
        }
    })
    .await;

    match result {
        Ok(message) => {
            log_debug!("Deserialized message: {:?}", message);
            Ok(message)
        }
        Err(e) => {
            log_debug!("Failed to generate message after retries: {}", e);
            Err(anyhow!("Failed to generate message: {}", e))
        }
    }
}

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
        "anthropic" => "claude-3-7-sonnet-20250219",
        "ollama" => "llama3",
        "google" => "gemini-2.0-flash",
        "groq" => "llama-3.1-70b-versatile",
        "xai" => "grok-2-beta",
        "deepseek" => "deepseek-chat",
        "phind" => "phind-v2",
        _ => "gpt-4o", // Default to OpenAI's model
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

    combined_params
}

fn clean_json_from_llm(json_str: &str) -> String {
    // Remove potential leading/trailing whitespace and invisible characters
    let trimmed = json_str
        .trim_start_matches(|c: char| c.is_whitespace() || !c.is_ascii())
        .trim_end_matches(|c: char| c.is_whitespace() || !c.is_ascii());

    // If wrapped in code block, remove the markers
    let without_codeblock = if trimmed.starts_with("```") && trimmed.ends_with("```") {
        let start = trimmed.find('{').unwrap_or(0);
        let end = trimmed.rfind('}').map_or(trimmed.len(), |i| i + 1);
        &trimmed[start..end]
    } else {
        trimmed
    };

    // Find the first '{' and last '}' to extract the JSON object
    let start = without_codeblock.find('{').unwrap_or(0);
    let end = without_codeblock
        .rfind('}')
        .map_or(without_codeblock.len(), |i| i + 1);

    without_codeblock[start..end].trim().to_string()
}
