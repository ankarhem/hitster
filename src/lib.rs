pub mod config;
pub mod qr_generator;
pub mod templates;
pub mod infrastructure;
pub mod application;
pub mod web_server;

pub use config::{Settings, ConfigError};
pub use infrastructure::SpotifyService;
pub use web_server::WebServer;
pub use templates::{CardsTemplate, CardTemplate};
pub use application::{HitsterService, Playlist, PlaylistId, Track};