//! Configuration management for Git-Iris.
//!
//! Handles personal config (~/.config/git-iris/config.toml) and
//! per-project config (.irisconfig) with proper layering.

use crate::git::GitRepo;
use crate::instruction_presets::get_instruction_preset_library;
use crate::log_debug;
use crate::providers::{Provider, ProviderConfig};

use anyhow::{Context, Result, anyhow};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Project configuration filename
pub const PROJECT_CONFIG_FILENAME: &str = ".irisconfig";

/// Main configuration structure
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    /// Default LLM provider
    #[serde(default)]
    pub default_provider: String,
    /// Provider-specific configurations (keyed by provider name)
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    /// Use gitmoji in commit messages
    #[serde(default = "default_true")]
    pub use_gitmoji: bool,
    /// Custom instructions for all operations
    #[serde(default)]
    pub instructions: String,
    /// Instruction preset name
    #[serde(default = "default_preset")]
    pub instruction_preset: String,
    /// Runtime-only: temporary instructions override
    #[serde(skip)]
    pub temp_instructions: Option<String>,
    /// Runtime-only: temporary preset override
    #[serde(skip)]
    pub temp_preset: Option<String>,
    /// Runtime-only: flag if loaded from project config
    #[serde(skip)]
    pub is_project_config: bool,
}

fn default_true() -> bool {
    true
}

fn default_preset() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();
        for provider in Provider::ALL {
            providers.insert(
                provider.name().to_string(),
                ProviderConfig::with_defaults(*provider),
            );
        }

        Self {
            default_provider: Provider::default().name().to_string(),
            providers,
            use_gitmoji: true,
            instructions: String::new(),
            instruction_preset: default_preset(),
            temp_instructions: None,
            temp_preset: None,
            is_project_config: false,
        }
    }
}

