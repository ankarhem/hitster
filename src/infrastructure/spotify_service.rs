use anyhow::Result;
use rspotify::{
    clients::BaseClient,
    model::{PlayableItem, PlaylistId as RspotifyPlaylistId},
    ClientCredsSpotify, Credentials,
};
use crate::application::models::{Playlist, PlaylistId, SongCard};
use crate::Settings;
use futures::StreamExt;
use tracing::{debug, info, warn};

impl TryFrom<PlaylistId> for RspotifyPlaylistId<'static> {
    type Error = anyhow::Error;

    fn try_from(id: PlaylistId) -> Result<Self, Self::Error> {
        Ok(RspotifyPlaylistId::from_id(id.as_str().to_string())?)
    }
}

#[derive(Clone)]
pub struct SpotifyService {
    client: ClientCredsSpotify,
}

impl SpotifyService {
    pub async fn new(settings: &Settings) -> Result<Self> {
        debug!("Creating Spotify service instance");
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);

        let spotify = ClientCredsSpotify::new(creds);
        debug!("Requesting Spotify access token");
        spotify.request_token().await?;
        info!("Spotify authentication successful");

        Ok(Self { client: spotify })
    }

    pub async fn get_playlist(&self, playlist_id: PlaylistId) -> Result<Playlist> {
        info!("Fetching playlist: {}", playlist_id);
        let rspotify_playlist_id: RspotifyPlaylistId<'static> = playlist_id.clone().try_into()?;
        
        debug!("Fetching playlist metadata");
        let playlist = self.client.playlist(rspotify_playlist_id, None, None).await?;
        debug!("Playlist name: {}", playlist.name);
        
        let mut tracks_stream = self
            .client
            .playlist_items(playlist.id, None, None);
        
        let mut tracks = Vec::new();
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
                tracks.push(SongCard {
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
        
        if skipped_tracks > 0 {
            warn!("Skipped {} non-track items", skipped_tracks);
        }
        
        Ok(Playlist {
            id: playlist_id,
            name: playlist.name,
            tracks,
        })
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
    fn test_playlist_creation() {
        let playlist_id: PlaylistId = "test_id".parse().unwrap();
        let tracks = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://open.spotify.com/track/test".to_string(),
            },
        ];
        
        let playlist = Playlist {
            id: playlist_id,
            name: "Test Playlist".to_string(),
            tracks,
        };
        
        assert_eq!(playlist.name, "Test Playlist");
        assert_eq!(playlist.tracks.len(), 1);
        assert_eq!(playlist.tracks[0].title, "Test Song");
    }
}