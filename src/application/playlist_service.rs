use crate::infrastructure::{Database, SpotifyService};
use crate::application::models::{Playlist, PlaylistId, Track, Pdf, PdfSide, JobId};
use crate::infrastructure::NewTrack;
use crate::domain::errors::{DomainError, DomainResult};
use tracing::info;
use std::sync::Arc;
use std::str::FromStr;

/// Application service for playlist management business logic
#[derive(Clone)]
pub struct PlaylistService {
    database: Arc<Database>,
    spotify_service: Arc<SpotifyService>,
}

impl PlaylistService {
    pub fn new(database: Arc<Database>, spotify_service: Arc<SpotifyService>) -> Self {
        Self {
            database,
            spotify_service,
        }
    }

    /// Get a playlist by ID
    pub async fn get_playlist(&self, id: &PlaylistId) -> DomainResult<Playlist> {
        info!("Getting playlist: {}", id);
        
        let db_playlist = self.database.get_playlist_by_id(id.as_str()).await?
            .ok_or_else(|| DomainError::PlaylistNotFound(id.as_str().to_string()))?;
        
        let tracks = self.database.get_tracks_by_playlist_id(id.as_str()).await?;
        let domain_tracks = tracks.into_iter().map(|t| Track {
            title: t.title,
            artist: t.artist,
            year: t.year,
            spotify_url: t.spotify_url,
        }).collect();
        
        let playlist = Playlist::with_tracks(
            PlaylistId(db_playlist.id),
            db_playlist.name,
            domain_tracks,
        );
        
        Ok(playlist)
    }

    /// Generate PDFs for a playlist
    pub async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> DomainResult<JobId> {
        info!("Generating PDFs for playlist: {}", id);
        
        // Create a job for PDF generation
        let job_id = JobId::new();
        let _ = self.database.create_job(id.as_str()).await?;
        
        Ok(job_id)
    }

    /// Get PDF for a playlist
    pub async fn get_playlist_pdf(&self, id: &PlaylistId, side: PdfSide) -> DomainResult<Pdf> {
        info!("Getting PDF for playlist: {}, side: {:?}", id, side);
        
        // Get the latest job for this playlist
        let job = self.database.get_latest_job_for_playlist(id.as_str()).await?
            .ok_or_else(|| DomainError::JobNotFound("No job found for playlist".to_string()))?;
        
        // Get the PDF path based on the side
        let pdf_path = match side {
            PdfSide::Front => job.front_pdf_path,
            PdfSide::Back => job.back_pdf_path,
        }.ok_or_else(|| DomainError::PdfNotFound("PDF not available".to_string()))?;
        
        // Read the PDF file
        let pdf_bytes = tokio::fs::read(&pdf_path).await
            .map_err(|_| DomainError::PdfNotFound("Failed to read PDF file".to_string()))?;
        
        Ok(Pdf::new(pdf_bytes))
    }

    /// Refetch playlist from Spotify
    pub async fn refetch_playlist(&self, id: &PlaylistId) -> DomainResult<()> {
        info!("Refetching playlist: {}", id);
        
        // Get the current playlist from database
        let db_playlist = self.database.get_playlist_by_id(id.as_str()).await?
            .ok_or_else(|| DomainError::PlaylistNotFound(id.as_str().to_string()))?;
        
        // Refetch from Spotify
        let spotify_playlist = self.spotify_service.get_playlist(PlaylistId::from_str(&db_playlist.spotify_id)?).await?;
        
        // Delete existing tracks for this playlist
        self.database.delete_tracks_for_playlist(id.as_str()).await?;
        
        // Create new tracks
        let infrastructure_tracks: Vec<NewTrack> = spotify_playlist.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| NewTrack {
                playlist_id: id.as_str().to_string(),
                title: track.title.clone(),
                artist: track.artist.clone(),
                year: track.year.clone(),
                spotify_url: track.spotify_url.clone(),
                position: i as i32,
            })
            .collect();
        
        // Insert new tracks
        self.database.create_tracks(&infrastructure_tracks).await?;
        
        // Update playlist name if it changed
        if db_playlist.name != spotify_playlist.name {
            // TODO: Update playlist name in database
        }
        
        Ok(())
    }
}