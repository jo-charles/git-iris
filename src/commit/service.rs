use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use super::prompt::{create_system_prompt, create_user_prompt, process_commit_message};
use super::review::GeneratedReview;
use super::types::GeneratedMessage;
use crate::config::Config;
use crate::context::CommitContext;
use crate::git::{CommitResult, GitRepo};
use crate::instruction_presets::{PresetType, get_instruction_preset_library};
use crate::llm;
use crate::log_debug;
use crate::token_optimizer::TokenOptimizer;

/// Service for handling Git commit operations with AI assistance
pub struct IrisCommitService {
    config: Config,
    repo: Arc<GitRepo>,
    provider_name: String,
    use_gitmoji: bool,
    verify: bool,
    cached_context: Arc<RwLock<Option<CommitContext>>>,
}

impl IrisCommitService {
    /// Create a new `IrisCommitService` instance
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the service
    /// * `repo_path` - The path to the Git repository (unused but kept for API compatibility)
    /// * `provider_name` - The name of the LLM provider to use
    /// * `use_gitmoji` - Whether to use Gitmoji in commit messages
    /// * `verify` - Whether to verify commits
    /// * `git_repo` - An existing `GitRepo` instance
    ///
    /// # Returns
    ///
    /// A Result containing the new `IrisCommitService` instance or an error
    pub fn new(
        config: Config,
        _repo_path: &Path,
        provider_name: &str,
        use_gitmoji: bool,
        verify: bool,
        git_repo: GitRepo,
    ) -> Result<Self> {
        Ok(Self {
            config,
            repo: Arc::new(git_repo),
            provider_name: provider_name.to_string(),
            use_gitmoji,
            verify,
            cached_context: Arc::new(RwLock::new(None)),
        })
    }

    /// Check if the repository is remote
    pub fn is_remote_repository(&self) -> bool {
        self.repo.is_remote()
    }

    /// Check the environment for necessary prerequisites
    pub fn check_environment(&self) -> Result<()> {
        self.config.check_environment()
    }

    /// Get Git information for the current repository
    pub async fn get_git_info(&self) -> Result<CommitContext> {
        {
            let cached_context = self.cached_context.read().await;
            if let Some(context) = &*cached_context {
                return Ok(context.clone());
            }
        }

        let context = self.repo.get_git_info(&self.config).await?;

        {
            let mut cached_context = self.cached_context.write().await;
            *cached_context = Some(context.clone());
        }
        Ok(context)
    }

    /// Private helper method to handle common token optimization logic
    ///
    /// # Arguments
    ///
    /// * `config_clone` - Configuration with preset and instructions
    /// * `system_prompt` - The system prompt to use
    /// * `context` - The commit context
    /// * `create_user_prompt_fn` - A function that creates a user prompt from a context
    ///
    /// # Returns
    ///
    /// A tuple containing the optimized context and final user prompt
    fn optimize_prompt<F>(
        &self,
        config_clone: &Config,
        system_prompt: &str,
        mut context: CommitContext,
        create_user_prompt_fn: F,
    ) -> (CommitContext, String)
    where
        F: Fn(&CommitContext) -> String,
    {
        // Get the token limit for the provider from config or default value
        let token_limit = config_clone
            .providers
            .get(&self.provider_name)
            .and_then(|p| p.token_limit)
            .unwrap_or({
                match self.provider_name.as_str() {
                    "openai" => 16_000,          // Default for OpenAI
                    "anthropic" => 100_000,      // Anthropic Claude has large context
                    "google" | "groq" => 32_000, // Default for Google and Groq
                    _ => 8_000,                  // Conservative default for other providers
                }
            });

        // Create a token optimizer to count tokens
        let optimizer = TokenOptimizer::new(token_limit);
        let system_tokens = optimizer.count_tokens(system_prompt);

        log_debug!("Token limit: {}", token_limit);
        log_debug!("System prompt tokens: {}", system_tokens);

        // Reserve tokens for system prompt and some buffer for formatting
        // 1000 token buffer provides headroom for model responses and formatting
        let context_token_limit = token_limit.saturating_sub(system_tokens + 1000);
        log_debug!("Available tokens for context: {}", context_token_limit);

        // Count tokens before optimization
        let user_prompt_before = create_user_prompt_fn(&context);
        let total_tokens_before = system_tokens + optimizer.count_tokens(&user_prompt_before);
        log_debug!("Total tokens before optimization: {}", total_tokens_before);

        // Optimize the context with remaining token budget
        context.optimize(context_token_limit);

        let user_prompt = create_user_prompt_fn(&context);
        let user_tokens = optimizer.count_tokens(&user_prompt);
        let total_tokens = system_tokens + user_tokens;

        log_debug!("User prompt tokens after optimization: {}", user_tokens);
        log_debug!("Total tokens after optimization: {}", total_tokens);

        // If we're still over the limit, truncate the user prompt directly
        // 100 token safety buffer ensures we stay under the limit
        let final_user_prompt = if total_tokens > token_limit {
            log_debug!(
                "Total tokens {} still exceeds limit {}, truncating user prompt",
                total_tokens,
                token_limit
            );
            let max_user_tokens = token_limit.saturating_sub(system_tokens + 100);
            optimizer.truncate_string(&user_prompt, max_user_tokens)
        } else {
            user_prompt
        };

        let final_tokens = system_tokens + optimizer.count_tokens(&final_user_prompt);
        log_debug!(
            "Final total tokens after potential truncation: {}",
            final_tokens
        );

        (context, final_user_prompt)
    }

