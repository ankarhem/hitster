use crate::application::models::PlaylistId;

/// Types of jobs that can be created
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobType {
    /// Generate PDFs for a playlist
    GeneratePlaylistPdf {
        /// The playlist ID to generate PDFs for
        id: PlaylistId,
    },
}

impl JobType {
    /// Get a string representation of the job type
    pub fn as_str(&self) -> &'static str {
        match self {
            JobType::GeneratePlaylistPdf { .. } => "generate_playlist_pdf",
        }
    }
    
    /// Get the playlist ID associated with this job type (if any)
    pub fn playlist_id(&self) -> Option<&PlaylistId> {
        match self {
            JobType::GeneratePlaylistPdf { id } => Some(id),
        }
    }
    
    /// Check if this job type is for a specific playlist
    pub fn is_for_playlist(&self, playlist_id: &PlaylistId) -> bool {
        match self {
            JobType::GeneratePlaylistPdf { id } => id == playlist_id,
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::GeneratePlaylistPdf { id } => {
                write!(f, "GeneratePlaylistPdf({})", id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::models::PlaylistId;

    #[test]
    fn test_job_type_generate_playlist_pdf() {
        let playlist_id = PlaylistId("test-playlist-id".to_string());
        let job_type = JobType::GeneratePlaylistPdf { 
            id: playlist_id.clone() 
        };
        
        assert_eq!(job_type.as_str(), "generate_playlist_pdf");
        assert_eq!(job_type.playlist_id(), Some(&playlist_id));
        assert!(job_type.is_for_playlist(&playlist_id));
        
        let other_id = PlaylistId("other-id".to_string());
        assert!(!job_type.is_for_playlist(&other_id));
    }

    #[test]
    fn test_job_type_display() {
        let playlist_id = PlaylistId("test-playlist-id".to_string());
        let job_type = JobType::GeneratePlaylistPdf { id: playlist_id };
        assert_eq!(job_type.to_string(), "GeneratePlaylistPdf(test-playlist-id)");
    }

    #[test]
    fn test_job_type_playlist_id_none() {
        // All current job types have playlist IDs, but this tests the interface
        let playlist_id = PlaylistId("test-playlist-id".to_string());
        let job_type = JobType::GeneratePlaylistPdf { id: playlist_id };
        assert!(job_type.playlist_id().is_some());
    }
}