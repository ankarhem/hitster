use crate::infrastructure::spotify_service::SpotifyService;
use crate::application::models::{Playlist, PlaylistId};
use anyhow::Result;
use tracing::{info, instrument};

#[derive(Clone)]
pub struct HitsterService {
    spotify_service: SpotifyService,
}

impl HitsterService {
    #[instrument(skip(spotify_service))]
    pub fn new(spotify_service: SpotifyService) -> Result<Self> {
        Ok(Self {
            spotify_service,
        })
    }

    #[instrument(skip(self), fields(playlist_id))]
    pub async fn get_playlist_by_id(&self, playlist_id: &str) -> Result<Playlist> {
        let playlist_id: PlaylistId = playlist_id.parse()?;
        let playlist = self.spotify_service.get_playlist(playlist_id.clone()).await?;
        
        info!("Retrieved playlist: {}", playlist_id);
        Ok(playlist)
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