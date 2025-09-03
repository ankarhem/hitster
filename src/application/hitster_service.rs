use crate::infrastructure::spotify_service::{SpotifyService, PlaylistId};
use crate::HtmlGenerator;
use anyhow::Result;
use tracing::{debug, info};

/// Main application service for Hitster
/// 
/// Orchestrates the flow from playlist ID to generated HTML cards
#[derive(Clone)]
pub struct HitsterService {
    spotify_service: SpotifyService,
    html_generator: HtmlGenerator,
}

impl HitsterService {
    /// Create a new Hitster service instance
    pub fn new(spotify_service: SpotifyService) -> Result<Self> {
        Ok(Self {
            spotify_service,
            html_generator: HtmlGenerator::new()?,
        })
    }

    /// Generate playlist cards as HTML
    pub async fn generate_playlist_cards(&self, playlist_id: &str, title: Option<String>) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[tokio::test]
    async fn test_hitster_service_creation() {
        let settings = Settings::new().unwrap();
        let spotify_service = SpotifyService::new(&settings).await.unwrap();
        let service = HitsterService::new(spotify_service);
        assert!(service.is_ok());
    }
}