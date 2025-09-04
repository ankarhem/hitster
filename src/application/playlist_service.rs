use crate::infrastructure::Database;
use crate::application::{models::{Playlist, PlaylistId, Track}, HitsterService};
use crate::infrastructure::NewTrack;
use anyhow::Result;
use tracing::{info, instrument};
use std::sync::Arc;

/// Application service for playlist management business logic
#[derive(Clone)]
pub struct PlaylistService {
    database: Arc<Database>,
    hitster_service: HitsterService,
}

impl PlaylistService {
    pub fn new(database: Arc<Database>, hitster_service: HitsterService) -> Self {
        Self { database, hitster_service }
    }

    /// Process a playlist submission from a URL
    /// 
    /// This method handles the complete business logic:
    /// - Parse and validate the playlist URL
    /// - Check if playlist already exists in database
    /// - If not exists, fetch from Spotify and create with tracks
    /// - Return the database playlist ID
    #[instrument(skip(self), fields(playlist_url))]
    pub async fn process_playlist_submission(&self, playlist_url: &str) -> Result<String> {
        // Parse and validate the playlist URL
        let playlist_id = self.parse_playlist_url(playlist_url)?;
        info!("Parsed playlist ID: {}", playlist_id);

        // Check if playlist already exists
        if let Some(existing_playlist) = self.database.get_playlist_by_spotify_id(&playlist_id).await? {
            info!("Found existing playlist in database: {}", existing_playlist.id);
            return Ok(existing_playlist.id);
        }

        info!("Playlist not found in database, fetching from Spotify");
        
        // Fetch playlist from Spotify
        let spotify_playlist = self.hitster_service.get_playlist_by_id(&playlist_id).await?;
        info!("Successfully fetched playlist from Spotify: {} with {} tracks", 
              spotify_playlist.name, spotify_playlist.tracks.len());

        // Create playlist with tracks in database
        let db_playlist_id = self.create_playlist_with_tracks(
            &playlist_id,
            &spotify_playlist.name,
            &spotify_playlist.tracks,
        ).await?;

        Ok(db_playlist_id)
    }

    /// Create a new playlist with tracks in the database
    /// 
    /// This is a business operation that should be called after fetching
    /// playlist data from Spotify
    #[instrument(skip(self, tracks))]
    pub async fn create_playlist_with_tracks(
        &self,
        spotify_id: &str,
        name: &str,
        tracks: &[Track],
    ) -> Result<String> {
        info!("Creating playlist '{}' with {} tracks", name, tracks.len());

        // Convert domain tracks to infrastructure tracks
        let infrastructure_tracks: Vec<NewTrack> = tracks
            .iter()
            .enumerate()
            .map(|(i, track)| NewTrack {
                playlist_id: "".to_string(), // Will be set by transaction
                title: track.title.clone(),
                artist: track.artist.clone(),
                year: track.year.clone(),
                spotify_url: track.spotify_url.clone(),
                position: i as i32,
            })
            .collect();

        // Use transactional database operation
        let playlist = self
            .database
            .create_playlist_with_tracks(spotify_id, name, &infrastructure_tracks)
            .await?;

        info!("Successfully created playlist with ID: {}", playlist.id);
        Ok(playlist.id)
    }

    /// Parse and validate a playlist URL
    fn parse_playlist_url(&self, url: &str) -> Result<String> {
        // Use the domain model's parsing logic
        let playlist_id: PlaylistId = url.parse()?;
        Ok(playlist_id.as_str().to_string())
    }

    /// Get a playlist by database ID
    pub async fn get_playlist_by_id(&self, _id: &str) -> Result<Option<Playlist>> {
        // This would need to be implemented to convert from infrastructure to domain model
        // For now, return None
        Ok(None)
    }

    /// Get tracks for a playlist
    pub async fn get_tracks_for_playlist(&self, _playlist_id: &str) -> Result<Vec<Track>> {
        // This would need to be implemented to convert from infrastructure to domain model
        // For now, return empty vector
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_service_creation() {
        // This would need a mock database for testing
        // For now, just test that the struct can be created
        // let service = PlaylistService::new(/* database */);
        // assert!(true);
    }
}