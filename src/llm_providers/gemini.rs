use super::{LLMProvider, LLMProviderConfig, ProviderMetadata};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

/// Represents the Gemini LLM provider
pub struct GeminiProvider {
    config: LLMProviderConfig,
    client: Client,
}

impl GeminiProvider {
    /// Creates a new instance of `GeminiProvider` with the given configuration
    pub fn new(config: LLMProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    /// Generates a message using the Gemini API
    async fn generate_message(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        // Use default model if not specified
        let model = if self.config.model.is_empty() {
            "gemini-2.0-flash"
        } else {
            &self.config.model
        };

        let mut request_body = json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        {"text": format!("{}\n\n{}", system_prompt, user_prompt)}
                    ]
                }
            ],
            "generationConfig": {
                // Model is specified in the URL, not here
                "maxOutputTokens": 4096
            }
        });

        // Add additional parameters from the configuration
        for (key, value) in &self.config.additional_params {
            // Try to convert the value to a number first if it looks like one
            if let Ok(num_val) = value.parse::<f64>() {
                request_body["generationConfig"][key] = json!(num_val);
            } else {
                request_body["generationConfig"][key] = json!(value);
            }
        }

        // If 'response_mime_type' is not already set and we're parsing JSON output,
        // add the application/json mime type
        if !self
            .config
            .additional_params
            .contains_key("response_mime_type")
            && user_prompt.contains("JSON")
        {
            request_body["generationConfig"]["response_mime_type"] = json!("application/json");
        }

        // Make the API request
        let api_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.config.api_key
        );

        let response = self
            .client
            .post(api_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        // Check for successful response
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Gemini API request failed with status {}: {}",
                status,
                text
            ));
        }

        // Parse the response body
        let response_body: serde_json::Value = response.json().await?;

        // Extract content from the response
        // The response format is:
        // {
        //   "candidates": [
        //     {
        //       "content": {
        //         "parts": [
        //           { "text": "Response text here" }
        //         ]
        //       }
        //     }
        //   ]
        // }
        let content = response_body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to extract content from Gemini API response"))?;

        Ok(content.to_string())
    }
}

pub(super) fn get_metadata() -> ProviderMetadata {
    ProviderMetadata {
        name: "Gemini",
        default_model: "gemini-2.0-flash",
        default_token_limit: 1_048_576,
        requires_api_key: true,
    }
}
