use crate::config::Config;
use crate::instruction_presets::{PresetType, get_instruction_preset_library};
use crate::llm;
use anyhow::Result;
use clap::Args;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DetailLevel {
    Minimal,
    Standard,
    Detailed,
}

impl FromStr for DetailLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(Self::Minimal),
            "standard" => Ok(Self::Standard),
            "detailed" => Ok(Self::Detailed),
            _ => Err(anyhow::anyhow!("Invalid detail level: {}", s)),
        }
    }
}

impl DetailLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Detailed => "detailed",
        }
    }
}

#[derive(Args, Clone, Default, Debug)]
pub struct CommonParams {
    /// Override default LLM provider
    #[arg(long, help = "Override default LLM provider", value_parser = available_providers_parser)]
    pub provider: Option<String>,

    /// Custom instructions for this operation
    #[arg(short, long, help = "Custom instructions for this operation")]
    pub instructions: Option<String>,

    /// Select an instruction preset
    #[arg(
        long,
        help = "Select an instruction preset (use 'git-iris list-presets' to see available presets for commits and reviews)"
    )]
    pub preset: Option<String>,

    /// Enable or disable Gitmoji
    #[arg(long, help = "Enable or disable Gitmoji")]
    pub gitmoji: Option<bool>,

    /// Set the detail level
    #[arg(
        long,
        help = "Set the detail level (minimal, standard, detailed)",
        default_value = "standard"
    )]
    pub detail_level: String,

    /// Repository URL to use instead of local repository
    #[arg(
        short = 'r',
        long = "repo",
        help = "Repository URL to use instead of local repository"
    )]
    pub repository_url: Option<String>,
}

impl CommonParams {
    pub fn apply_to_config(&self, config: &mut Config) -> Result<()> {
        if let Some(provider) = &self.provider {
            // Convert "claude" to "anthropic" for backward compatibility
            let provider_name = if provider.to_lowercase() == "claude" {
                "anthropic".to_string()
            } else {
                provider.clone()
            };
            config.default_provider.clone_from(&provider_name);
        }
        if let Some(instructions) = &self.instructions {
            config.set_temp_instructions(Some(instructions.clone()));
        }
        if let Some(preset) = &self.preset {
            config.set_temp_preset(Some(preset.clone()));
        }
        if let Some(use_gitmoji) = self.gitmoji {
            config.use_gitmoji = use_gitmoji;
        }
        Ok(())
    }

    /// Check if the provided preset is valid for the specified preset type
    pub fn is_valid_preset_for_type(&self, preset_type: PresetType) -> bool {
        if let Some(preset_name) = &self.preset {
            let library = get_instruction_preset_library();
            let valid_presets = library.list_valid_presets_for_command(preset_type);
            return valid_presets.iter().any(|(key, _)| *key == preset_name);
        }
        true // No preset specified is always valid
    }
}

/// Validates that a provider name is available in the system
pub fn available_providers_parser(s: &str) -> Result<String, String> {
    let mut provider_name = s.to_lowercase();

    // Handle legacy "claude" provider name by mapping it to "anthropic"
    if provider_name == "claude" {
        provider_name = "anthropic".to_string();
    }

    let available_providers = llm::get_available_provider_names();

    if available_providers
        .iter()
        .any(|p| p.to_lowercase() == provider_name)
    {
        Ok(provider_name)
    } else {
        Err(format!(
            "Invalid provider '{}'. Available providers: {}",
            s,
            available_providers.join(", ")
        ))
    }
}

pub fn get_combined_instructions(config: &Config) -> String {
    let mut prompt = String::from("\n\n");

    if !config.instructions.is_empty() {
        prompt.push_str(&format!(
            "\n\nAdditional instructions for the request:\n{}\n\n",
            config.instructions
        ));
    }

    let preset_library = get_instruction_preset_library();
    if let Some(preset_instructions) = preset_library.get_preset(config.instruction_preset.as_str())
    {
        prompt.push_str(&format!(
            "\n\nIMPORTANT: Use this style for your output:\n{}\n\n",
            preset_instructions.instructions
        ));
    }

    prompt
}
