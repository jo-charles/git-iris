use crate::ProviderConfig;
use crate::common::CommonParams;
use crate::config::Config;
use crate::instruction_presets::{
    PresetType, get_instruction_preset_library, list_presets_formatted_by_type,
};
use crate::llm::get_available_provider_names;
use crate::log_debug;
use crate::ui;
use anyhow::Context;
use anyhow::{Result, anyhow};
use colored::Colorize;
use std::collections::HashMap;

/// Handle the 'config' command
#[allow(clippy::too_many_lines)]
pub fn handle_config_command(
    common: CommonParams,
    api_key: Option<String>,
    model: Option<String>,
    token_limit: Option<usize>,
    param: Option<Vec<String>>,
) -> anyhow::Result<()> {
    log_debug!(
        "Starting 'config' command with common: {:?}, api_key: {:?}, model: {:?}, token_limit: {:?}, param: {:?}",
        common,
        api_key,
        model,
        token_limit,
        param
    );

    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;
    let mut changes_made = false;

    if let Some(provider) = common.provider {
        if !get_available_provider_names()
            .iter()
            .any(|p| p == &provider)
        {
            return Err(anyhow!("Invalid provider: {}", provider));
        }
        if config.default_provider != provider {
            config.default_provider.clone_from(&provider);
            changes_made = true;
        }
        if !config.providers.contains_key(&provider) {
            config
                .providers
                .insert(provider.clone(), ProviderConfig::default());
            changes_made = true;
        }
    }

    let provider_config = config
        .providers
        .get_mut(&config.default_provider)
        .context("Could not get default provider")?;

    if let Some(key) = api_key {
        if provider_config.api_key != key {
            provider_config.api_key = key;
            changes_made = true;
        }
    }
    if let Some(model) = model {
        if provider_config.model != model {
            provider_config.model = model;
            changes_made = true;
        }
    }
    if let Some(params) = param {
        let additional_params = parse_additional_params(&params);
        if provider_config.additional_params != additional_params {
            provider_config.additional_params = additional_params;
            changes_made = true;
        }
    }
    if let Some(use_gitmoji) = common.gitmoji {
        if config.use_gitmoji != use_gitmoji {
            config.use_gitmoji = use_gitmoji;
            changes_made = true;
        }
    }
    if let Some(instr) = common.instructions {
        if config.instructions != instr {
            config.instructions = instr;
            changes_made = true;
        }
    }
    if let Some(limit) = token_limit {
        if provider_config.token_limit != Some(limit) {
            provider_config.token_limit = Some(limit);
            changes_made = true;
        }
    }
    if let Some(preset) = common.preset {
        let preset_library = get_instruction_preset_library();
        if preset_library.get_preset(&preset).is_some() {
            if config.instruction_preset != preset {
                config.instruction_preset = preset;
                changes_made = true;
            }
        } else {
            return Err(anyhow!("Invalid preset: {}", preset));
        }
    }

    if changes_made {
        config.save()?;
        ui::print_success("Configuration updated successfully.");
        println!();
    }

    // Print the configuration with beautiful styling
    print_configuration(&config);

    Ok(())
}

/// Display the configuration with beautiful styling and colors
fn print_configuration(config: &Config) {
    // Create a title with gradient
    println!(
        "\n{}",
        ui::create_gradient_text("🔮 Git-Iris Configuration 🔮").bold()
    );
    println!();

    // Global settings section
    println!("{}", "Global Settings".bright_magenta().bold().underline());
    println!();

    let provider_label = "Default Provider:".bright_cyan().bold();
    let provider_value = config.default_provider.bright_white();
    println!("  {} {} {}", "🔹".cyan(), provider_label, provider_value);

    let gitmoji_label = "Use Gitmoji:".bright_cyan().bold();
    let gitmoji_value = if config.use_gitmoji {
        "Yes".bright_green()
    } else {
        "No".bright_red()
    };
    println!("  {} {} {}", "🔹".cyan(), gitmoji_label, gitmoji_value);

    let preset_label = "Instruction Preset:".bright_cyan().bold();
    let preset_value = config.instruction_preset.bright_yellow();
    println!("  {} {} {}", "🔹".cyan(), preset_label, preset_value);

    println!();

    // Instructions section (if any)
    if !config.instructions.is_empty() {
        println!("{}", "Custom Instructions".bright_blue().bold().underline());
        println!();

        // Display full instructions, preserving newlines
        config.instructions.lines().for_each(|line| {
            println!("  {}", line.bright_white().italic());
        });

        println!();
    }

    // Provider configurations
    for (provider, provider_config) in &config.providers {
        println!(
            "{}",
            format!("Provider: {provider}")
                .bright_green()
                .bold()
                .underline()
        );
        println!();

        // API Key status with lock emoji
        let api_key_label = "API Key:".yellow().bold();
        let api_key_value = if provider_config.api_key.is_empty() {
            "Not set".bright_red().italic()
        } else {
            "Set ✓".bright_green()
        };
        println!("  {} {} {}", "🔒".yellow(), api_key_label, api_key_value);

        // Model with sparkle emoji
        let model_label = "Model:".yellow().bold();
        let model_value = provider_config.model.bright_cyan();
        println!("  {} {} {}", "✨".yellow(), model_label, model_value);

        // Token limit with gauge emoji
        let token_limit_label = "Token Limit:".yellow().bold();
        let token_limit_value = provider_config
            .token_limit
            .map_or("Default".bright_yellow(), |limit| {
                limit.to_string().bright_white()
            });
        println!(
            "  {} {} {}",
            "🔢".yellow(),
            token_limit_label,
            token_limit_value
        );

        // Additional parameters if any
        if !provider_config.additional_params.is_empty() {
            let params_label = "Additional Parameters:".yellow().bold();
            println!("  {} {}", "🔧".yellow(), params_label);

            for (key, value) in &provider_config.additional_params {
                println!("    - {}: {}", key.bright_blue(), value.bright_white());
            }
        }

        println!();
    }
}

/// Parse additional parameters from the command line
fn parse_additional_params(params: &[String]) -> HashMap<String, String> {
    params
        .iter()
        .filter_map(|param| {
            let parts: Vec<&str> = param.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Handle the '`list_presets`' command
pub fn handle_list_presets_command() -> Result<()> {
    let library = get_instruction_preset_library();

    // Get different categories of presets
    let both_presets = list_presets_formatted_by_type(&library, Some(PresetType::Both));
    let commit_only_presets = list_presets_formatted_by_type(&library, Some(PresetType::Commit));
    let review_only_presets = list_presets_formatted_by_type(&library, Some(PresetType::Review));

    println!(
        "{}",
        "\nGit-Iris Instruction Presets\n".bright_magenta().bold()
    );

    println!(
        "{}",
        "General Presets (usable for both commit and review):"
            .bright_cyan()
            .bold()
    );
    println!("{both_presets}\n");

    if !commit_only_presets.is_empty() {
        println!("{}", "Commit-specific Presets:".bright_green().bold());
        println!("{commit_only_presets}\n");
    }

    if !review_only_presets.is_empty() {
        println!("{}", "Review-specific Presets:".bright_blue().bold());
        println!("{review_only_presets}\n");
    }

    println!("{}", "Usage:".bright_yellow().bold());
    println!("  git-iris gen --preset <preset-key>");
    println!("  git-iris review --preset <preset-key>");
    println!("\nPreset types: [B] = Both commands, [C] = Commit only, [R] = Review only");

    Ok(())
}
