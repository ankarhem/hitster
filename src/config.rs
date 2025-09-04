//! Configuration management for Hitster
//! 
//! This module handles loading configuration from environment variables
//! and .env files for Spotify API credentials.

use serde::Deserialize;
use thiserror::Error;
use dotenv::dotenv;

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
    pub database_url: String,
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
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                let current_dir = std::env::current_dir().unwrap();
                current_dir.join("db/hitster.db").to_string_lossy().to_string()
            });
        
        Ok(Settings {
            client_id,
            client_secret,
            database_url,
        })
    }
}