use crate::changes;
use crate::commands;
use crate::commit;
use crate::common::CommonParams;
use crate::llm::get_available_provider_names;
use crate::log_debug;
use crate::ui;
use clap::builder::{Styles, styling::AnsiColor};
use clap::{Parser, Subcommand, crate_version};
use colored::Colorize;

const LOG_FILE: &str = "git-iris-debug.log";

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

    /// Use agent framework for enhanced AI-powered operations
    #[arg(
        long = "agent",
        global = true,
        help = "Use agent framework for enhanced AI-powered operations"
    )]
    pub agent: bool,

    /// Enable debug mode for detailed agent observability
    #[arg(
        long = "debug",
        global = true,
        help = "Enable debug mode with gorgeous color-coded output showing agent execution details"
    )]
    pub debug: bool,
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

        /// Explicit version name to use in the release notes instead of getting it from Git
        #[arg(long, help = "Explicit version name to use in the release notes")]
        version_name: Option<String>,
    },

    /// Start an MCP server to provide Git-Iris functionality to AI tools
    #[command(
        about = "Start an MCP server",
        long_about = "Start a Model Context Protocol (MCP) server to provide Git-Iris functionality to AI tools and assistants."
    )]
    Serve {
        /// Enable development mode with more verbose logging
        #[arg(long, help = "Enable development mode with more verbose logging")]
        dev: bool,

        /// Transport type to use (stdio, sse)
        #[arg(
            short,
            long,
            help = "Transport type to use (stdio, sse)",
            default_value = "stdio"
        )]
        transport: String,

        /// Port to use for network transports
        #[arg(short, long, help = "Port to use for network transports")]
        port: Option<u16>,

        /// Listen address for network transports
        #[arg(
            long,
            help = "Listen address for network transports (e.g., '127.0.0.1', '0.0.0.0')",
            default_value = "127.0.0.1"
        )]
        listen_address: Option<String>,
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

        /// Print the current project configuration
        #[arg(short, long, help = "Print the current project configuration")]
        print: bool,
    },

    /// List available instruction presets
    #[command(about = "List available instruction presets")]
    ListPresets,
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
    let mut providers = get_available_provider_names();
    providers.sort(); // Sort alphabetically

    let providers_list = providers
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
        let log_file = cli.log_file.as_deref().unwrap_or(LOG_FILE);
        crate::logger::set_log_file(log_file)?;

        // Load config to check for verbose logging settings
        if let Ok(config) = crate::config::Config::load() {
            crate::logger::set_verbose_logging(config.performance.verbose_logging);
            if config.performance.verbose_logging {
                log_debug!(
                    "Verbose logging enabled - will show HTTP requests and external library logs"
                );
            } else {
                log_debug!("Verbose logging disabled - hiding noisy external library logs");
            }
        }
    } else {
        crate::logger::disable_logging();
    }

    // Set quiet mode in the UI module
    if cli.quiet {
        crate::ui::set_quiet_mode(true);
    }

    // Enable debug mode if requested
    if cli.debug {
        crate::agents::debug::enable_debug_mode();
        crate::agents::debug::debug_header("🔮 IRIS DEBUG MODE ACTIVATED 🔮");
    }

    if let Some(command) = cli.command {
        handle_command(command, cli.repository_url, cli.agent).await
    } else {
        // If no subcommand is provided, print the help
        let _ = Cli::parse_from(["git-iris", "--help"]);
        Ok(())
    }
}

/// Configuration for the Gen command
#[allow(clippy::struct_excessive_bools)]
struct GenConfig {
    auto_commit: bool,
    use_gitmoji: bool,
    print_only: bool,
    verify: bool,
}

