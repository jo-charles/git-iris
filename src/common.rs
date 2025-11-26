use crate::config::Config;
use crate::instruction_presets::{PresetType, get_instruction_preset_library};
use crate::providers::{Provider, ProviderConfig};
use anyhow::Result;
use clap::Args;

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

    /// Repository URL to use instead of local repository
    #[arg(
        short = 'r',
        long = "repo",
        help = "Repository URL to use instead of local repository"
    )]
    pub repository_url: Option<String>,
}

impl CommonParams {
    pub fn apply_to_config(&self, config: &mut Config) -> Result<bool> {
        let mut changes_made = false;

        if let Some(provider_str) = &self.provider {
            // Parse and validate provider
            let provider: Provider = provider_str.parse()?;
            let provider_name = provider.name().to_string();

            // Check if we need to update the default provider
            if config.default_provider != provider_name {
                // Ensure the provider exists in the providers HashMap
                if !config.providers.contains_key(&provider_name) {
                    config.providers.insert(
                        provider_name.clone(),
                        ProviderConfig::with_defaults(provider),
                    );
                }

                config.default_provider = provider_name;
                changes_made = true;
            }
        }

        if let Some(instructions) = &self.instructions {
            config.set_temp_instructions(Some(instructions.clone()));
        }

        if let Some(preset) = &self.preset {
            config.set_temp_preset(Some(preset.clone()));
        }

        if let Some(use_gitmoji) = self.gitmoji
            && config.use_gitmoji != use_gitmoji
        {
            config.use_gitmoji = use_gitmoji;
            changes_made = true;
        }

        Ok(changes_made)
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
    match s.parse::<Provider>() {
        Ok(provider) => Ok(provider.name().to_string()),
        Err(_) => Err(format!(
            "Invalid provider '{}'. Available providers: {}",
            s,
            Provider::all_names().join(", ")
        )),
    }
}
