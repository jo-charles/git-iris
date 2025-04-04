use super::format_commit_result;
use super::service::IrisCommitService;
use super::types::format_commit_message;
use crate::common::CommonParams;
use crate::config::Config;
use crate::git::GitRepo;
use crate::instruction_presets::PresetType;
use crate::messages;
use crate::tui::run_tui_commit;
use crate::ui;
use anyhow::{Context, Result};
use std::sync::Arc;

#[allow(clippy::fn_params_excessive_bools)] // its ok to use multiple bools here
#[allow(clippy::too_many_lines)] // Function is slightly over the line limit but still readable
pub async fn handle_gen_command(
    common: CommonParams,
    auto_commit: bool,
    use_gitmoji: bool,
    print: bool,
    verify: bool,
    repository_url: Option<String>,
) -> Result<()> {
    // Check if the preset is appropriate for commit messages
    if !common.is_valid_preset_for_type(PresetType::Commit) {
        ui::print_warning(
            "The specified preset may not be suitable for commit messages. Consider using a commit or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for commits.");
    }

    let mut config = Config::load()?;
    common.apply_to_config(&mut config)?;

    // Combine repository URL from CLI and CommonParams
    let repo_url = repository_url.or(common.repository_url.clone());

    // Create the git repository
    let git_repo = GitRepo::new_from_url(repo_url).context("Failed to create GitRepo")?;

    let repo_path = git_repo.repo_path().clone();
    let provider_name = &config.default_provider;

    let service = Arc::new(
        IrisCommitService::new(
            config.clone(),
            &repo_path,
            provider_name,
            use_gitmoji && config.use_gitmoji,
            verify,
            git_repo,
        )
        .context("Failed to create IrisCommitService")?,
    );

    // Check environment prerequisites
    if let Err(e) = service.check_environment() {
        ui::print_error(&format!("Error: {e}"));
        ui::print_info("\nPlease ensure the following:");
        ui::print_info("1. Git is installed and accessible from the command line.");
        ui::print_info(
            "2. You are running this command from within a Git repository or provide a repository URL with --repo.",
        );
        ui::print_info("3. You have set up your configuration using 'git-iris config'.");
        return Err(e);
    }

    let git_info = service.get_git_info().await?;

    if git_info.staged_files.is_empty() {
        ui::print_warning(
            "No staged changes. Please stage your changes before generating a commit message.",
        );
        ui::print_info("You can stage changes using 'git add <file>' or 'git add .'");
        return Ok(());
    }

    // Run pre-commit hook before we do anything else
    if let Err(e) = service.pre_commit() {
        ui::print_error(&format!("Pre-commit failed: {e}"));
        return Err(e);
    }

    let effective_instructions = common
        .instructions
        .unwrap_or_else(|| config.instructions.clone());
    let preset_str = common.preset.as_deref().unwrap_or("");

    // Create and start the spinner
    let spinner = ui::create_spinner("");
    let random_message = messages::get_waiting_message();
    spinner.set_message(random_message.text);

    // Generate an initial message
    let initial_message = service
        .generate_message(preset_str, &effective_instructions)
        .await?;

    // Stop the spinner
    spinner.finish_and_clear();

    if print {
        println!("{}", format_commit_message(&initial_message));
        return Ok(());
    }

    if auto_commit {
        // Only allow auto-commit for local repositories
        if service.is_remote_repository() {
            ui::print_error(
                "Cannot automatically commit to a remote repository. Use --print instead.",
            );
            return Err(anyhow::anyhow!(
                "Auto-commit not supported for remote repositories"
            ));
        }

        match service.perform_commit(&format_commit_message(&initial_message)) {
            Ok(result) => {
                let output =
                    format_commit_result(&result, &format_commit_message(&initial_message));
                println!("{output}");
            }
            Err(e) => {
                eprintln!("Failed to commit: {e}");
                return Err(e);
            }
        }
        return Ok(());
    }

    // Only allow interactive commit for local repositories
    if service.is_remote_repository() {
        ui::print_warning(
            "Interactive commit not available for remote repositories. Using print mode instead.",
        );
        println!("{}", format_commit_message(&initial_message));
        return Ok(());
    }

    run_tui_commit(
        vec![initial_message],
        effective_instructions,
        String::from(preset_str),
        git_info.user_name,
        git_info.user_email,
        service,
        config.use_gitmoji,
    )
    .await?;

    Ok(())
}
