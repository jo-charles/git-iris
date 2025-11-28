use crate::common::CommonParams;
use crate::config::Config;
use crate::instruction_presets::{
    PresetType, get_instruction_preset_library, list_presets_formatted_by_type,
};
use crate::log_debug;
use crate::providers::{Provider, ProviderConfig};
use crate::ui;
use anyhow::Context;
use anyhow::{Result, anyhow};
use colored::Colorize;
use std::collections::HashMap;

/// Helper to get themed colors for terminal output
mod colors {
    use crate::theme;

    pub fn accent_primary() -> (u8, u8, u8) {
        let c = theme::current().color("accent.primary");
        (c.r, c.g, c.b)
    }

    pub fn accent_secondary() -> (u8, u8, u8) {
        let c = theme::current().color("accent.secondary");
        (c.r, c.g, c.b)
    }

    pub fn accent_tertiary() -> (u8, u8, u8) {
        let c = theme::current().color("accent.tertiary");
        (c.r, c.g, c.b)
    }

    pub fn warning() -> (u8, u8, u8) {
        let c = theme::current().color("warning");
        (c.r, c.g, c.b)
    }

    pub fn success() -> (u8, u8, u8) {
        let c = theme::current().color("success");
        (c.r, c.g, c.b)
    }

    pub fn text_secondary() -> (u8, u8, u8) {
        let c = theme::current().color("text.secondary");
        (c.r, c.g, c.b)
    }

    pub fn text_dim() -> (u8, u8, u8) {
        let c = theme::current().color("text.dim");
        (c.r, c.g, c.b)
    }
}

/// Apply common configuration changes to a config object
/// Returns true if any changes were made
///
/// This centralized function handles changes to configuration objects, used by both
/// personal and project configuration commands.
///
/// # Arguments
///
/// * `config` - The configuration object to modify
/// * `common` - Common parameters from command line
/// * `model` - Optional model to set for the selected provider
/// * `token_limit` - Optional token limit to set
/// * `param` - Optional additional parameters to set
/// * `api_key` - Optional API key to set (ignored in project configs)
///
/// # Returns
///
/// Boolean indicating if any changes were made to the configuration
fn apply_config_changes(
    config: &mut Config,
    common: &CommonParams,
    model: Option<String>,
    fast_model: Option<String>,
    token_limit: Option<usize>,
    param: Option<Vec<String>>,
    api_key: Option<String>,
) -> anyhow::Result<bool> {
    let mut changes_made = false;

    // Apply common parameters to the config and track if changes were made
    let common_changes = common.apply_to_config(config)?;
    changes_made |= common_changes;

    // Handle provider change - validate and insert if needed
    if let Some(provider_str) = &common.provider {
        let provider: Provider = provider_str.parse().map_err(|_| {
            anyhow!(
                "Invalid provider: {}. Available: {}",
                provider_str,
                Provider::all_names().join(", ")
            )
        })?;

        // Only check for provider insertion if it wasn't already handled
        if !config.providers.contains_key(provider.name()) {
            config.providers.insert(
                provider.name().to_string(),
                ProviderConfig::with_defaults(provider),
            );
            changes_made = true;
        }
    }

    let provider_config = config
        .providers
        .get_mut(&config.default_provider)
        .context("Could not get default provider")?;

    // Apply API key if provided
    if let Some(key) = api_key
        && provider_config.api_key != key
    {
        provider_config.api_key = key;
        changes_made = true;
    }

    // Apply model change
    if let Some(model) = model
        && provider_config.model != model
    {
        provider_config.model = model;
        changes_made = true;
    }

    // Apply fast model change
    if let Some(fast_model) = fast_model
        && provider_config.fast_model != Some(fast_model.clone())
    {
        provider_config.fast_model = Some(fast_model);
        changes_made = true;
    }

    // Apply parameter changes
    if let Some(params) = param {
        let additional_params = parse_additional_params(&params);
        if provider_config.additional_params != additional_params {
            provider_config.additional_params = additional_params;
            changes_made = true;
        }
    }

    // Apply gitmoji setting
    if let Some(use_gitmoji) = common.gitmoji
        && config.use_gitmoji != use_gitmoji
    {
        config.use_gitmoji = use_gitmoji;
        changes_made = true;
    }

    // Apply instructions
    if let Some(instr) = &common.instructions
        && config.instructions != *instr
    {
        config.instructions.clone_from(instr);
        changes_made = true;
    }

    // Apply token limit
    if let Some(limit) = token_limit
        && provider_config.token_limit != Some(limit)
    {
        provider_config.token_limit = Some(limit);
        changes_made = true;
    }

    // Apply preset
    if let Some(preset) = &common.preset {
        let preset_library = get_instruction_preset_library();
        if preset_library.get_preset(preset).is_some() {
            if config.instruction_preset != *preset {
                config.instruction_preset.clone_from(preset);
                changes_made = true;
            }
        } else {
            return Err(anyhow!("Invalid preset: {}", preset));
        }
    }

    Ok(changes_made)
}

