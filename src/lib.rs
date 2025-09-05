pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod web;
pub mod pdf_generator;

pub use config::{Settings, ConfigError};
pub use infrastructure::{SpotifyClient};
pub use web::generate_qr_data_url;
pub use web::templates::{PlaylistTemplate, CardTemplate};
pub use web::AppError;
pub use pdf_generator::generate_pdfs;