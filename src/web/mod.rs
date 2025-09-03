pub mod qr_code;
pub mod server;
pub mod templates;

pub use qr_code::generate_qr_data_url;
pub use server::WebServer;
pub use templates::{CardsTemplate, CardTemplate};