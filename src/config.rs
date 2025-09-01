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
    pub client_id: String,
    pub client_secret: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();
        
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ConfigError::EnvVarNotFound("SPOTIFY_CLIENT_ID".to_string()))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ConfigError::EnvVarNotFound("SPOTIFY_CLIENT_SECRET".to_string()))?;
        
        Ok(Settings {
            client_id,
            client_secret,
        })
    }
}