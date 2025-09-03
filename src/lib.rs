pub mod config;
pub mod infrastructure;
pub mod application;
pub mod web;

pub use config::{Settings, ConfigError};
pub use infrastructure::SpotifyService;
pub use web::WebServer;
pub use web::generate_qr_data_url;
pub use web::templates::{CardsTemplate, CardTemplate};
pub use application::{HitsterService, Playlist, PlaylistId, Track};