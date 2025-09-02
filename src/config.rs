//! Configuration management for Hitster
//! 
//! This module handles loading configuration from environment variables
//! and .env files for Spotify API credentials.

use serde::Deserialize;
use thiserror::Error;
use dotenv::dotenv;

/// Configuration errors that can occur during loading
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Application configuration
/// 
/// Contains Spotify API credentials required for authentication
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Spotify application client ID
    pub client_id: String,
    /// Spotify application client secret
    pub client_secret: String,
}

impl Settings {
    /// Load configuration from environment variables
    /// 
    /// This method will:
    /// 1. Load .env file if present
    /// 2. Read SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET from environment
    /// 3. Return error if any required variable is missing
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hitster::Settings;
    /// 
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = Settings::new()?;
    /// println!("Client ID: {}", settings.client_id);
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns `ConfigError::EnvVarNotFound` if required environment variables are not set
    pub fn new() -> Result<Self, ConfigError> {
        Self::from_env()
    }

    /// Load configuration from environment variables with custom .env file path
    /// 
    /// # Arguments
    /// 
    /// * `env_path` - Optional path to .env file (if None, uses default .env)
    /// 
    /// # Errors
    /// 
    /// Returns `ConfigError::EnvVarNotFound` if required environment variables are not set
    pub fn from_env_path(env_path: Option<&str>) -> Result<Self, ConfigError> {
        if let Some(path) = env_path {
            dotenv::from_filename(path).ok();
        } else {
            dotenv().ok();
        }
        
        Self::load_from_env()
    }

    /// Load configuration from environment variables (internal)
    fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();
        Self::load_from_env()
    }

    /// Load configuration from already loaded environment variables
    fn load_from_env() -> Result<Self, ConfigError> {
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ConfigError::EnvVarNotFound("SPOTIFY_CLIENT_ID".to_string()))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ConfigError::EnvVarNotFound("SPOTIFY_CLIENT_SECRET".to_string()))?;
        
        Ok(Settings {
            client_id,
            client_secret,
        })
    }

    /// Validate configuration by checking if credentials are non-empty
    /// 
    /// # Returns
    /// 
    /// `true` if configuration is valid, `false` otherwise
    pub fn is_valid(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}