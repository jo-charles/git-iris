use crate::commands;
use crate::common::CommonParams;
use crate::log_debug;
use crate::providers::Provider;
use crate::theme;
use crate::ui;
use clap::builder::{Styles, styling::AnsiColor};
use clap::{CommandFactory, Parser, Subcommand, crate_version};
use clap_complete::{Shell, generate};
use colored::Colorize;
use std::io;

/// Default log file path for debug output
pub const LOG_FILE: &str = "git-iris-debug.log";

/// CLI structure defining the available commands and global arguments
#[derive(Parser)]
#[command(
    author,
    version = crate_version!(),
    about = "Git-Iris: AI-powered Git workflow assistant",
    long_about = "Git-Iris enhances your Git workflow with AI-assisted commit messages, code reviews, changelogs, and more.",
    disable_version_flag = true,
    after_help = get_dynamic_help(),
    styles = get_styles(),
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Subcommands available for the CLI
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Log debug messages to a file
    #[arg(
        short = 'l',
        long = "log",
        global = true,
        help = "Log debug messages to a file"
    )]
    pub log: bool,

    /// Specify a custom log file path
    #[arg(
        long = "log-file",
        global = true,
        help = "Specify a custom log file path"
    )]
    pub log_file: Option<String>,

    /// Suppress non-essential output (spinners, waiting messages, etc.)
    #[arg(
        short = 'q',
        long = "quiet",
        global = true,
        help = "Suppress non-essential output"
    )]
    pub quiet: bool,

    /// Display the version
    #[arg(
        short = 'v',
        long = "version",
        global = true,
        help = "Display the version"
    )]
    pub version: bool,

    /// Repository URL to use instead of local repository
    #[arg(
        short = 'r',
        long = "repo",
        global = true,
        help = "Repository URL to use instead of local repository"
    )]
    pub repository_url: Option<String>,

    /// Enable debug mode for detailed agent observability
    #[arg(
        long = "debug",
        global = true,
        help = "Enable debug mode with gorgeous color-coded output showing agent execution details"
    )]
    pub debug: bool,

    /// Override the theme for this session
    #[arg(
        long = "theme",
        global = true,
        help = "Override theme for this session (use 'git-iris themes' to list available)"
    )]
    pub theme: Option<String>,
}

