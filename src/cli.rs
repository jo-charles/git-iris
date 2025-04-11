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
        crate::logger::set_log_file(LOG_FILE)?;
    } else {
        crate::logger::disable_logging();
    }

    if let Some(command) = cli.command {
        handle_command(command, cli.repository_url).await
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

/// Handle the `Gen` command
async fn handle_gen(
    common: CommonParams,
    config: GenConfig,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'gen' command with common: {:?}, auto_commit: {}, use_gitmoji: {}, print: {}, verify: {}",
        common,
        config.auto_commit,
        config.use_gitmoji,
        config.print_only,
        config.verify
    );

    ui::print_version(crate_version!());
    println!();

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

/// Handle the `Config` command
fn handle_config(
    common: &CommonParams,
    api_key: Option<String>,
    model: Option<String>,
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
    commands::handle_config_command(common, api_key, model, token_limit, param)
}

/// Handle the `Review` command
async fn handle_review(
    common: CommonParams,
    print: bool,
    repository_url: Option<String>,
    include_unstaged: bool,
    commit: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'review' command with common: {:?}, print: {}, include_unstaged: {}, commit: {:?}",
        common,
        print,
        include_unstaged,
        commit
    );
    ui::print_version(crate_version!());
    println!();
    commit::review::handle_review_command(common, print, repository_url, include_unstaged, commit)
        .await
}

/// Handle the `Changelog` command
async fn handle_changelog(
    common: CommonParams,
    from: String,
    to: Option<String>,
    repository_url: Option<String>,
    update: bool,
    file: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'changelog' command with common: {:?}, from: {}, to: {:?}, update: {}, file: {:?}",
        common,
        from,
        to,
        update,
        file
    );
    changes::handle_changelog_command(common, from, to, repository_url, update, file).await
}

/// Handle the `ReleaseNotes` command
async fn handle_release_notes(
    common: CommonParams,
    from: String,
    to: Option<String>,
    repository_url: Option<String>,
) -> anyhow::Result<()> {
    log_debug!(
        "Handling 'release-notes' command with common: {:?}, from: {}, to: {:?}",
        common,
        from,
        to
    );
    changes::handle_release_notes_command(common, from, to, repository_url).await
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
        } => {
            let config = GenConfig {
                auto_commit,
                use_gitmoji: !no_gitmoji,
                print_only: print,
                verify: !no_verify,
            };
            handle_gen(common, config, repository_url).await?;
        }
        Commands::Config {
            common,
            api_key,
            model,
            token_limit,
            param,
        } => {
            handle_config(&common, api_key, model, token_limit, param)?;
        }
        Commands::ListPresets => {
            log_debug!("Handling 'list_presets' command");
            commands::handle_list_presets_command()?;
        }
        Commands::Changelog {
            common,
            from,
            to,
            update,
            file,
        } => {
            handle_changelog(common, from, to, repository_url, update, file).await?;
        }
        Commands::ReleaseNotes { common, from, to } => {
            handle_release_notes(common, from, to, repository_url).await?;
        }
        Commands::Review {
            common,
            print,
            include_unstaged,
            commit,
        } => {
            handle_review(common, print, repository_url, include_unstaged, commit).await?;
        }
        Commands::Serve {
            dev,
            transport,
            port,
            listen_address,
        } => {
            handle_serve(dev, transport, port, listen_address).await?;
        }
        Commands::ProjectConfig {
            common,
            model,
            token_limit,
            param,
            print,
        } => {
            commands::handle_project_config_command(&common, model, token_limit, param, print)?;
        }
    }

    Ok(())
}
