//! Template modules
//! 
//! This module contains the Rust template structs that correspond to the HTML templates.
pub mod playlist;
pub use playlist::PlaylistTemplate;
pub mod error;
pub use error::ErrorTemplate;
pub mod index;
pub use index::IndexTemplate;