/// Enumeration of available subcommands
#[derive(Subcommand)]
#[command(subcommand_negates_reqs = true)]
#[command(subcommand_precedence_over_arg = true)]
pub enum Commands {
    // Feature commands first
    /// Generate a commit message using AI
    #[command(
        about = "Generate a commit message using AI",
        long_about = "Generate a commit message using AI based on the current Git context.",
        after_help = get_dynamic_help()
    )]
    Gen {
        #[command(flatten)]
        common: CommonParams,

        /// Automatically commit with the generated message
        #[arg(short, long, help = "Automatically commit with the generated message")]
        auto_commit: bool,

        /// Disable Gitmoji for this commit
        #[arg(long, help = "Disable Gitmoji for this commit")]
        no_gitmoji: bool,

        /// Print the generated message to stdout and exit
        #[arg(short, long, help = "Print the generated message to stdout and exit")]
        print: bool,

        /// Skip the verification step (pre/post commit hooks)
        #[arg(long, help = "Skip verification steps (pre/post commit hooks)")]
        no_verify: bool,

        /// Amend the previous commit instead of creating a new one
        #[arg(long, help = "Amend the previous commit with staged changes")]
        amend: bool,
    },

    /// Review staged changes and provide feedback
    #[command(
        about = "Review staged changes using AI",
        long_about = "Generate a comprehensive multi-dimensional code review of staged changes using AI. Analyzes code across 10 dimensions including complexity, security, performance, and more."
    )]
    Review {
        #[command(flatten)]
        common: CommonParams,

        /// Print the generated review to stdout and exit
        #[arg(short, long, help = "Print the generated review to stdout and exit")]
        print: bool,

        /// Output raw markdown without any console formatting
        #[arg(long, help = "Output raw markdown without any console formatting")]
        raw: bool,

        /// Include unstaged changes in the review
        #[arg(long, help = "Include unstaged changes in the review")]
        include_unstaged: bool,

        /// Review a specific commit by ID (hash, branch, or reference)
        #[arg(
            long,
            help = "Review a specific commit by ID (hash, branch, or reference)"
        )]
        commit: Option<String>,

        /// Starting branch for comparison (defaults to 'main')
        #[arg(
            long,
            help = "Starting branch for comparison (defaults to 'main'). Used with --to for branch comparison reviews"
        )]
        from: Option<String>,

        /// Target branch for comparison (e.g., 'feature-branch', 'pr-branch')
        #[arg(
            long,
            help = "Target branch for comparison (e.g., 'feature-branch', 'pr-branch'). Used with --from for branch comparison reviews"
        )]
        to: Option<String>,
    },

    /// Generate a pull request description
    #[command(
        about = "Generate a pull request description using AI",
        long_about = "Generate a comprehensive pull request description based on commit ranges, branch differences, or single commits. Analyzes the overall changeset as an atomic unit and creates professional PR descriptions with summaries, detailed explanations, and testing notes.\n\nUsage examples:\n• Single commit: --from abc1234 or --to abc1234\n• Single commitish: --from HEAD~1 or --to HEAD~2\n• Multiple commits: --from HEAD~3 (reviews last 3 commits)\n• Commit range: --from abc1234 --to def5678\n• Branch comparison: --from main --to feature-branch\n• From main to branch: --to feature-branch\n\nSupported commitish syntax: HEAD~2, HEAD^, @~3, main~1, origin/main^, etc."
    )]
    Pr {
        #[command(flatten)]
        common: CommonParams,

        /// Print the generated PR description to stdout and exit
        #[arg(
            short,
            long,
            help = "Print the generated PR description to stdout and exit"
        )]
        print: bool,

        /// Output raw markdown without any console formatting
        #[arg(long, help = "Output raw markdown without any console formatting")]
        raw: bool,

        /// Starting branch, commit, or commitish for comparison
        #[arg(
            long,
            help = "Starting branch, commit, or commitish for comparison. For single commit analysis, specify just this parameter with a commit hash (e.g., --from abc1234). For reviewing multiple commits, use commitish syntax (e.g., --from HEAD~3 to review last 3 commits)"
        )]
        from: Option<String>,

        /// Target branch, commit, or commitish for comparison
        #[arg(
            long,
            help = "Target branch, commit, or commitish for comparison. For single commit analysis, specify just this parameter with a commit hash or commitish (e.g., --to HEAD~2)"
        )]
        to: Option<String>,
    },

    /// Generate a changelog
    #[command(
        about = "Generate a changelog",
        long_about = "Generate a changelog between two specified Git references."
    )]
    Changelog {
        #[command(flatten)]
        common: CommonParams,

        /// Starting Git reference (commit hash, tag, or branch name)
        #[arg(long, required = true)]
        from: String,

        /// Ending Git reference (commit hash, tag, or branch name). Defaults to HEAD if not specified.
        #[arg(long)]
        to: Option<String>,

        /// Output raw markdown without any console formatting
        #[arg(long, help = "Output raw markdown without any console formatting")]
        raw: bool,

        /// Update the changelog file with the new changes
        #[arg(long, help = "Update the changelog file with the new changes")]
        update: bool,

        /// Path to the changelog file
        #[arg(long, help = "Path to the changelog file (defaults to CHANGELOG.md)")]
        file: Option<String>,

        /// Explicit version name to use in the changelog instead of getting it from Git
        #[arg(long, help = "Explicit version name to use in the changelog")]
        version_name: Option<String>,
    },

    /// Generate release notes
    #[command(
        about = "Generate release notes",
        long_about = "Generate comprehensive release notes between two specified Git references."
    )]
    ReleaseNotes {
        #[command(flatten)]
        common: CommonParams,

        /// Starting Git reference (commit hash, tag, or branch name)
        #[arg(long, required = true)]
        from: String,

        /// Ending Git reference (commit hash, tag, or branch name). Defaults to HEAD if not specified.
        #[arg(long)]
        to: Option<String>,

        /// Output raw markdown without any console formatting
        #[arg(long, help = "Output raw markdown without any console formatting")]
        raw: bool,

        /// Update the release notes file with the new content
        #[arg(long, help = "Update the release notes file with the new content")]
        update: bool,

        /// Path to the release notes file
        #[arg(
            long,
            help = "Path to the release notes file (defaults to RELEASE_NOTES.md)"
        )]
        file: Option<String>,

        /// Explicit version name to use in the release notes instead of getting it from Git
        #[arg(long, help = "Explicit version name to use in the release notes")]
        version_name: Option<String>,
    },

    /// Launch Iris Studio - unified TUI for all operations
    #[command(
        about = "Launch Iris Studio TUI",
        long_about = "Launch Iris Studio, a unified terminal user interface for exploring code, generating commits, reviewing changes, and more. The interface adapts to your repository state."
    )]
    Studio {
        #[command(flatten)]
        common: CommonParams,

        /// Initial mode to launch in
        #[arg(
            long,
            value_name = "MODE",
            help = "Initial mode: explore, commit, review, pr, changelog"
        )]
        mode: Option<String>,

        /// Starting ref for PR/changelog comparison (defaults to main/master)
        #[arg(long, value_name = "REF", help = "Starting ref for comparison")]
        from: Option<String>,

        /// Ending ref for PR/changelog comparison (defaults to HEAD)
        #[arg(long, value_name = "REF", help = "Ending ref for comparison")]
        to: Option<String>,
    },

    // Configuration and utility commands
    /// Configure the AI-assisted Git commit message generator
    #[command(about = "Configure Git-Iris settings and providers")]
    Config {
        #[command(flatten)]
        common: CommonParams,

        /// Set API key for the specified provider
        #[arg(long, help = "Set API key for the specified provider")]
        api_key: Option<String>,

        /// Set model for the specified provider
        #[arg(long, help = "Set model for the specified provider")]
        model: Option<String>,

        /// Set fast model for the specified provider (used for status updates and simple tasks)
        #[arg(
            long,
            help = "Set fast model for the specified provider (used for status updates and simple tasks)"
        )]
        fast_model: Option<String>,

        /// Set token limit for the specified provider
        #[arg(long, help = "Set token limit for the specified provider")]
        token_limit: Option<usize>,

        /// Set additional parameters for the specified provider
        #[arg(
            long,
            help = "Set additional parameters for the specified provider (key=value)"
        )]
        param: Option<Vec<String>>,

        /// Set timeout in seconds for parallel subagent tasks
        #[arg(
            long,
            help = "Set timeout in seconds for parallel subagent tasks (default: 120)"
        )]
        subagent_timeout: Option<u64>,
    },

    /// Create or update a project-specific configuration file
    #[command(
        about = "Manage project-specific configuration",
        long_about = "Create or update a project-specific .irisconfig file in the repository root."
    )]
    ProjectConfig {
        #[command(flatten)]
        common: CommonParams,

        /// Set model for the specified provider
        #[arg(long, help = "Set model for the specified provider")]
        model: Option<String>,

        /// Set fast model for the specified provider (used for status updates and simple tasks)
        #[arg(
            long,
            help = "Set fast model for the specified provider (used for status updates and simple tasks)"
        )]
        fast_model: Option<String>,

        /// Set token limit for the specified provider
        #[arg(long, help = "Set token limit for the specified provider")]
        token_limit: Option<usize>,

        /// Set additional parameters for the specified provider
        #[arg(
            long,
            help = "Set additional parameters for the specified provider (key=value)"
        )]
        param: Option<Vec<String>>,

        /// Set timeout in seconds for parallel subagent tasks
        #[arg(
            long,
            help = "Set timeout in seconds for parallel subagent tasks (default: 120)"
        )]
        subagent_timeout: Option<u64>,

        /// Print the current project configuration
        #[arg(short, long, help = "Print the current project configuration")]
        print: bool,
    },

    /// List available instruction presets
    #[command(about = "List available instruction presets")]
    ListPresets,

    /// List available themes
    #[command(about = "List available themes")]
    Themes,

    /// Generate shell completions
    #[command(
        about = "Generate shell completions",
        long_about = "Generate shell completion scripts for bash, zsh, fish, elvish, or powershell.\n\nUsage examples:\n• Bash: git-iris completions bash >> ~/.bashrc\n• Zsh:  git-iris completions zsh >> ~/.zshrc\n• Fish: git-iris completions fish > ~/.config/fish/completions/git-iris.fish"
    )]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Define custom styles for Clap
fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Magenta.on_default().bold())
        .usage(AnsiColor::Cyan.on_default().bold())
        .literal(AnsiColor::Green.on_default().bold())
        .placeholder(AnsiColor::Yellow.on_default())
        .valid(AnsiColor::Blue.on_default().bold())
        .invalid(AnsiColor::Red.on_default().bold())
        .error(AnsiColor::Red.on_default().bold())
}

/// Parse the command-line arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}

/// Generate dynamic help including available LLM providers
fn get_dynamic_help() -> String {
    let providers_list = Provider::all_names()
        .iter()
        .map(|p| format!("{}", p.bold()))
        .collect::<Vec<_>>()
        .join(" • ");

    format!("\nAvailable LLM Providers: {providers_list}")
}

/// Main function to parse arguments and handle the command
pub async fn main() -> anyhow::Result<()> {
    let cli = parse_args();

    if cli.version {
        ui::print_version(crate_version!());
        return Ok(());
    }

    if cli.log {
        crate::logger::enable_logging();
        crate::logger::set_log_to_stdout(true);
        let log_file = cli.log_file.as_deref().unwrap_or(LOG_FILE);
        crate::logger::set_log_file(log_file)?;
        log_debug!("Debug logging enabled");
    } else {
        crate::logger::disable_logging();
    }

    // Set quiet mode in the UI module
    if cli.quiet {
        crate::ui::set_quiet_mode(true);
    }

    // Initialize theme
    initialize_theme(cli.theme.as_deref());

    // Enable debug mode if requested
    if cli.debug {
        crate::agents::debug::enable_debug_mode();
        crate::agents::debug::debug_header("🔮 IRIS DEBUG MODE ACTIVATED 🔮");
    }

    if let Some(command) = cli.command {
        handle_command(command, cli.repository_url).await
    } else {
        // Default: launch Studio with auto-detect mode
        handle_studio(
            CommonParams::default(),
            None,
            None,
            None,
            cli.repository_url,
        )
        .await
    }
}

