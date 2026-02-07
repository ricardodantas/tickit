//! Configuration module

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::theme::Theme;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Selected theme
    #[serde(default)]
    pub theme: Theme,

    /// Show completed tasks
    #[serde(default = "default_show_completed")]
    pub show_completed: bool,

    /// Default list ID for new tasks (None = inbox)
    pub default_list_id: Option<String>,

    /// Date format string
    #[serde(default = "default_date_format")]
    pub date_format: String,

    /// Enable vim-like keybindings
    #[serde(default = "default_vim_mode")]
    pub vim_mode: bool,

    /// Enable desktop notifications for due tasks
    #[serde(default = "default_notifications")]
    pub notifications: bool,

    /// Sync configuration (optional)
    #[serde(default)]
    pub sync: SyncConfig,
}

/// Sync configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Enable sync feature
    #[serde(default)]
    pub enabled: bool,

    /// Sync server URL (e.g., "https://sync.example.com")
    pub server: Option<String>,

    /// API token for authentication
    pub token: Option<String>,

    /// Auto-sync interval in seconds (0 = manual only)
    #[serde(default = "default_sync_interval")]
    pub interval_secs: u64,
}

fn default_show_completed() -> bool {
    true
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

fn default_vim_mode() -> bool {
    true
}

fn default_notifications() -> bool {
    true
}

fn default_sync_interval() -> u64 {
    300 // 5 minutes
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            show_completed: default_show_completed(),
            default_list_id: None,
            date_format: default_date_format(),
            vim_mode: default_vim_mode(),
            notifications: default_notifications(),
            sync: SyncConfig::default(),
        }
    }
}

impl Config {
    /// Get the default config file path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("tickit");
        Ok(config_dir.join("config.toml"))
    }

    /// Load config from the default path or create default
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load_from(&path)
    }

    /// Load config from a specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path).context("Failed to read config file")?;
            toml::from_str(&content).context("Failed to parse config file")
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to the default path
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;
        self.save_to(&path)
    }

    /// Save config to a specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(path, content).context("Failed to write config file")?;

        Ok(())
    }
}