impl Config {
    /// Load configuration (personal + project overlay)
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            Self::migrate_if_needed(config)
        } else {
            Self::default()
        };

        // Overlay project config if available
        if let Ok(project_config) = Self::load_project_config() {
            config.merge_with_project_config(project_config);
        }

        log_debug!("Configuration loaded: {:?}", config);
        Ok(config)
    }

    /// Load project-specific configuration
    pub fn load_project_config() -> Result<Self> {
        let config_path = Self::get_project_config_path()?;
        if !config_path.exists() {
            return Err(anyhow!("Project configuration file not found"));
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        let mut config: Self = toml::from_str(&content).with_context(|| {
            format!(
                "Invalid {} format. Check for syntax errors.",
                PROJECT_CONFIG_FILENAME
            )
        })?;

        config.is_project_config = true;
        Ok(config)
    }

    /// Get path to project config file
    pub fn get_project_config_path() -> Result<PathBuf> {
        let repo_root = GitRepo::get_repo_root()?;
        Ok(repo_root.join(PROJECT_CONFIG_FILENAME))
    }

    /// Merge project config into this config (project takes precedence, but never API keys)
    pub fn merge_with_project_config(&mut self, project_config: Self) {
        log_debug!("Merging with project configuration");

        // Override default provider if set
        if !project_config.default_provider.is_empty()
            && project_config.default_provider != Provider::default().name()
        {
            self.default_provider = project_config.default_provider;
        }

        // Merge provider configs (never override API keys from project config)
        for (provider_name, proj_config) in project_config.providers {
            let entry = self.providers.entry(provider_name).or_default();

            if !proj_config.model.is_empty() {
                entry.model = proj_config.model;
            }
            if proj_config.fast_model.is_some() {
                entry.fast_model = proj_config.fast_model;
            }
            if proj_config.token_limit.is_some() {
                entry.token_limit = proj_config.token_limit;
            }
            entry
                .additional_params
                .extend(proj_config.additional_params);
        }

        // Override other settings
        self.use_gitmoji = project_config.use_gitmoji;
        self.instructions = project_config.instructions;

        if project_config.instruction_preset != default_preset() {
            self.instruction_preset = project_config.instruction_preset;
        }
    }

    /// Migrate older config formats
    fn migrate_if_needed(mut config: Self) -> Self {
        let mut migrated = false;

        // Migrate "claude" provider to "anthropic"
        if config.providers.contains_key("claude") {
            log_debug!("Migrating 'claude' provider to 'anthropic'");
            if let Some(claude_config) = config.providers.remove("claude") {
                config
                    .providers
                    .insert("anthropic".to_string(), claude_config);
            }
            if config.default_provider == "claude" {
                config.default_provider = "anthropic".to_string();
            }
            migrated = true;
        }

        if migrated && let Err(e) = config.save() {
            log_debug!("Failed to save migrated config: {}", e);
        }

        config
    }

    /// Save configuration to personal config file
    pub fn save(&self) -> Result<()> {
        if self.is_project_config {
            return Ok(());
        }

        let config_path = Self::get_config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        log_debug!("Configuration saved");
        Ok(())
    }

    /// Save as project-specific configuration (strips API keys)
    pub fn save_as_project_config(&self) -> Result<()> {
        let config_path = Self::get_project_config_path()?;

        let mut project_config = self.clone();
        project_config.is_project_config = true;

        // Strip API keys for security
        for provider_config in project_config.providers.values_mut() {
            provider_config.api_key.clear();
        }

        let content = toml::to_string_pretty(&project_config)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Get path to personal config file
    fn get_config_path() -> Result<PathBuf> {
        let mut path =
            config_dir().ok_or_else(|| anyhow!("Unable to determine config directory"))?;
        path.push("git-iris");
        fs::create_dir_all(&path)?;
        path.push("config.toml");
        Ok(path)
    }

    /// Check environment prerequisites
    pub fn check_environment(&self) -> Result<()> {
        if !GitRepo::is_inside_work_tree()? {
            return Err(anyhow!(
                "Not in a Git repository. Please run this command from within a Git repository."
            ));
        }
        Ok(())
    }

    /// Set temporary instructions for this session
    pub fn set_temp_instructions(&mut self, instructions: Option<String>) {
        self.temp_instructions = instructions;
    }

    /// Set temporary preset for this session
    pub fn set_temp_preset(&mut self, preset: Option<String>) {
        self.temp_preset = preset;
    }

    /// Get effective preset name (temp overrides saved)
    pub fn get_effective_preset_name(&self) -> &str {
        self.temp_preset
            .as_deref()
            .unwrap_or(&self.instruction_preset)
    }

    /// Get effective instructions (combines preset + custom)
    pub fn get_effective_instructions(&self) -> String {
        let preset_library = get_instruction_preset_library();
        let preset_instructions = self
            .temp_preset
            .as_ref()
            .or(Some(&self.instruction_preset))
            .and_then(|p| preset_library.get_preset(p))
            .map(|p| p.instructions.clone())
            .unwrap_or_default();

        let custom = self
            .temp_instructions
            .as_ref()
            .unwrap_or(&self.instructions);

        format!("{preset_instructions}\n\n{custom}")
            .trim()
            .to_string()
    }

    /// Update configuration with new values
    #[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
    pub fn update(
        &mut self,
        provider: Option<String>,
        api_key: Option<String>,
        model: Option<String>,
        fast_model: Option<String>,
        additional_params: Option<HashMap<String, String>>,
        use_gitmoji: Option<bool>,
        instructions: Option<String>,
        token_limit: Option<usize>,
    ) -> Result<()> {
        if let Some(ref provider_name) = provider {
            // Validate provider
            let parsed: Provider = provider_name.parse().with_context(|| {
                format!(
                    "Unknown provider '{}'. Supported: {}",
                    provider_name,
                    Provider::all_names().join(", ")
                )
            })?;

            self.default_provider = parsed.name().to_string();

            // Ensure provider config exists
            if !self.providers.contains_key(parsed.name()) {
                self.providers.insert(
                    parsed.name().to_string(),
                    ProviderConfig::with_defaults(parsed),
                );
            }
        }

        let provider_config = self
            .providers
            .get_mut(&self.default_provider)
            .context("Could not get default provider config")?;

        if let Some(key) = api_key {
            provider_config.api_key = key;
        }
        if let Some(m) = model {
            provider_config.model = m;
        }
        if let Some(fm) = fast_model {
            provider_config.fast_model = Some(fm);
        }
        if let Some(params) = additional_params {
            provider_config.additional_params.extend(params);
        }
        if let Some(gitmoji) = use_gitmoji {
            self.use_gitmoji = gitmoji;
        }
        if let Some(instr) = instructions {
            self.instructions = instr;
        }
        if let Some(limit) = token_limit {
            provider_config.token_limit = Some(limit);
        }

        log_debug!("Configuration updated");
        Ok(())
    }

    /// Get the provider configuration for a specific provider
    pub fn get_provider_config(&self, provider: &str) -> Option<&ProviderConfig> {
        // Handle legacy "claude" alias
        let name = if provider.eq_ignore_ascii_case("claude") {
            "anthropic"
        } else {
            provider
        };

        self.providers
            .get(name)
            .or_else(|| self.providers.get(&name.to_lowercase()))
    }

    /// Get the current provider as `Provider` enum
    pub fn provider(&self) -> Option<Provider> {
        self.default_provider.parse().ok()
    }

    /// Validate that the current provider is properly configured
    pub fn validate(&self) -> Result<()> {
        let provider: Provider = self
            .default_provider
            .parse()
            .with_context(|| format!("Invalid provider: {}", self.default_provider))?;

        let config = self
            .get_provider_config(provider.name())
            .ok_or_else(|| anyhow!("No configuration found for provider: {}", provider.name()))?;

        if !config.has_api_key() {
            // Check environment variable as fallback
            if std::env::var(provider.api_key_env()).is_err() {
                return Err(anyhow!(
                    "API key required for {}. Set {} or configure in ~/.config/git-iris/config.toml",
                    provider.name(),
                    provider.api_key_env()
                ));
            }
        }

        Ok(())
    }
}
