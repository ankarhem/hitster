//! # Hitster - Spotify Playlist Card Generator
//! 
//! A web application that generates printable HTML cards from Spotify playlists.
//! Each card contains song information and a QR code that links to the song on Spotify.
//! 
//! ## Features
//! - Generate HTML cards from any Spotify playlist
//! - Embedded QR codes for each song
//! - Print-optimized layout (business card size)
//! - Web interface for easy playlist input
//! - No external dependencies for QR code generation
//! 
//! ## Usage
//! 
//! ### Web Server Mode
//! ```bash
//! cargo run
//! ```
//! 
//! This will start a web server on port 3000. Navigate to `http://localhost:3000`
//! to access the web interface.
//! 
//! ### Configuration
//! Set the following environment variables:
//! - `SPOTIFY_CLIENT_ID`: Your Spotify application client ID
//! - `SPOTIFY_CLIENT_SECRET`: Your Spotify application client secret
//! 
//! You can also create a `.env` file in the project root with these variables.

pub mod config;
pub mod spotify_service;
pub mod html_generator;
pub mod web_server;
pub mod qr_generator;
pub mod templates;

pub use config::{Settings, ConfigError};
pub use spotify_service::{SpotifyService, SongCard, PlaylistId};
pub use html_generator::HtmlGenerator;
pub use web_server::WebServer;
pub use templates::{CardsTemplate, CardTemplate};