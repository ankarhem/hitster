//! Configuration management for Hitster
//!
//! This module handles loading configuration from environment variables,
//! .env files, and configuration files using the `config` crate.

use config::{Config, File};
use serde::Deserialize;
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
        Self::initialize_config_files()?;

        let builder = Config::builder()
            .add_source(File::with_name("config.default.toml").required(true))
            .add_source(File::with_name("config.toml").required(false))
            .add_source(config::Environment::with_prefix("HITSTER").separator("_"));

        let config = builder.build()?;

        let settings: Settings = config.try_deserialize()?;

        Ok(settings)
    }

    fn initialize_config_files() -> anyhow::Result<()> {
        use std::fs;
        use std::path::Path;

        let default_path = Path::new("config.default.toml");
        if !default_path.exists() {
            info!("Default config file does not exist, creating");
            fs::write(default_path, include_str!("../config.default.toml"))?;
        }

        let custom_path = Path::new("config.toml");
        if !custom_path.exists() {
            info!("Custom config file does not exist, creating");
            fs::write(custom_path, include_str!("../config.example.toml"))?;
        }

        Ok(())
    }
}
