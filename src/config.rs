//! Configuration management for Hitster
//!
//! This module handles loading configuration from environment variables,
//! .env files, and configuration files using the `config` crate.

use config::{Config, File};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Spotify application client ID
    pub spotify: SpotifyConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Server configuration
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = std::env::var("HITSTER_CONFIG_DIR")
            .unwrap_or_else(|_| ".".to_string())
            .parse::<PathBuf>()?;

        Self::initialize_config_files(&config_dir)?;

        let builder = Config::builder()
            .add_source(File::from(config_dir.join("config.default.toml")))
            .add_source(File::from(config_dir.join("config.toml")))
            .add_source(
                config::Environment::with_prefix("HITSTER")
                    .prefix_separator("_")
                    .separator("__"),
            );

        let config = builder.build()?;

        let settings: Settings = config.try_deserialize()?;

        Ok(settings)
    }

    fn initialize_config_files(config_dir: &Path) -> anyhow::Result<()> {
        use std::fs;

        let default_path = config_dir.join("config.default.toml");
        if !default_path.exists() {
            info!("Default config file does not exist, creating");
            fs::write(default_path, include_str!("../config.default.toml"))?;
        }

        let custom_path = config_dir.join("config.toml");
        if !custom_path.exists() {
            info!("Custom config file does not exist, creating");
            fs::write(custom_path, include_str!("../config.example.toml"))?;
        }

        Ok(())
    }
}
