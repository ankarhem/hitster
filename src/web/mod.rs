pub mod qr_code;
pub mod server;
pub mod templates;
pub mod error;
pub mod controllers;

pub use qr_code::generate_qr_data_url;
pub use server::WebServer;
pub use templates::{CardsTemplate, CardTemplate};
pub use error::AppError;
pub use controllers::*;