/// Initialize the theme from CLI flag or config
fn initialize_theme(cli_theme: Option<&str>) {
    use crate::config::Config;

    // CLI flag takes precedence
    let theme_name = if let Some(name) = cli_theme {
        Some(name.to_string())
    } else {
        // Try to load from config
        Config::load().ok().and_then(|c| {
            if c.theme.is_empty() {
                None
            } else {
                Some(c.theme)
            }
        })
    };

    // Load the theme if specified, otherwise default is already active
    if let Some(name) = theme_name {
        if let Err(e) = theme::load_theme_by_name(&name) {
            ui::print_warning(&format!(
                "Failed to load theme '{}': {}. Using default.",
                name, e
            ));
        } else {
            log_debug!("Loaded theme: {}", name);
        }
    }
}

/// Configuration for the Gen command
#[allow(clippy::struct_excessive_bools)]
struct GenConfig {
    auto_commit: bool,
    use_gitmoji: bool,
    print_only: bool,
    verify: bool,
    amend: bool,
}

/// Handle the `Gen` command with agent framework and Studio integration
#[allow(clippy::too_many_lines)]
async fn handle_gen_with_agent(
    common: CommonParams,
    config: GenConfig,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    use crate::agents::{IrisAgentService, StructuredResponse, TaskContext};
    use crate::config::Config;
    use crate::git::GitRepo;
    use crate::instruction_presets::PresetType;
    use crate::output::format_commit_result;
    use crate::services::GitCommitService;
    use crate::studio::{Mode, run_studio};
    use crate::types::format_commit_message;
    use anyhow::Context;
    use std::sync::Arc;

    // Check if the preset is appropriate for commit messages
    if !common.is_valid_preset_for_type(PresetType::Commit) {
        ui::print_warning(
            "The specified preset may not be suitable for commit messages. Consider using a commit or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for commits.");
    }

    // Amend mode requires --print or --auto-commit (Studio amend support coming later)
    if config.amend && !config.print_only && !config.auto_commit {
        ui::print_warning("--amend requires --print or --auto-commit for now.");
        ui::print_info("Example: git-iris gen --amend --auto-commit");
        return Ok(());
    }

    let mut cfg = Config::load()?;
    common.apply_to_config(&mut cfg)?;

    // Create git repo and services
    let repo_url = repository_url.clone().or(common.repository_url.clone());
    let git_repo = Arc::new(GitRepo::new_from_url(repo_url).context("Failed to create GitRepo")?);
    let use_gitmoji = config.use_gitmoji && cfg.use_gitmoji;

    // Create GitCommitService for commit operations
    let commit_service = Arc::new(GitCommitService::new(
        git_repo.clone(),
        use_gitmoji,
        config.verify,
    ));

    // Create IrisAgentService for LLM operations
    let agent_service = Arc::new(IrisAgentService::from_common_params(
        &common,
        repository_url.clone(),
    )?);

    // Get git info for staged files check
    let git_info = git_repo.get_git_info(&cfg)?;

    // For --print or --auto-commit, we need to generate the message first
    if config.print_only || config.auto_commit {
        // For amend mode, we allow empty staged changes (amending message only)
        // For regular commits, we require staged changes
        if git_info.staged_files.is_empty() && !config.amend {
            ui::print_warning(
                "No staged changes. Please stage your changes before generating a commit message.",
            );
            ui::print_info("You can stage changes using 'git add <file>' or 'git add .'");
            return Ok(());
        }

        // Run pre-commit hook before we do anything else
        if let Err(e) = commit_service.pre_commit() {
            ui::print_error(&format!("Pre-commit failed: {e}"));
            return Err(e);
        }

        // Create spinner for agent mode
        let spinner_msg = if config.amend {
            "Generating amended commit message..."
        } else {
            "Generating commit message..."
        };
        let spinner = ui::create_spinner(spinner_msg);

        // Use IrisAgentService for commit message generation
        // For amend, we pass the original message as context
        let context = if config.amend {
            let original_message = commit_service.get_head_commit_message().unwrap_or_default();
            TaskContext::for_amend(original_message)
        } else {
            TaskContext::for_gen()
        };
        let response = agent_service.execute_task("commit", context).await?;

        // Extract commit message from response
        let StructuredResponse::CommitMessage(generated_message) = response else {
            return Err(anyhow::anyhow!("Expected commit message response"));
        };

        // Finish spinner after agent completes
        spinner.finish_and_clear();

        if config.print_only {
            println!("{}", format_commit_message(&generated_message));
            return Ok(());
        }

        // Auto-commit/amend mode
        if commit_service.is_remote() {
            ui::print_error(
                "Cannot automatically commit to a remote repository. Use --print instead.",
            );
            return Err(anyhow::anyhow!(
                "Auto-commit not supported for remote repositories"
            ));
        }

        let commit_result = if config.amend {
            commit_service.perform_amend(&format_commit_message(&generated_message))
        } else {
            commit_service.perform_commit(&format_commit_message(&generated_message))
        };

        match commit_result {
            Ok(result) => {
                let output =
                    format_commit_result(&result, &format_commit_message(&generated_message));
                println!("{output}");
            }
            Err(e) => {
                let action = if config.amend { "amend" } else { "commit" };
                eprintln!("Failed to {action}: {e}");
                return Err(e);
            }
        }
        return Ok(());
    }

    // Interactive mode: launch Studio (it handles staged check and auto-generation)
    if commit_service.is_remote() {
        ui::print_warning(
            "Interactive commit not available for remote repositories. Use --print instead.",
        );
        return Ok(());
    }

    // Launch Studio in Commit mode - it will auto-generate if there are staged changes
    run_studio(
        cfg,
        Some(git_repo),
        Some(commit_service),
        Some(agent_service),
        Some(Mode::Commit),
        None,
        None,
    )
}