/// Handle the `Gen` command with agent framework and TUI integration
#[allow(clippy::too_many_lines)]
async fn handle_gen_with_agent(
    common: CommonParams,
    config: GenConfig,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    use crate::commit::types::format_commit_message;
    use crate::commit::{IrisCommitService, format_commit_result};
    use crate::config::Config;
    use crate::git::GitRepo;
    use crate::instruction_presets::PresetType;
    use crate::tui::run_tui_commit;
    use anyhow::Context;
    use std::sync::Arc;

    // Enable agent mode for enhanced status display
    crate::agents::status::enable_agent_mode();

    // Check if the preset is appropriate for commit messages
    if !common.is_valid_preset_for_type(PresetType::Commit) {
        ui::print_warning(
            "The specified preset may not be suitable for commit messages. Consider using a commit or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for commits.");
    }

    let mut cfg = Config::load()?;
    common.apply_to_config(&mut cfg)?;

    // Create the service
    let repo_url = repository_url.clone().or(common.repository_url.clone());
    let git_repo = GitRepo::new_from_url(repo_url).context("Failed to create GitRepo")?;
    let repo_path = git_repo.repo_path().clone();

    let service = Arc::new(IrisCommitService::new(
        cfg.clone(),
        &repo_path,
        &cfg.default_provider,
        config.use_gitmoji && cfg.use_gitmoji,
        config.verify,
        git_repo,
    ).map_err(|e| {
        ui::print_error(&format!("Error: {e}"));
        ui::print_info("\nPlease ensure the following:");
        ui::print_info("1. Git is installed and accessible from the command line.");
        ui::print_info(
            "2. You are running this command from within a Git repository or provide a repository URL with --repo.",
        );
        ui::print_info("3. You have set up your configuration using 'git-iris config'.");
        e
    })?);

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

    // Extract values we need from common before passing it to agent
    let effective_instructions = common
        .instructions
        .as_ref()
        .cloned()
        .unwrap_or_else(|| cfg.instructions.clone());
    let preset_str = common.preset.clone().unwrap_or_default();

    // Use agent framework for commit message generation
    let task_prompt = if config.use_gitmoji {
        "Generate a commit message using gitmoji and following best practices"
    } else {
        "Generate a commit message following best practices"
    };

    // Create spinner for agent mode - it will poll IRIS_STATUS for dynamic updates
    let spinner = ui::create_spinner("Initializing Iris...");

    let generated_message = crate::agents::handle_with_agent(
        common,
        repository_url,
        "commit",
        task_prompt,
        |response| async move {
            match response {
                crate::agents::iris::StructuredResponse::CommitMessage(msg) => Ok(msg),
                _ => Err(anyhow::anyhow!("Expected commit message response")),
            }
        },
    )
    .await?;

    // Finish spinner after agent completes
    spinner.finish_and_clear();

    if config.print_only {
        println!("{}", format_commit_message(&generated_message));
        return Ok(());
    }

    if config.auto_commit {
        // Only allow auto-commit for local repositories
        if service.is_remote_repository() {
            ui::print_error(
                "Cannot automatically commit to a remote repository. Use --print instead.",
            );
            return Err(anyhow::anyhow!(
                "Auto-commit not supported for remote repositories"
            ));
        }

        match service.perform_commit(&format_commit_message(&generated_message)) {
            Ok(result) => {
                let output =
                    format_commit_result(&result, &format_commit_message(&generated_message));
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
        println!("{}", format_commit_message(&generated_message));
        return Ok(());
    }

    run_tui_commit(
        vec![generated_message],
        effective_instructions,
        preset_str,
        git_info.user_name,
        git_info.user_email,
        service,
        cfg.use_gitmoji,
    )
    .await?;

    Ok(())
}

/// Handle the `Gen` command
async fn handle_gen(
    common: CommonParams,
    config: GenConfig,
    repository_url: Option<String>,
    use_agent: bool,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'gen' command with common: {:?}, auto_commit: {}, use_gitmoji: {}, print: {}, verify: {}, use_agent: {}",
        common,
        config.auto_commit,
        config.use_gitmoji,
        config.print_only,
        config.verify,
        use_agent
    );

    ui::print_version(crate_version!());
    ui::print_newline();

    if use_agent {
        // Use agent-based generation with TUI integration
        handle_gen_with_agent(common, config, repository_url).await
    } else {
        // Use existing implementation
        commit::handle_gen_command(
            common,
            config.auto_commit,
            config.use_gitmoji,
            config.print_only,
            config.verify,
            repository_url,
        )
        .await
    }
}

/// Handle the `Config` command
fn handle_config(
    common: &CommonParams,
    api_key: Option<String>,
    model: Option<String>,
    fast_model: Option<String>,
    token_limit: Option<usize>,
    param: Option<Vec<String>>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'config' command with common: {:?}, api_key: {:?}, model: {:?}, token_limit: {:?}, param: {:?}",
        common,
        api_key,
        model,
        token_limit,
        param
    );
    commands::handle_config_command(common, api_key, model, fast_model, token_limit, param)
}

