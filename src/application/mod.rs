//! Application layer

pub mod interfaces;
pub mod playlist_service;

pub use playlist_service::PlaylistService;
pub use interfaces::*;