/// Handle the 'config' command
#[allow(clippy::too_many_lines)]
pub fn handle_config_command(
    common: &CommonParams,
    api_key: Option<String>,
    model: Option<String>,
    fast_model: Option<String>,
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

    // Apply configuration changes
    let changes_made = apply_config_changes(
        &mut config,
        common,
        model,
        fast_model,
        token_limit,
        param,
        api_key,
    )?;

    if changes_made {
        config.save()?;
        ui::print_success("Configuration updated successfully.");
        ui::print_newline();
    }

    // Print the configuration with beautiful styling
    print_configuration(&config);

    Ok(())
}

/// Process and apply configuration changes to a config object for project configs
///
/// This is a specialized wrapper around the `apply_config_changes` function that ensures
/// API keys are never passed to project configuration files.
///
/// # Arguments
///
/// * `config` - The configuration object to modify
/// * `common` - Common parameters from command line
/// * `model` - Optional model to set for the selected provider
/// * `token_limit` - Optional token limit to set
/// * `param` - Optional additional parameters to set
///
/// # Returns
///
/// Boolean indicating if any changes were made to the configuration
fn apply_project_config_changes(
    config: &mut Config,
    common: &CommonParams,
    model: Option<String>,
    fast_model: Option<String>,
    token_limit: Option<usize>,
    param: Option<Vec<String>>,
) -> anyhow::Result<bool> {
    // Use the shared function but don't pass an API key (never stored in project configs)
    apply_config_changes(config, common, model, fast_model, token_limit, param, None)
}

/// Handle printing current project configuration
///
/// Loads and displays the current project configuration if it exists,
/// or shows a message if no project configuration is found.
fn print_project_config() {
    if let Ok(project_config) = Config::load_project_config() {
        ui::print_message(&format!(
            "\n{}",
            "Current project configuration:".bright_cyan().bold()
        ));
        print_configuration(&project_config);
    } else {
        ui::print_message(&format!(
            "\n{}",
            "No project configuration file found.".yellow()
        ));
        ui::print_message("You can create one with the project-config command.");
    }
}

/// Handle the 'project-config' command
///
/// Creates or updates a project-specific configuration file (.irisconfig)
/// in the repository root. Project configurations allow teams to share
/// common settings without sharing sensitive data like API keys.
///
/// # Security
///
/// API keys are never stored in project configuration files, ensuring that
/// sensitive credentials are not accidentally committed to version control.
///
/// # Arguments
///
/// * `common` - Common parameters from command line
/// * `model` - Optional model to set for the selected provider
/// * `token_limit` - Optional token limit to set  
/// * `param` - Optional additional parameters to set
/// * `print` - Whether to just print the current project config
///
/// # Returns
///
/// Result indicating success or an error
pub fn handle_project_config_command(
    common: &CommonParams,
    model: Option<String>,
    fast_model: Option<String>,
    token_limit: Option<usize>,
    param: Option<Vec<String>>,
    print: bool,
) -> anyhow::Result<()> {
    log_debug!(
        "Starting 'project-config' command with common: {:?}, model: {:?}, token_limit: {:?}, param: {:?}, print: {}",
        common,
        model,
        token_limit,
        param,
        print
    );

    // Load the global config first
    let mut config = Config::load()?;

    // Set up a header to explain what's happening
    println!("\n{}", "✨ Project Configuration".bright_magenta().bold());

    // If print-only mode, just display the current project config if it exists
    if print {
        print_project_config();
        return Ok(());
    }

    // Apply changes and track if any were made
    let changes_made =
        apply_project_config_changes(&mut config, common, model, fast_model, token_limit, param)?;

    if changes_made {
        // Save to project config file
        config.save_as_project_config()?;
        ui::print_success("Project configuration created/updated successfully.");
        println!();

        // Print a notice about API keys not being stored in project config
        println!(
            "{}",
            "Note: API keys are never stored in project configuration files."
                .yellow()
                .italic()
        );
        println!();

        // Print the newly created/updated config
        println!("{}", "Current project configuration:".bright_cyan().bold());
        print_configuration(&config);
    } else {
        println!("{}", "No changes made to project configuration.".yellow());
        println!();

        // Check if a project config exists and show it if found
        if let Ok(project_config) = Config::load_project_config() {
            println!("{}", "Current project configuration:".bright_cyan().bold());
            print_configuration(&project_config);
        } else {
            println!("{}", "No project configuration exists yet.".bright_yellow());
            println!(
                "{}",
                "Use this command with options like --model or --provider to create one."
                    .bright_white()
            );
        }
    }

    Ok(())
}

