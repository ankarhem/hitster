pub mod config;
pub mod html_generator;
pub mod qr_generator;
pub mod templates;
pub mod infrastructure;
pub mod application;
pub mod web_server;

pub use config::{Settings, ConfigError};
pub use infrastructure::SpotifyService;
pub use html_generator::HtmlGenerator;
pub use web_server::WebServer;
pub use templates::{CardsTemplate, CardTemplate};
pub use application::{HitsterService, Playlist, PlaylistId, SongCard};