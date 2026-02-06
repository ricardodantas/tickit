//! Configuration for tickit-sync server

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind address
    #[serde(default = "default_bind")]
    pub bind: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Human-readable name for the token
    pub name: String,
    /// The API token (generated with `tickit-sync token`)
    pub token: String,
}

fn default_bind() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3030
}

fn default_db_path() -> PathBuf {
    PathBuf::from("tickit-sync.sqlite")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind: default_bind(),
                port: default_port(),
            },
            database: DatabaseConfig {
                path: default_db_path(),
            },
            tokens: Vec::new(),
        }
    }
}

impl Config {
    /// Default config path
    pub fn default_path() -> Result<PathBuf> {
        // Check for config in current directory first
        let local = PathBuf::from("config.toml");
        if local.exists() {
            return Ok(local);
        }

        // Then check XDG config
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("tickit-sync");

        Ok(config_dir.join("config.toml"))
    }

    /// Load config from default path
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        if path.exists() {
            Self::load_from(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load config from specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path).context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config file")
    }

    /// Save config to specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        // Add helpful comments
        let with_comments = format!(
            "# tickit-sync configuration\n\
             # See: https://github.com/ricardodantas/tickit\n\n\
             {}\n\n\
             # Add tokens with: tickit-sync token --name <device-name>\n\
             # Example:\n\
             # [[tokens]]\n\
             # name = \"macbook\"\n\
             # token = \"your-generated-token-here\"\n",
            content
        );

        std::fs::write(path, with_comments).context("Failed to write config file")?;

        Ok(())
    }

    /// Check if a token is valid
    pub fn validate_token(&self, token: &str) -> bool {
        self.tokens.iter().any(|t| t.token == token)
    }
}
