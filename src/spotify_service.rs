//! Spotify service integration
//! 
//! This module handles all interactions with the Spotify API,
//! including authentication, playlist fetching, and data processing.

use anyhow::Result;
use rspotify::{
    clients::BaseClient,
    model::{PlayableItem, PlaylistId as RspotifyPlaylistId},
    ClientCredsSpotify, Credentials,
};
use crate::Settings;
use futures::StreamExt;
use std::str::FromStr;
use tracing::{debug, info, warn};

/// Represents a song card with all necessary information
/// 
/// This struct contains the song details that will be displayed
/// on each generated card, including the Spotify URL for QR code generation.
#[derive(Debug, Clone, PartialEq)]
pub struct SongCard {
    /// Song title
    pub title: String,
    /// Artist name(s) comma-separated
    pub artist: String,
    /// Release year
    pub year: String,
    /// Full Spotify URL for the song
    pub spotify_url: String,
}

/// Wrapper for Spotify playlist ID with validation
/// 
/// This type provides validation and parsing for Spotify playlist IDs,
/// supporting both raw IDs and full URLs.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistId(String);

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

/// Spotify API client for fetching playlist data
/// 
/// This service handles authentication with Spotify and provides methods
/// to fetch playlist information and convert tracks to song cards.
#[derive(Clone)]
pub struct SpotifyService {
    /// Internal Spotify client
    client: ClientCredsSpotify,
}

impl SpotifyService {
    /// Create a new Spotify service instance
    /// 
    /// This method authenticates with Spotify using client credentials
    /// and prepares the client for API calls.
    /// 
    /// # Arguments
    /// 
    /// * `settings` - Application configuration containing Spotify credentials
    /// 
    /// # Errors
    /// 
    /// Returns an error if authentication fails or credentials are invalid
    pub async fn new(settings: &Settings) -> Result<Self> {
        debug!("Creating Spotify service instance");
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);

        let spotify = ClientCredsSpotify::new(creds);
        debug!("Requesting Spotify access token");
        spotify.request_token().await?;
        info!("Spotify authentication successful");

        Ok(Self { client: spotify })
    }

    /// Fetch all tracks from a playlist and convert to song cards
    /// 
    /// This method fetches a playlist from Spotify and converts all tracks
    /// to `SongCard` instances suitable for card generation.
    /// 
    /// # Arguments
    /// 
    /// * `playlist_id` - The playlist ID to fetch
    /// 
    /// # Returns
    /// 
    /// A vector of `SongCard` instances representing each track in the playlist
    /// 
    /// # Errors
    /// 
    /// Returns an error if the playlist cannot be fetched or if API calls fail
    pub async fn get_playlist_tracks_by_id(&self, playlist_id: PlaylistId) -> Result<Vec<SongCard>> {
        info!("Fetching playlist: {}", playlist_id);
        let rspotify_playlist_id: RspotifyPlaylistId<'static> = playlist_id.try_into()?;
        
        debug!("Fetching playlist metadata");
        let playlist = self.client.playlist(rspotify_playlist_id, None, None).await?;
        debug!("Playlist name: {}", playlist.name);
        
        let mut tracks_stream = self
            .client
            .playlist_items(playlist.id, None, None);
        
        let mut cards = Vec::new();
        let mut skipped_tracks = 0;
        
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
                
                let title = track.name.clone();
                cards.push(SongCard {
                    title: title.clone(),
                    artist: artist_names.join(", "),
                    year,
                    spotify_url: track.external_urls.get("spotify").cloned().unwrap_or_default(),
                });
                
                debug!("Processed track: {} - {}", title, artist_names.join(", "));
            } else {
                skipped_tracks += 1;
                debug!("Skipping non-track item");
            }
        }
        
        info!("Processed {} tracks, skipped {} non-track items", cards.len(), skipped_tracks);
        if skipped_tracks > 0 {
            warn!("Playlist contained {} non-track items that were skipped", skipped_tracks);
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