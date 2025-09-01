use serde::Deserialize;
use config::ConfigError;
use dotenv::dotenv;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub client_id: String,
    pub client_secret: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();
        
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ConfigError::NotFound("SPOTIFY_CLIENT_ID not found".into()))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ConfigError::NotFound("SPOTIFY_CLIENT_SECRET not found".into()))?;
        
        Ok(Settings {
            client_id,
            client_secret,
        })
    }
}