/// Handle the `Gen` command
async fn handle_gen(
    common: CommonParams,
    config: GenConfig,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'gen' command with common: {:?}, auto_commit: {}, use_gitmoji: {}, print: {}, verify: {}, amend: {}",
        common,
        config.auto_commit,
        config.use_gitmoji,
        config.print_only,
        config.verify,
        config.amend
    );

    ui::print_version(crate_version!());
    ui::print_newline();

    handle_gen_with_agent(common, config, repository_url).await
}

/// Handle the `Config` command
fn handle_config(
    common: &CommonParams,
    api_key: Option<String>,
    model: Option<String>,
    fast_model: Option<String>,
    token_limit: Option<usize>,
    param: Option<Vec<String>>,
    subagent_timeout: Option<u64>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'config' command with common: {:?}, api_key: {:?}, model: {:?}, token_limit: {:?}, param: {:?}, subagent_timeout: {:?}",
        common,
        api_key,
        model,
        token_limit,
        param,
        subagent_timeout
    );
    commands::handle_config_command(
        common,
        api_key,
        model,
        fast_model,
        token_limit,
        param,
        subagent_timeout,
    )
}

/// Handle the `Review` command
#[allow(clippy::too_many_arguments)]
async fn handle_review(
    common: CommonParams,
    print: bool,
    raw: bool,
    repository_url: Option<String>,
    include_unstaged: bool,
    commit: Option<String>,
    from: Option<String>,
    to: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'review' command with common: {:?}, print: {}, raw: {}, include_unstaged: {}, commit: {:?}, from: {:?}, to: {:?}",
        common,
        print,
        raw,
        include_unstaged,
        commit,
        from,
        to
    );

    // For raw output, skip all formatting
    if !raw {
        ui::print_version(crate_version!());
        ui::print_newline();
    }

    use crate::agents::{IrisAgentService, TaskContext};

    // Validate parameters and create structured context
    let context = TaskContext::for_review(commit, from, to, include_unstaged)?;

    // Create spinner for progress indication (skip for raw output)
    let spinner = if raw {
        None
    } else {
        Some(ui::create_spinner("Initializing Iris..."))
    };

    // Use IrisAgentService for agent execution
    let service = IrisAgentService::from_common_params(&common, repository_url)?;
    let response = service.execute_task("review", context).await?;

    // Finish spinner
    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    if raw || print {
        println!("{response}");
    } else {
        ui::print_success("Code review completed successfully");
        println!("{response}");
    }
    Ok(())
}