/// Display the configuration with `SilkCircuit` styling
fn print_configuration(config: &Config) {
    let purple = colors::accent_primary();
    let cyan = colors::accent_secondary();
    let coral = colors::accent_tertiary();
    let yellow = colors::warning();
    let green = colors::success();
    let dim = colors::text_secondary();
    let dim_sep = colors::text_dim();

    println!();
    println!(
        "{}  {}  {}",
        "━━━".truecolor(purple.0, purple.1, purple.2),
        "IRIS CONFIGURATION".truecolor(cyan.0, cyan.1, cyan.2).bold(),
        "━━━".truecolor(purple.0, purple.1, purple.2)
    );
    println!();

    // Global Settings
    print_section_header("GLOBAL");

    print_config_row("Provider", &config.default_provider, cyan, true);
    print_config_row(
        "Gitmoji",
        if config.use_gitmoji {
            "enabled"
        } else {
            "disabled"
        },
        if config.use_gitmoji { green } else { dim },
        false,
    );
    print_config_row("Preset", &config.instruction_preset, yellow, false);

    // Custom Instructions (if any)
    if !config.instructions.is_empty() {
        println!();
        print_section_header("INSTRUCTIONS");
        for line in config.instructions.lines() {
            println!("  {}", line.truecolor(dim.0, dim.1, dim.2).italic());
        }
    }

    // Show all configured providers (those with API keys), sorted alphabetically
    let mut providers: Vec<_> = config
        .providers
        .iter()
        .filter(|(_, cfg)| !cfg.api_key.is_empty())
        .collect();
    providers.sort_by_key(|(name, _)| name.as_str());

    for (provider_name, provider_config) in providers {
        println!();
        let is_active = provider_name == &config.default_provider;
        let header = if is_active {
            format!("{} ✦", provider_name.to_uppercase())
        } else {
            provider_name.to_uppercase()
        };
        print_section_header(&header);

        // Model
        print_config_row("Model", &provider_config.model, cyan, true);

        // Fast Model
        let fast_model = provider_config.fast_model.as_deref().unwrap_or("(default)");
        print_config_row("Fast Model", fast_model, cyan, false);

        // Token Limit
        if let Some(limit) = provider_config.token_limit {
            print_config_row("Token Limit", &limit.to_string(), coral, false);
        }

        // Additional Parameters
        if !provider_config.additional_params.is_empty() {
            println!(
                "  {} {}",
                "Params".truecolor(dim.0, dim.1, dim.2),
                "─".truecolor(dim_sep.0, dim_sep.1, dim_sep.2)
            );
            for (key, value) in &provider_config.additional_params {
                println!(
                    "    {} {} {}",
                    key.truecolor(cyan.0, cyan.1, cyan.2),
                    "→".truecolor(dim_sep.0, dim_sep.1, dim_sep.2),
                    value.truecolor(dim.0, dim.1, dim.2)
                );
            }
        }
    }

    println!();
    println!(
        "{}",
        "─".repeat(40).truecolor(dim_sep.0, dim_sep.1, dim_sep.2)
    );
    println!();
}

/// Print a section header in `SilkCircuit` style
fn print_section_header(name: &str) {
    let purple = colors::accent_primary();
    let dim_sep = colors::text_dim();
    println!(
        "{} {} {}",
        "─".truecolor(purple.0, purple.1, purple.2),
        name.truecolor(purple.0, purple.1, purple.2).bold(),
        "─"
            .repeat(30 - name.len().min(28))
            .truecolor(dim_sep.0, dim_sep.1, dim_sep.2)
    );
}

/// Print a config row with label and value
fn print_config_row(label: &str, value: &str, value_color: (u8, u8, u8), highlight: bool) {
    let dim = colors::text_secondary();
    let label_styled = format!("{label:>12}").truecolor(dim.0, dim.1, dim.2);

    let value_styled = if highlight {
        value
            .truecolor(value_color.0, value_color.1, value_color.2)
            .bold()
    } else {
        value.truecolor(value_color.0, value_color.1, value_color.2)
    };

    println!("{label_styled}  {value_styled}");
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
