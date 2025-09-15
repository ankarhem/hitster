//! Configuration management for Hitster
//!
//! This module handles loading configuration from environment variables,
//! .env files, and configuration files using the `config` crate.

use config::{Config, File};
use serde::Deserialize;

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
        let builder = Config::builder()
            .add_source(File::with_name("config.default").required(true))
            .add_source(File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("HITSTER").separator("_"));

        let config = builder.build()?;

        let settings: Settings = config.try_deserialize()?;

        Ok(settings)
    }
}