    /// Generate a commit message using AI
    ///
    /// # Arguments
    ///
    /// * `preset` - The instruction preset to use
    /// * `instructions` - Custom instructions for the AI
    ///
    /// # Returns
    ///
    /// A Result containing the generated commit message or an error
    pub async fn generate_message(
        &self,
        preset: &str,
        instructions: &str,
    ) -> anyhow::Result<GeneratedMessage> {
        let mut config_clone = self.config.clone();

        // Check if the preset exists and is valid for commits
        if preset.is_empty() {
            config_clone.instruction_preset = "default".to_string();
        } else {
            let library = get_instruction_preset_library();
            if let Some(preset_info) = library.get_preset(preset) {
                if preset_info.preset_type == PresetType::Review {
                    log_debug!(
                        "Warning: Preset '{}' is review-specific, not ideal for commits",
                        preset
                    );
                }
                config_clone.instruction_preset = preset.to_string();
            } else {
                log_debug!("Preset '{}' not found, using default", preset);
                config_clone.instruction_preset = "default".to_string();
            }
        }

        config_clone.instructions = instructions.to_string();

        let context = self.get_git_info().await?;

        // Create system prompt
        let system_prompt = create_system_prompt(&config_clone)?;

        // Use the shared optimization logic
        let (_, final_user_prompt) =
            self.optimize_prompt(&config_clone, &system_prompt, context, create_user_prompt);

        let mut generated_message = llm::get_message::<GeneratedMessage>(
            &config_clone,
            &self.provider_name,
            &system_prompt,
            &final_user_prompt,
        )
        .await?;

        // Apply gitmoji setting
        if !self.use_gitmoji {
            generated_message.emoji = None;
        }

        Ok(generated_message)
    }

    /// Generate a code review using AI
    ///
    /// # Arguments
    ///
    /// * `preset` - The instruction preset to use
    /// * `instructions` - Custom instructions for the AI
    ///
    /// # Returns
    ///
    /// A Result containing the generated code review or an error
    pub async fn generate_review(
        &self,
        preset: &str,
        instructions: &str,
    ) -> anyhow::Result<GeneratedReview> {
        let mut config_clone = self.config.clone();

        // Check if the preset exists and is valid for reviews
        if preset.is_empty() {
            config_clone.instruction_preset = "default".to_string();
        } else {
            let library = get_instruction_preset_library();
            if let Some(preset_info) = library.get_preset(preset) {
                if preset_info.preset_type == PresetType::Commit {
                    log_debug!(
                        "Warning: Preset '{}' is commit-specific, not ideal for reviews",
                        preset
                    );
                }
                config_clone.instruction_preset = preset.to_string();
            } else {
                log_debug!("Preset '{}' not found, using default", preset);
                config_clone.instruction_preset = "default".to_string();
            }
        }

        config_clone.instructions = instructions.to_string();

        let context = self.get_git_info().await?;

        // Create system prompt
        let system_prompt = super::prompt::create_review_system_prompt(&config_clone)?;

        // Use the shared optimization logic
        let (_, final_user_prompt) = self.optimize_prompt(
            &config_clone,
            &system_prompt,
            context,
            super::prompt::create_review_user_prompt,
        );

        llm::get_message::<GeneratedReview>(
            &config_clone,
            &self.provider_name,
            &system_prompt,
            &final_user_prompt,
        )
        .await
    }

    /// Performs a commit with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message.
    ///
    /// # Returns
    ///
    /// A Result containing the `CommitResult` or an error.
    pub fn perform_commit(&self, message: &str) -> Result<CommitResult> {
        // Check if this is a remote repository
        if self.is_remote_repository() {
            return Err(anyhow::anyhow!("Cannot commit to a remote repository"));
        }

        let processed_message = process_commit_message(message.to_string(), self.use_gitmoji);
        log_debug!("Performing commit with message: {}", processed_message);

        if !self.verify {
            log_debug!("Skipping pre-commit hook (verify=false)");
            return self.repo.commit(&processed_message);
        }

        // Execute pre-commit hook
        log_debug!("Executing pre-commit hook");
        if let Err(e) = self.repo.execute_hook("pre-commit") {
            log_debug!("Pre-commit hook failed: {}", e);
            return Err(e);
        }
        log_debug!("Pre-commit hook executed successfully");

        // Perform the commit
        match self.repo.commit(&processed_message) {
            Ok(result) => {
                // Execute post-commit hook
                log_debug!("Executing post-commit hook");
                if let Err(e) = self.repo.execute_hook("post-commit") {
                    log_debug!("Post-commit hook failed: {}", e);
                    // We don't fail the commit if post-commit hook fails
                }
                log_debug!("Commit performed successfully");
                Ok(result)
            }
            Err(e) => {
                log_debug!("Commit failed: {}", e);
                Err(e)
            }
        }
    }

    /// Execute the pre-commit hook if verification is enabled
    pub fn pre_commit(&self) -> Result<()> {
        // Skip pre-commit hook for remote repositories
        if self.is_remote_repository() {
            log_debug!("Skipping pre-commit hook for remote repository");
            return Ok(());
        }

        if self.verify {
            self.repo.execute_hook("pre-commit")
        } else {
            Ok(())
        }
    }

    /// Create a channel for message generation
    pub fn create_message_channel(
        &self,
    ) -> (
        mpsc::Sender<Result<GeneratedMessage>>,
        mpsc::Receiver<Result<GeneratedMessage>>,
    ) {
        mpsc::channel(1)
    }
}