/// Handle the `Changelog` command
#[allow(clippy::too_many_arguments)]
async fn handle_changelog(
    common: CommonParams,
    from: String,
    to: Option<String>,
    raw: bool,
    repository_url: Option<String>,
    update: bool,
    file: Option<String>,
    version_name: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'changelog' command with common: {:?}, from: {}, to: {:?}, raw: {}, update: {}, file: {:?}, version_name: {:?}",
        common,
        from,
        to,
        raw,
        update,
        file,
        version_name
    );

    // For raw output, skip all formatting
    if !raw {
        ui::print_version(crate_version!());
        ui::print_newline();
    }

    use crate::agents::{IrisAgentService, TaskContext};
    use crate::changelog::ChangelogGenerator;
    use crate::git::GitRepo;
    use anyhow::Context;
    use std::sync::Arc;

    // Create structured context for changelog
    let context = TaskContext::for_changelog(from.clone(), to.clone());
    let to_ref = to.unwrap_or_else(|| "HEAD".to_string());

    // Create spinner for progress indication (skip for raw output)
    let spinner = if raw {
        None
    } else {
        Some(ui::create_spinner("Initializing Iris..."))
    };

    // Use IrisAgentService for agent execution
    let service = IrisAgentService::from_common_params(&common, repository_url.clone())?;
    let response = service.execute_task("changelog", context).await?;

    // Finish spinner
    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    // Print the changelog
    println!("{response}");

    if update {
        // Extract the formatted content for file update
        let formatted_content = response.to_string();
        let changelog_path = file.unwrap_or_else(|| "CHANGELOG.md".to_string());
        let repo_url_for_update = repository_url.or(common.repository_url.clone());

        // Create GitRepo for file update
        let git_repo = if let Some(url) = repo_url_for_update {
            Arc::new(
                GitRepo::clone_remote_repository(&url)
                    .context("Failed to clone repository for changelog update")?,
            )
        } else {
            let repo_path = std::env::current_dir()?;
            Arc::new(
                GitRepo::new(&repo_path)
                    .context("Failed to create GitRepo for changelog update")?,
            )
        };

        // Update changelog file
        let update_spinner =
            ui::create_spinner(&format!("Updating changelog file at {changelog_path}..."));

        match ChangelogGenerator::update_changelog_file(
            &formatted_content,
            &changelog_path,
            &git_repo,
            &to_ref,
            version_name,
        ) {
            Ok(()) => {
                update_spinner.finish_and_clear();
                ui::print_success(&format!(
                    "✨ Changelog successfully updated at {}",
                    changelog_path.bright_green()
                ));
            }
            Err(e) => {
                update_spinner.finish_and_clear();
                ui::print_error(&format!("Failed to update changelog file: {e}"));
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Handle the `Release Notes` command
#[allow(clippy::too_many_arguments)]
async fn handle_release_notes(
    common: CommonParams,
    from: String,
    to: Option<String>,
    raw: bool,
    repository_url: Option<String>,
    update: bool,
    file: Option<String>,
    _version_name: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'release-notes' command with common: {:?}, from: {}, to: {:?}, raw: {}, update: {}, file: {:?}",
        common,
        from,
        to,
        raw,
        update,
        file
    );

    // For raw output, skip all formatting
    if !raw {
        ui::print_version(crate_version!());
        ui::print_newline();
    }

    use crate::agents::{IrisAgentService, TaskContext};
    use std::fs;
    use std::path::Path;

    // Create structured context for release notes
    let context = TaskContext::for_changelog(from, to);

    // Create spinner for progress indication (skip for raw output)
    let spinner = if raw {
        None
    } else {
        Some(ui::create_spinner("Initializing Iris..."))
    };

    // Use IrisAgentService for agent execution
    let service = IrisAgentService::from_common_params(&common, repository_url)?;
    let response = service.execute_task("release_notes", context).await?;

    // Finish spinner
    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    println!("{response}");

    // Handle --update flag
    if update {
        let release_notes_path = file.unwrap_or_else(|| "RELEASE_NOTES.md".to_string());
        let formatted_content = response.to_string();

        let update_spinner = ui::create_spinner(&format!(
            "Updating release notes file at {release_notes_path}..."
        ));

        // Write or append to file
        let path = Path::new(&release_notes_path);
        let result = if path.exists() {
            // Prepend to existing file
            let existing = fs::read_to_string(path)?;
            fs::write(path, format!("{formatted_content}\n\n---\n\n{existing}"))
        } else {
            // Create new file
            fs::write(path, &formatted_content)
        };

        match result {
            Ok(()) => {
                update_spinner.finish_and_clear();
                ui::print_success(&format!(
                    "✨ Release notes successfully updated at {}",
                    release_notes_path.bright_green()
                ));
            }
            Err(e) => {
                update_spinner.finish_and_clear();
                ui::print_error(&format!("Failed to update release notes file: {e}"));
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Handle the command based on parsed arguments
#[allow(clippy::too_many_lines)]
pub async fn handle_command(
    command: Commands,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    match command {
        Commands::Gen {
            common,
            auto_commit,
            no_gitmoji,
            print,
            no_verify,
            amend,
        } => {
            handle_gen(
                common,
                GenConfig {
                    auto_commit,
                    use_gitmoji: !no_gitmoji,
                    print_only: print,
                    verify: !no_verify,
                    amend,
                },
                repository_url,
            )
            .await
        }
        Commands::Config {
            common,
            api_key,
            model,
            fast_model,
            token_limit,
            param,
            subagent_timeout,
        } => handle_config(
            &common,
            api_key,
            model,
            fast_model,
            token_limit,
            param,
            subagent_timeout,
        ),
        Commands::Review {
            common,
            print,
            raw,
            include_unstaged,
            commit,
            from,
            to,
        } => {
            handle_review(
                common,
                print,
                raw,
                repository_url,
                include_unstaged,
                commit,
                from,
                to,
            )
            .await
        }
        Commands::Changelog {
            common,
            from,
            to,
            raw,
            update,
            file,
            version_name,
        } => {
            handle_changelog(
                common,
                from,
                to,
                raw,
                repository_url,
                update,
                file,
                version_name,
            )
            .await
        }
        Commands::ReleaseNotes {
            common,
            from,
            to,
            raw,
            update,
            file,
            version_name,
        } => {
            handle_release_notes(
                common,
                from,
                to,
                raw,
                repository_url,
                update,
                file,
                version_name,
            )
            .await
        }
        Commands::ProjectConfig {
            common,
            model,
            fast_model,
            token_limit,
            param,
            subagent_timeout,
            print,
        } => commands::handle_project_config_command(
            &common,
            model,
            fast_model,
            token_limit,
            param,
            subagent_timeout,
            print,
        ),
        Commands::ListPresets => commands::handle_list_presets_command(),
        Commands::Themes => {
            handle_themes();
            Ok(())
        }
        Commands::Completions { shell } => {
            handle_completions(shell);
            Ok(())
        }
        Commands::Pr {
            common,
            print,
            raw,
            from,
            to,
        } => handle_pr(common, print, raw, from, to, repository_url).await,
        Commands::Studio {
            common,
            mode,
            from,
            to,
        } => handle_studio(common, mode, from, to, repository_url).await,
    }
}

/// Handle the `Themes` command - list available themes
fn handle_themes() {
    ui::print_version(crate_version!());
    ui::print_newline();

    let available = theme::list_available_themes();
    let current = theme::current();
    let current_name = &current.meta.name;

    // Header
    let header_color = theme::current().color("accent.primary");
    println!(
        "{}",
        "Available Themes:"
            .truecolor(header_color.r, header_color.g, header_color.b)
            .bold()
    );
    println!();

    for info in available {
        let is_current = info.display_name == *current_name;
        let marker = if is_current { "● " } else { "  " };

        let name_color = if is_current {
            theme::current().color("success")
        } else {
            theme::current().color("accent.secondary")
        };

        let desc_color = theme::current().color("text.secondary");

        print!(
            "{}{}",
            marker.truecolor(name_color.r, name_color.g, name_color.b),
            info.name
                .truecolor(name_color.r, name_color.g, name_color.b)
                .bold()
        );

        // Show display name if different from filename
        if info.display_name != info.name {
            print!(
                " ({})",
                info.display_name
                    .truecolor(desc_color.r, desc_color.g, desc_color.b)
            );
        }

        // Show variant
        let variant_str = match info.variant {
            theme::ThemeVariant::Dark => "dark",
            theme::ThemeVariant::Light => "light",
        };
        let dim_color = theme::current().color("text.dim");
        print!(
            " [{}]",
            variant_str.truecolor(dim_color.r, dim_color.g, dim_color.b)
        );

        if is_current {
            let active_color = theme::current().color("success");
            print!(
                " {}",
                "(active)".truecolor(active_color.r, active_color.g, active_color.b)
            );
        }

        println!();
    }

    println!();

    // Usage hint
    let hint_color = theme::current().color("text.dim");
    println!(
        "{}",
        "Use --theme <name> to override, or set 'theme' in config.toml".truecolor(
            hint_color.r,
            hint_color.g,
            hint_color.b
        )
    );
}

/// Handle the `Completions` command - generate shell completion scripts
fn handle_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "git-iris", &mut io::stdout());
}

/// Handle the `Pr` command with agent framework
async fn handle_pr_with_agent(
    common: CommonParams,
    print: bool,
    raw: bool,
    from: Option<String>,
    to: Option<String>,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    use crate::agents::{IrisAgentService, StructuredResponse, TaskContext};
    use crate::instruction_presets::PresetType;

    // Check if the preset is appropriate for PR descriptions (skip for raw output)
    if !raw
        && !common.is_valid_preset_for_type(PresetType::Review)
        && !common.is_valid_preset_for_type(PresetType::Both)
    {
        ui::print_warning(
            "The specified preset may not be suitable for PR descriptions. Consider using a review or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for PRs.");
    }

    // Create structured context for PR (handles defaults: from=main, to=HEAD)
    let context = TaskContext::for_pr(from, to);

    // Create spinner for progress indication (skip for raw output)
    let spinner = if raw {
        None
    } else {
        Some(ui::create_spinner("Initializing Iris..."))
    };

    // Use IrisAgentService for agent execution
    let service = IrisAgentService::from_common_params(&common, repository_url)?;
    let response = service.execute_task("pr", context).await?;

    // Finish spinner
    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    // Extract PR from response
    let StructuredResponse::PullRequest(generated_pr) = response else {
        return Err(anyhow::anyhow!("Expected pull request response"));
    };

    if raw || print {
        println!("{}", generated_pr.format());
    } else {
        ui::print_success("PR description generated successfully");
        println!("{}", generated_pr.format());
    }

    Ok(())
}

/// Handle the `Pr` command
async fn handle_pr(
    common: CommonParams,
    print: bool,
    raw: bool,
    from: Option<String>,
    to: Option<String>,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'pr' command with common: {:?}, print: {}, raw: {}, from: {:?}, to: {:?}",
        common,
        print,
        raw,
        from,
        to
    );

    // For raw output, skip all formatting
    if !raw {
        ui::print_version(crate_version!());
        ui::print_newline();
    }

    handle_pr_with_agent(common, print, raw, from, to, repository_url).await
}

/// Handle the `Studio` command
#[allow(clippy::unused_async)] // Will need async when agent integration is complete
async fn handle_studio(
    common: CommonParams,
    mode: Option<String>,
    from: Option<String>,
    to: Option<String>,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    use crate::agents::IrisAgentService;
    use crate::config::Config;
    use crate::git::GitRepo;
    use crate::services::GitCommitService;
    use crate::studio::{Mode, run_studio};
    use anyhow::Context;
    use std::sync::Arc;

    // Disable stdout logging immediately for TUI mode - it owns the terminal
    crate::logger::set_log_to_stdout(false);

    log_debug!(
        "Handling 'studio' command with common: {:?}, mode: {:?}, from: {:?}, to: {:?}",
        common,
        mode,
        from,
        to
    );

    let mut cfg = Config::load()?;
    common.apply_to_config(&mut cfg)?;

    // Create git repo
    let repo_url = repository_url.clone().or(common.repository_url.clone());
    let git_repo =
        Arc::new(GitRepo::new_from_url(repo_url.clone()).context("Failed to create GitRepo")?);

    // Create services
    let commit_service = Arc::new(GitCommitService::new(
        git_repo.clone(),
        cfg.use_gitmoji,
        true, // verify hooks
    ));

    let agent_service = Arc::new(IrisAgentService::from_common_params(
        &common,
        repository_url,
    )?);

    // Parse initial mode
    let initial_mode = mode
        .as_deref()
        .and_then(|m| match m.to_lowercase().as_str() {
            "explore" => Some(Mode::Explore),
            "commit" => Some(Mode::Commit),
            "review" => Some(Mode::Review),
            "pr" => Some(Mode::PR),
            "changelog" => Some(Mode::Changelog),
            _ => {
                ui::print_warning(&format!("Unknown mode '{}', using auto-detect", m));
                None
            }
        });

    run_studio(
        cfg,
        Some(git_repo),
        Some(commit_service),
        Some(agent_service),
        initial_mode,
        from,
        to,
    )
}