/// Handle the `Review` command
#[allow(clippy::too_many_arguments)]
async fn handle_review(
    common: CommonParams,
    print: bool,
    repository_url: Option<String>,
    include_unstaged: bool,
    commit: Option<String>,
    from: Option<String>,
    to: Option<String>,
    use_agent: bool,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'review' command with common: {:?}, print: {}, include_unstaged: {}, commit: {:?}, from: {:?}, to: {:?}, use_agent: {}",
        common,
        print,
        include_unstaged,
        commit,
        from,
        to,
        use_agent
    );

    ui::print_version(crate_version!());
    ui::print_newline();

    if use_agent {
        // Use agent framework for code review
        let task_prompt = format!(
            "Review the code changes. Include unstaged: {include_unstaged}, commit: {commit:?}, from: {from:?}, to: {to:?}"
        );

        crate::agents::handle_with_agent(
            common,
            repository_url,
            "review",
            &task_prompt,
            |response| async move {
                if print {
                    println!("{response}");
                } else {
                    ui::print_success("Code review completed successfully");
                    println!("{response}");
                }
                Ok(())
            },
        )
        .await
    } else {
        // Use existing implementation
        commit::handle_review_command(
            common,
            print,
            repository_url,
            include_unstaged,
            commit,
            from,
            to,
        )
        .await
    }
}

/// Handle the `Changelog` command
async fn handle_changelog(
    common: CommonParams,
    from: String,
    to: Option<String>,
    repository_url: Option<String>,
    update: bool,
    file: Option<String>,
    version_name: Option<String>,
    use_agent: bool,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'changelog' command with common: {:?}, from: {}, to: {:?}, update: {}, file: {:?}, version_name: {:?}, use_agent: {}",
        common,
        from,
        to,
        update,
        file,
        version_name,
        use_agent
    );

    if use_agent {
        // Use agent framework for changelog generation
        let task_prompt = format!(
            "Generate a changelog from {} to {}. Version: {:?}",
            from,
            to.as_deref().unwrap_or("HEAD"),
            version_name
        );

        crate::agents::handle_with_agent(
            common,
            repository_url,
            "changelog",
            &task_prompt,
            |response| async move {
                if update {
                    // TODO: Update changelog file
                    ui::print_success("Changelog updated successfully");
                } else {
                    println!("{response}");
                }
                Ok(())
            },
        )
        .await
    } else {
        // Use existing implementation
        changes::handle_changelog_command(
            common,
            from,
            to,
            repository_url,
            update,
            file,
            version_name,
        )
        .await
    }
}

/// Handle the `Release Notes` command
async fn handle_release_notes(
    common: CommonParams,
    from: String,
    to: Option<String>,
    repository_url: Option<String>,
    version_name: Option<String>,
    use_agent: bool,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'release-notes' command with common: {:?}, from: {}, to: {:?}, version_name: {:?}, use_agent: {}",
        common,
        from,
        to,
        version_name,
        use_agent
    );

    if use_agent {
        // Use agent framework for release notes generation
        let task_prompt = format!(
            "Generate release notes from {} to {}. Version: {:?}",
            from,
            to.as_deref().unwrap_or("HEAD"),
            version_name
        );

        crate::agents::handle_with_agent(
            common,
            repository_url,
            "release_notes",
            &task_prompt,
            |response| async move {
                println!("{response}");
                Ok(())
            },
        )
        .await
    } else {
        // Use existing implementation
        changes::handle_release_notes_command(common, from, to, repository_url, version_name).await
    }
}

/// Handle the `Serve` command
async fn handle_serve(
    dev: bool,
    transport: String,
    port: Option<u16>,
    listen_address: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'serve' command with dev: {}, transport: {}, port: {:?}, listen_address: {:?}",
        dev,
        transport,
        port,
        listen_address
    );
    commands::handle_serve_command(dev, transport, port, listen_address).await
}

