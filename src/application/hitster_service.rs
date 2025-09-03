use crate::infrastructure::spotify_service::SpotifyService;
use crate::application::models::PlaylistId;
use crate::HtmlGenerator;
use anyhow::Result;
use tracing::info;

#[derive(Clone)]
pub struct HitsterService {
    spotify_service: SpotifyService,
    html_generator: HtmlGenerator,
}

impl HitsterService {
    pub fn new(spotify_service: SpotifyService) -> Result<Self> {
        Ok(Self {
            spotify_service,
            html_generator: HtmlGenerator::new()?,
        })
    }

    pub async fn generate_playlist_cards(&self, playlist_id: &str, title: Option<String>) -> Result<String> {
        let playlist_id: PlaylistId = playlist_id.parse()?;
        let playlist = self.spotify_service.get_playlist(playlist_id.clone()).await?;
        let title = title.unwrap_or_else(|| playlist.name);
        let html = self.html_generator.build_html_content(playlist.tracks, &title)?;
        
        info!("Generated HTML for playlist: {}", playlist_id);
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