use anyhow::Result;
use git_iris::config::Config;
use git_iris::llm::{
    get_available_provider_names, get_default_model_for_provider,
    get_default_token_limit_for_provider, validate_provider_config,
};

#[test]
fn test_get_available_providers() {
    let providers = get_available_provider_names();
    assert!(!providers.is_empty(), "Provider list should not be empty");

    // Check if common providers are available
    // Note: The actual list may vary depending on the built-in backends in the llm crate
    let common_providers = vec!["openai", "anthropic", "ollama"];
    let mut found_any = false;

    for provider in common_providers {
        if providers.iter().any(|p| p == provider) {
            found_any = true;
            break;
        }
    }

    assert!(found_any, "Expected to find at least one common provider");
}

#[test]
fn test_get_default_model_for_provider() {
    // Test known providers
    assert_eq!(get_default_model_for_provider("openai"), "gpt-4o");
    assert_eq!(
        get_default_model_for_provider("anthropic"),
        "claude-3-7-sonnet-20250219"
    );
    assert_eq!(get_default_model_for_provider("google"), "gemini-2.0-flash");
    assert_eq!(get_default_model_for_provider("xai"), "grok-2-beta");

    // Test fallback for unknown provider
    assert_eq!(get_default_model_for_provider("unknown"), "gpt-4o");
}

#[test]
fn test_get_default_token_limit_for_provider() -> Result<()> {
    // Test known providers
    assert_eq!(get_default_token_limit_for_provider("openai")?, 128_000);
    assert_eq!(get_default_token_limit_for_provider("anthropic")?, 200_000);
    assert_eq!(get_default_token_limit_for_provider("google")?, 1_000_000);
    assert_eq!(get_default_token_limit_for_provider("deepseek")?, 64_000);
    assert_eq!(get_default_token_limit_for_provider("phind")?, 32_000);

    // Test fallback for unknown provider
    assert_eq!(get_default_token_limit_for_provider("unknown")?, 8_192);
    Ok(())
}

#[test]
fn test_validate_provider_config() -> Result<()> {
    // Create a config with valid provider configuration
    let mut config = Config::default();
    config.default_provider = "openai".to_string();

    let mut provider_config = git_iris::config::ProviderConfig::default();
    provider_config.api_key = "dummy-api-key".to_string();
    provider_config.model = "gpt-4o".to_string();

    config
        .providers
        .insert("openai".to_string(), provider_config);

    // Validation should pass with API key set
    assert!(validate_provider_config(&config, "openai").is_ok());

    // Test with missing API key
    let mut invalid_config = config.clone();
    invalid_config.providers.get_mut("openai").unwrap().api_key = String::new();
    assert!(validate_provider_config(&invalid_config, "openai").is_err());

    Ok(())
}
