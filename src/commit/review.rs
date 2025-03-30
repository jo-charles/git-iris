use super::service::IrisCommitService;
use crate::common::CommonParams;
use crate::config::Config;
use crate::context::QualityDimension;
use crate::instruction_presets::PresetType;
use crate::messages;
use crate::ui;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Handles the review command which generates an AI code review of staged changes
/// with comprehensive analysis across multiple dimensions of code quality
pub async fn handle_review_command(common: CommonParams, _print: bool) -> Result<()> {
    // Check if the preset is appropriate for code reviews
    if !common.is_valid_preset_for_type(PresetType::Review) {
        ui::print_warning(
            "The specified preset may not be suitable for code reviews. Consider using a review or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for reviews.");
    }

    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;
    let current_dir = std::env::current_dir()?;

    let provider_name = &config.default_provider;

    let service = Arc::new(
        IrisCommitService::new(
            config.clone(),
            &current_dir.clone(),
            provider_name,
            false, // gitmoji not needed for review
            false, // verification not needed for review
        )
        .context("Failed to create IrisCommitService")?,
    );

    // Check environment prerequisites
    if let Err(e) = service.check_environment() {
        ui::print_error(&format!("Error: {e}"));
        ui::print_info("\nPlease ensure the following:");
        ui::print_info("1. Git is installed and accessible from the command line.");
        ui::print_info("2. You are running this command from within a Git repository.");
        ui::print_info("3. You have set up your configuration using 'git-iris config'.");
        return Err(e);
    }

    let git_info = service.get_git_info().await?;

    if git_info.staged_files.is_empty() {
        ui::print_warning(
            "No staged changes. Please stage your changes before generating a review.",
        );
        ui::print_info("You can stage changes using 'git add <file>' or 'git add .'");
        return Ok(());
    }

    let effective_instructions = common
        .instructions
        .unwrap_or_else(|| config.instructions.clone());
    let preset_str = common.preset.as_deref().unwrap_or("");

    // Create and start the spinner
    let spinner = ui::create_spinner("");
    let random_message = messages::get_review_waiting_message();
    spinner.set_message(format!("{}", random_message.text));

    // Generate the code review
    let review = service
        .generate_review(preset_str, &effective_instructions)
        .await?;

    // Stop the spinner
    spinner.finish_and_clear();

    // Print information about the enhanced review
    ui::print_info("\n✨ Enhanced Code Review ✨");
    ui::print_info("This review analyzes your code across multiple dimensions:");

    // Show all dimensions using the enum
    for dimension in QualityDimension::all() {
        ui::print_info(&format!(" • {}", dimension.display_name()));
    }
    println!();

    // Print the review to stdout or save to file if requested
    println!("{}", review.format());

    Ok(())
}