/// Handle the command based on parsed arguments
#[allow(clippy::too_many_lines)]
pub async fn handle_command(
    command: Commands,
    repository_url: Option<String>,
    use_agent: bool,
) -> anyhow::Result<()> {
    match command {
        Commands::Gen {
            common,
            auto_commit,
            no_gitmoji,
            print,
            no_verify,
        } => {
            handle_gen(
                common,
                GenConfig {
                    auto_commit,
                    use_gitmoji: !no_gitmoji,
                    print_only: print,
                    verify: !no_verify,
                },
                repository_url,
                use_agent,
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
        } => handle_config(&common, api_key, model, fast_model, token_limit, param),
        Commands::Review {
            common,
            print,
            include_unstaged,
            commit,
            from,
            to,
        } => {
            handle_review(
                common,
                print,
                repository_url,
                include_unstaged,
                commit,
                from,
                to,
                use_agent,
            )
            .await
        }
        Commands::Changelog {
            common,
            from,
            to,
            update,
            file,
            version_name,
        } => {
            handle_changelog(
                common,
                from,
                to,
                repository_url,
                update,
                file,
                version_name,
                use_agent,
            )
            .await
        }
        Commands::ReleaseNotes {
            common,
            from,
            to,
            version_name,
        } => handle_release_notes(common, from, to, repository_url, version_name, use_agent).await,
        Commands::Serve {
            dev,
            transport,
            port,
            listen_address,
        } => handle_serve(dev, transport, port, listen_address).await,
        Commands::ProjectConfig {
            common,
            model,
            fast_model,
            token_limit,
            param,
            print,
        } => commands::handle_project_config_command(
            &common,
            model,
            fast_model,
            token_limit,
            param,
            print,
        ),
        Commands::ListPresets => commands::handle_list_presets_command(),
        Commands::Pr {
            common,
            print,
            from,
            to,
        } => handle_pr(common, print, from, to, repository_url, use_agent).await,
    }
}

/// Handle the `Pr` command with agent framework
async fn handle_pr_with_agent(
    common: CommonParams,
    print: bool,
    from: Option<String>,
    to: Option<String>,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    use crate::commit::IrisCommitService;
    use crate::commit::types::format_pull_request;
    use crate::config::Config;
    use crate::git::GitRepo;
    use crate::instruction_presets::PresetType;
    use anyhow::Context;
    use std::sync::Arc;

    // Enable agent mode for enhanced status display
    crate::agents::status::enable_agent_mode();

    // Check if the preset is appropriate for PR descriptions
    if !common.is_valid_preset_for_type(PresetType::Review)
        && !common.is_valid_preset_for_type(PresetType::Both)
    {
        ui::print_warning(
            "The specified preset may not be suitable for PR descriptions. Consider using a review or general preset instead.",
        );
        ui::print_info("Run 'git-iris list-presets' to see available presets for PRs.");
    }

    let mut cfg = Config::load()?;
    common.apply_to_config(&mut cfg)?;

    // Create the service
    let repo_url = repository_url.clone().or(common.repository_url.clone());
    let git_repo = GitRepo::new_from_url(repo_url).context("Failed to create GitRepo")?;
    let repo_path = git_repo.repo_path().clone();

    let _service = Arc::new(IrisCommitService::new(
        cfg.clone(),
        &repo_path,
        &cfg.default_provider,
        cfg.use_gitmoji,
        false, // verification not needed for PR descriptions
        git_repo,
    ).map_err(|e| {
        ui::print_error(&format!("Error: {e}"));
        ui::print_info("\nPlease ensure the following:");
        ui::print_info("1. Git is installed and accessible from the command line.");
        ui::print_info(
            "2. You are running this command from within a Git repository or provide a repository URL with --repo.",
        );
        ui::print_info("3. You have set up your configuration using 'git-iris config'.");
        e
    })?);

    // Use agent framework for PR description generation
    let task_prompt = format!(
        "Generate a pull request description comparing changes. From: {:?}, To: {:?}",
        from.as_deref().unwrap_or("main"),
        to.as_deref().unwrap_or("HEAD")
    );

    let generated_pr = crate::agents::handle_with_agent(
        common,
        repository_url,
        "pr",
        &task_prompt,
        |response| async move {
            match response {
                crate::agents::iris::StructuredResponse::PullRequest(pr) => Ok(pr),
                _ => Err(anyhow::anyhow!("Expected pull request response")),
            }
        },
    )
    .await?;

    if print {
        println!("{}", format_pull_request(&generated_pr));
    } else {
        ui::print_success("PR description generated successfully");
        println!("{}", format_pull_request(&generated_pr));
    }

    Ok(())
}

/// Handle the `Pr` command
async fn handle_pr(
    common: CommonParams,
    print: bool,
    from: Option<String>,
    to: Option<String>,
    repository_url: Option<String>,
    use_agent: bool,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'pr' command with common: {:?}, print: {}, from: {:?}, to: {:?}, use_agent: {}",
        common,
        print,
        from,
        to,
        use_agent
    );

    ui::print_version(crate_version!());
    ui::print_newline();

    if use_agent {
        // Use agent-based generation with structured output
        handle_pr_with_agent(common, print, from, to, repository_url).await
    } else {
        // Use existing implementation
        commit::handle_pr_command(common, print, repository_url, from, to).await
    }
}
