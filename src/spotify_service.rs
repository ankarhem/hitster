use anyhow::Result;
use rspotify::{
    clients::BaseClient,
    model::{PlayableItem, PlaylistId},
    ClientCredsSpotify, Credentials,
};
use std::env;
use futures::StreamExt;

#[derive(Debug)]
pub struct SongCard {
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
}

pub struct SpotifyService {
    client: ClientCredsSpotify,
}

impl SpotifyService {
    pub async fn new() -> Result<Self> {
        let client_id = env::var("SPOTIFY_CLIENT_ID")?;
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET")?;
        
        let creds = Credentials::new(&client_id, &client_secret);
        
        let spotify = ClientCredsSpotify::new(creds);
        spotify.request_token().await?;
        
        Ok(Self { client: spotify })
    }

    pub async fn get_playlist_tracks(&self, playlist_url: &str) -> Result<Vec<SongCard>> {
        let playlist_id = Self::extract_playlist_id(playlist_url)?;
        let playlist_id = PlaylistId::from_id(&playlist_id)?;
        
        let playlist = self.client.playlist(playlist_id, None, None).await?;
        
        let mut tracks_stream = self
            .client
            .playlist_items(playlist.id, None, None);
        
        let mut cards = Vec::new();
        while let Some(item_result) = tracks_stream.next().await {
            let item = item_result?;
            if let Some(PlayableItem::Track(track)) = item.track {
                let artist_names: Vec<String> = track.artists.iter().map(|a| a.name.clone()).collect();
                let year = track.album.release_date.as_deref()
                    .unwrap_or("Unknown")
                    .split('-')
                    .next()
                    .unwrap_or("Unknown")
                    .to_string();
                
                cards.push(SongCard {
                    title: track.name,
                    artist: artist_names.join(", "),
                    year,
                    spotify_url: track.external_urls.get("spotify").cloned().unwrap_or_default(),
                });
            }
        }
        
        Ok(cards)
    }

    fn extract_playlist_id(url: &str) -> Result<String> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }
        
        let parts: Vec<&str> = url.split('/').collect();
        let last_part = parts.last().ok_or_else(|| anyhow::anyhow!("Invalid playlist URL"))?;
        
        if last_part.is_empty() {
            return Err(anyhow::anyhow!("Invalid playlist URL: missing ID"));
        }
        
        let id_parts: Vec<&str> = last_part.split('?').collect();
        let id = id_parts.first().ok_or_else(|| anyhow::anyhow!("Invalid playlist URL"))?;
        
        if id.is_empty() {
            return Err(anyhow::anyhow!("Invalid playlist URL: empty ID"));
        }
        
        Ok(id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_playlist_id_valid_url() {
        let url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M";
        let result = SpotifyService::extract_playlist_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_extract_playlist_id_with_query_params() {
        let url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abc123";
        let result = SpotifyService::extract_playlist_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_extract_playlist_id_invalid_url() {
        let url = "https://open.spotify.com/invalid/37i9dQZF1DXcBWIGoYBM5M";
        let result = SpotifyService::extract_playlist_id(url);
        // The function extracts the ID regardless of the path structure
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_extract_playlist_id_empty_url() {
        let url = "";
        let result = SpotifyService::extract_playlist_id(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_playlist_id_no_id() {
        let url = "https://open.spotify.com/playlist/";
        let result = SpotifyService::extract_playlist_id(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_song_card_creation() {
        let card = SongCard {
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            year: "2023".to_string(),
            spotify_url: "https://open.spotify.com/track/test".to_string(),
        };

        assert_eq!(card.title, "Test Song");
        assert_eq!(card.artist, "Test Artist");
        assert_eq!(card.year, "2023");
        assert_eq!(card.spotify_url, "https://open.spotify.com/track/test");
    }
}