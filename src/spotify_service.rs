use anyhow::Result;
use rspotify::{
    clients::BaseClient,
    model::{PlayableItem, PlaylistId as RspotifyPlaylistId},
    ClientCredsSpotify, Credentials,
};
use crate::Settings;
use futures::StreamExt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct SongCard {
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistId(String);

impl PlaylistId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for PlaylistId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(anyhow::anyhow!("Playlist ID cannot be empty"));
        }
        
        // Extract ID from URL if needed
        let id = if s.contains("spotify.com/playlist/") {
            Self::extract_playlist_id_from_url(s)?
        } else {
            s.to_string()
        };
        
        if id.is_empty() {
            return Err(anyhow::anyhow!("Invalid playlist ID"));
        }
        
        Ok(PlaylistId(id))
    }
}

impl PlaylistId {
    pub fn extract_playlist_id_from_url(url: &str) -> Result<String> {
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

impl TryFrom<PlaylistId> for RspotifyPlaylistId<'static> {
    type Error = anyhow::Error;

    fn try_from(id: PlaylistId) -> Result<Self, Self::Error> {
        Ok(RspotifyPlaylistId::from_id(id.0)?)
    }
}

pub struct SpotifyService {
    client: ClientCredsSpotify,
}

impl SpotifyService {
    pub async fn new(settings: &Settings) -> Result<Self> {
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);

        let spotify = ClientCredsSpotify::new(creds);
        spotify.request_token().await?;

        Ok(Self { client: spotify })
    }

    pub async fn get_playlist_tracks_by_id(&self, playlist_id: PlaylistId) -> Result<Vec<SongCard>> {
        let rspotify_playlist_id: RspotifyPlaylistId<'static> = playlist_id.try_into()?;
        
        let playlist = self.client.playlist(rspotify_playlist_id, None, None).await?;
        
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_id_from_valid_url() {
        let url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M";
        let result: Result<PlaylistId, _> = url.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_playlist_id_from_id_string() {
        let id = "37i9dQZF1DXcBWIGoYBM5M";
        let result: Result<PlaylistId, _> = id.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_playlist_id_from_url_with_query_params() {
        let url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abc123";
        let result: Result<PlaylistId, _> = url.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_playlist_id_from_invalid_url() {
        let url = "https://open.spotify.com/invalid/37i9dQZF1DXcBWIGoYBM5M";
        let result: Result<PlaylistId, _> = url.parse();
        // Since the URL doesn't contain "spotify.com/playlist/", it's treated as a raw ID
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "https://open.spotify.com/invalid/37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_playlist_id_from_empty_string() {
        let url = "";
        let result: Result<PlaylistId, _> = url.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_id_from_no_id() {
        let url = "https://open.spotify.com/playlist/";
        let result: Result<PlaylistId, _> = url.parse();
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