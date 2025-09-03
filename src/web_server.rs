//! Web server for Hitster
//! 
//! This module provides a simplified web interface for generating Spotify playlist cards.
//! It serves HTML cards directly without any welcome page or interactive elements.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use crate::{SpotifyService, HtmlGenerator, PlaylistId};
use tracing::{debug, info, error};

/// Web server for Hitster application
/// 
/// Provides a single HTTP endpoint for generating playlist cards.
#[derive(Clone)]
pub struct WebServer {
    /// Spotify service for API interactions
    spotify_service: SpotifyService,
    /// HTML generator for card creation
    html_generator: HtmlGenerator,
}

impl WebServer {
    /// Create a new web server instance
    /// 
    /// # Arguments
    /// 
    /// * `spotify_service` - Configured Spotify service instance
    /// 
    /// # Errors
    /// 
    /// Returns an error if HTML generator creation fails
    pub fn new(spotify_service: SpotifyService) -> Result<Self> {
        Ok(Self {
            spotify_service,
            html_generator: HtmlGenerator::new()?,
        })
    }

    /// Run the web server
    /// 
    /// Starts the HTTP server and listens for incoming connections.
    /// 
    /// # Arguments
    /// 
    /// * `port` - Port number to listen on
    /// 
    /// # Errors
    /// 
    /// Returns an error if the server fails to start or run
    pub async fn run(&self, port: u16) -> Result<()> {
        info!("Starting web server on port {}", port);
        
        let app = Router::new()
            .route("/playlist/:playlist_id", get(playlist_cards))
            .with_state(self.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        info!("🚀 Web server running at http://localhost:{}", port);
        info!("📋 Endpoints:");
        info!("   GET /playlist/<id>             - HTML cards for playlist");
        info!("   Example: http://localhost:{}/playlist/3vnwX8FuGWpGgQX4hBa8sE", port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Get playlist cards as HTML
    /// 
    /// Fetches playlist data and generates HTML cards.
    /// 
    /// # Arguments
    /// 
    /// * `playlist_id` - The playlist ID to process
    /// * `title` - Optional custom title for the cards
    /// 
    /// # Returns
    /// 
    /// HTML string containing the generated cards
    /// 
    /// # Errors
    /// 
    /// Returns an error if playlist fetching or HTML generation fails
    pub async fn get_playlist_cards(&self, playlist_id: &str, title: Option<String>) -> Result<String> {
        debug!("Processing playlist request for ID: {}", playlist_id);
        let playlist_id: PlaylistId = playlist_id.parse()?;
        let title = title.unwrap_or_else(|| format!("Playlist: {}", playlist_id));
        
        debug!("Fetching playlist data");
        let cards = self.spotify_service.get_playlist_tracks_by_id(playlist_id.clone()).await?;
        
        debug!("Generating HTML content for {} cards", cards.len());
        let html = self.html_generator.build_html_content(cards, &title)?;
        
        info!("Successfully generated HTML for playlist: {}", playlist_id);
        Ok(html)
    }
}

/// Playlist cards handler - serves HTML cards for a playlist
async fn playlist_cards(
    Path(playlist_id): Path<String>,
    State(server): State<WebServer>,
) -> impl IntoResponse {
    debug!("Received playlist request for ID: {}", playlist_id);
    
    match server.get_playlist_cards(&playlist_id, None).await {
        Ok(html) => {
            info!("Successfully served playlist: {}", playlist_id);
            Html(html).into_response()
        },
        Err(e) => {
            error!("Failed to serve playlist {}: {}", playlist_id, e);
            let error_html = create_error_page(&e.to_string());
            Html(error_html).into_response()
        }
    }
}

/// Create a simple error page
fn create_error_page(error_message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error - Hitster Cards</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 40px;
            background: #f8f9fa;
            color: #333;
        }}
        .error-container {{
            max-width: 600px;
            margin: 0 auto;
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-align: center;
        }}
        h1 {{
            color: #dc3545;
            margin-bottom: 20px;
        }}
        .error-message {{
            background: #f8d7da;
            color: #721c24;
            padding: 15px;
            border-radius: 4px;
            margin: 20px 0;
            font-family: monospace;
            font-size: 14px;
        }}
        .back-link {{
            display: inline-block;
            margin-top: 20px;
            color: #007bff;
            text-decoration: none;
        }}
        .back-link:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <h1>❌ Error</h1>
        <p>Sorry, we couldn't generate the playlist cards.</p>
        <div class="error-message">{}</div>
        <p>Please check the playlist ID and try again.</p>
        <a href="/" class="back-link">← Go back</a>
    </div>
</body>
</html>"#,
        error_message
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_error_page() {
        let error_html = create_error_page("Test error message");
        assert!(error_html.contains("Test error message"));
        assert!(error_html.contains("❌ Error"));
        assert!(error_html.contains("Sorry, we couldn't generate"));
    }
}