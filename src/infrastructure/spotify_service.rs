use anyhow::Result;
use rspotify::{
    clients::BaseClient,
    model::{PlayableItem, PlaylistId as RspotifyPlaylistId},
    ClientCredsSpotify, Credentials,
};
use crate::application::models::{Playlist, PlaylistId, Track};
use crate::Settings;
use futures::StreamExt;
use tracing::{info, warn, instrument};

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
    #[instrument(skip(settings))]
    pub async fn new(settings: &Settings) -> Result<Self> {
        let creds = Credentials::new(&settings.client_id, &settings.client_secret);
        let spotify = ClientCredsSpotify::new(creds);
        spotify.request_token().await?;
        info!("Spotify authentication successful");

        Ok(Self { client: spotify })
    }

    #[instrument(skip(self), fields(playlist_id = %playlist_id))]
    pub async fn get_playlist(&self, playlist_id: PlaylistId) -> Result<Playlist> {
        let rspotify_playlist_id: RspotifyPlaylistId<'static> = playlist_id.clone().try_into()?;
        
        let playlist = self.client.playlist(rspotify_playlist_id, None, None).await?;

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
                
                tracks.push(Track {
                    title: track.name.clone(),
                    artist: artist_names.join(", "),
                    year,
                    spotify_url: track.external_urls.get("spotify").cloned().unwrap_or_default(),
                });
            } else {
                skipped_tracks += 1;
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
            Track {
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