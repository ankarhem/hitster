use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub spotify: SpotifySettings,
}

#[derive(Debug, Deserialize)]
pub struct SpotifySettings {
    pub client_id: String,
    pub client_secret: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name(".env").required(false))
            .add_source(Environment::with_prefix("HITSTER").separator("_"));

        builder = builder.add_source(
            Environment::with_prefix("SPOTIFY")
                .separator("_")
                .prefix_separator("_")
        );

        let settings = builder.build()?;

        settings.try_deserialize::<Settings>()
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(Environment::with_prefix("HITSTER").separator("_"));

        builder = builder.add_source(
            Environment::with_prefix("SPOTIFY")
                .separator("_")
                .prefix_separator("_")
        );

        let settings = builder.build()?;

        settings.try_deserialize::<Settings>()
    }
}