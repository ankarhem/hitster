//! Configuration management for Hitster
//!
//! This module handles loading configuration from environment variables
//! and .env files for Spotify API credentials.

use dotenv::dotenv;
use serde::Deserialize;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Spotify application client ID
    pub client_id: String,
    /// Spotify application client secret
    pub client_secret: String,
    /// Database URL
    pub database_path: String,
    /// Server binding host
    pub host: String,
    /// Server binding port
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Self::from_env()
    }

    fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();
        Self::load_from_env()
    }

    fn load_from_env() -> Result<Self, ConfigError> {
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ConfigError::EnvVarNotFound("SPOTIFY_CLIENT_ID".to_string()))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ConfigError::EnvVarNotFound("SPOTIFY_CLIENT_SECRET".to_string()))?;
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                let current_dir = std::env::current_dir()
                    .map_err(|e| ConfigError::Io(e))?;
                current_dir
                    .join("sqlite://./db/hitster.db")
                    .to_string_lossy()
                    .to_string()
            }
        };

        let database_path = database_url
            .split("sqlite://")
            .nth(1)
            .ok_or_else(|| ConfigError::EnvVarNotFound("DATABASE_URL".to_string()))?
            .to_string();

        let host = std::env::var("HITSTER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("HITSTER_PORT")
            .map(|s| s.parse().unwrap_or(3000))
            .unwrap_or(3000);

        info!("Database path: {}", database_path);
        info!("Server will bind to {}:{}", host, port);

        Ok(Settings {
            client_id,
            client_secret,
            database_path,
            host,
            port,
        })
    }
}
