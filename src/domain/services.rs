use std::str::FromStr;
use crate::application::models::{PlaylistId, JobId, JobType};
use crate::domain::errors::{DomainError, DomainResult};
use crate::application::models::{Playlist, Track};

/// Domain service for playlist business logic
#[derive(Clone)]
pub struct PlaylistDomainService {
    // Domain services don't have dependencies on infrastructure
    // They contain pure business logic
}

impl Default for PlaylistDomainService {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistDomainService {
    /// Create a new playlist domain service
    pub fn new() -> Self {
        Self {}
    }
    
    /// Validate a playlist ID
    pub fn validate_playlist_id(&self, id: &str) -> DomainResult<PlaylistId> {
        PlaylistId::from_str(id).map_err(|_| DomainError::InvalidPlaylistId(id.to_string()))
    }
    
    /// Validate playlist data
    pub fn validate_playlist(&self, playlist: &Playlist) -> DomainResult<()> {
        if !playlist.id.is_valid() {
            return Err(DomainError::InvalidPlaylistId(playlist.id.as_str().to_string()));
        }
        
        if playlist.name.trim().is_empty() {
            return Err(DomainError::ValidationError("Playlist name cannot be empty".to_string()));
        }
        
        // Validate tracks
        for (index, track) in playlist.tracks.iter().enumerate() {
            if !track.is_valid() {
                return Err(DomainError::ValidationError(format!("Track at index {} is invalid", index)));
            }
        }
        
        Ok(())
    }
    
    /// Check if a playlist can be processed for PDF generation
    pub fn can_generate_pdfs(&self, playlist: &Playlist) -> DomainResult<()> {
        self.validate_playlist(playlist)?;
        
        if playlist.is_empty() {
            return Err(DomainError::BusinessRuleViolation(
                "Cannot generate PDFs for empty playlist".to_string()
            ));
        }
        
        if playlist.track_count() > 1000 {
            return Err(DomainError::BusinessRuleViolation(
                "Playlist too large for PDF generation (max 1000 tracks)".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get a preview of the playlist for display purposes
    pub fn get_playlist_preview(&self, playlist: &Playlist) -> PlaylistPreview {
        PlaylistPreview {
            id: playlist.id.clone(),
            name: playlist.name.clone(),
            track_count: playlist.track_count(),
            sample_tracks: playlist.tracks.iter().take(3).cloned().collect(),
        }
    }
}

/// Preview of a playlist for display purposes
#[derive(Debug, Clone)]
pub struct PlaylistPreview {
    pub id: PlaylistId,
    pub name: String,
    pub track_count: usize,
    pub sample_tracks: Vec<Track>,
}

/// Domain service for job business logic
#[derive(Clone)]
pub struct JobDomainService {
    // Domain services don't have dependencies on infrastructure
    // They contain pure business logic
}

impl Default for JobDomainService {
    fn default() -> Self {
        Self::new()
    }
}

impl JobDomainService {
    /// Create a new job domain service
    pub fn new() -> Self {
        Self {}
    }
    
    /// Validate a job ID
    pub fn validate_job_id(&self, id: &str) -> DomainResult<JobId> {
        JobId::from_str(id).map_err(|_| DomainError::InvalidJobId(id.to_string()))
    }
    
    /// Validate job type
    pub fn validate_job_type(&self, job_type: &JobType) -> DomainResult<()> {
        match job_type {
            JobType::GeneratePlaylistPdf { id } => {
                if !id.is_valid() {
                    return Err(DomainError::InvalidPlaylistId(id.as_str().to_string()));
                }
                Ok(())
            }
        }
    }
    
    /// Check if a job can be created
    pub fn can_create_job(&self, job_type: &JobType) -> DomainResult<()> {
        self.validate_job_type(job_type)?;
        // Additional business rules can be added here
        Ok(())
    }
    
    /// Get job type description
    pub fn get_job_type_description(&self, job_type: &JobType) -> String {
        match job_type {
            JobType::GeneratePlaylistPdf { id } => {
                format!("Generate PDFs for playlist {}", id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_playlist_id_valid() {
        let service = PlaylistDomainService::new();
        let result = service.validate_playlist_id("3vnwX8FuGWpGgQX4hBa8sE");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_playlist_id_invalid() {
        let service = PlaylistDomainService::new();
        let result = service.validate_playlist_id("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidPlaylistId(_)));
    }

    #[test]
    fn test_validate_playlist_valid() {
        let service = PlaylistDomainService::new();
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let playlist = Playlist::with_tracks(
            id,
            "Test Playlist".to_string(),
            vec![Track::new("Song".to_string(), "Artist".to_string(), "2023".to_string(), "url".to_string())],
        );
        let result = service.validate_playlist(&playlist);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_playlist_empty_name() {
        let service = PlaylistDomainService::new();
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let playlist = Playlist::new(id, "".to_string());
        let result = service.validate_playlist(&playlist);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_can_generate_pdfs_valid() {
        let service = PlaylistDomainService::new();
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let playlist = Playlist::with_tracks(
            id,
            "Test Playlist".to_string(),
            vec![Track::new("Song".to_string(), "Artist".to_string(), "2023".to_string(), "url".to_string())],
        );
        let result = service.can_generate_pdfs(&playlist);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_generate_pdfs_empty() {
        let service = PlaylistDomainService::new();
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let playlist = Playlist::new(id, "Test Playlist".to_string());
        let result = service.can_generate_pdfs(&playlist);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::BusinessRuleViolation(_)));
    }

    #[test]
    fn test_validate_job_id_valid() {
        let service = JobDomainService::new();
        let job_id = JobId::new();
        let result = service.validate_job_id(job_id.as_str());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_job_id_invalid() {
        let service = JobDomainService::new();
        let result = service.validate_job_id("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidJobId(_)));
    }

    #[test]
    fn test_validate_job_type_valid() {
        let service = JobDomainService::new();
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let job_type = JobType::GeneratePlaylistPdf { id };
        let result = service.validate_job_type(&job_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_job_type_description() {
        let service = JobDomainService::new();
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let job_type = JobType::GeneratePlaylistPdf { id };
        let description = service.get_job_type_description(&job_type);
        assert!(description.contains("Generate PDFs for playlist"));
    }
}