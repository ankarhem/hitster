pub mod config;
pub mod infrastructure;
pub mod application;
pub mod web;
pub mod pdf_generator;

pub use config::{Settings, ConfigError};
pub use infrastructure::{SpotifyService, Database};
pub use web::WebServer;
pub use web::generate_qr_data_url;
pub use web::templates::{CardsTemplate, CardTemplate};
pub use web::AppError;
pub use application::{HitsterService, Playlist, PlaylistId, Track};
pub use pdf_generator::generate_pdfs;