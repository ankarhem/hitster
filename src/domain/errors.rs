use std::fmt;

/// Domain-specific errors
#[derive(Debug, Clone)]
pub enum DomainError {
    /// Playlist not found
    PlaylistNotFound(String),
    /// Invalid playlist ID
    InvalidPlaylistId(String),
    /// Job not found
    JobNotFound(String),
    /// Invalid job ID
    InvalidJobId(String),
    /// PDF generation failed
    PdfGenerationFailed(String),
    /// PDF not found
    PdfNotFound(String),
    /// Invalid PDF side
    InvalidPdfSide(String),
    /// Spotify API error
    SpotifyError(String),
    /// Validation error
    ValidationError(String),
    /// Business rule violation
    BusinessRuleViolation(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::PlaylistNotFound(id) => write!(f, "Playlist not found: {}", id),
            DomainError::InvalidPlaylistId(id) => write!(f, "Invalid playlist ID: {}", id),
            DomainError::JobNotFound(id) => write!(f, "Job not found: {}", id),
            DomainError::InvalidJobId(id) => write!(f, "Invalid job ID: {}", id),
            DomainError::PdfGenerationFailed(msg) => write!(f, "PDF generation failed: {}", msg),
            DomainError::PdfNotFound(id) => write!(f, "PDF not found: {}", id),
            DomainError::InvalidPdfSide(side) => write!(f, "Invalid PDF side: {}", side),
            DomainError::SpotifyError(msg) => write!(f, "Spotify API error: {}", msg),
            DomainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DomainError::BusinessRuleViolation(msg) => write!(f, "Business rule violation: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Convert from anyhow errors to domain errors
impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        DomainError::ValidationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        assert_eq!(
            DomainError::PlaylistNotFound("test".to_string()).to_string(),
            "Playlist not found: test"
        );
        assert_eq!(
            DomainError::InvalidPlaylistId("test".to_string()).to_string(),
            "Invalid playlist ID: test"
        );
        assert_eq!(
            DomainError::JobNotFound("test".to_string()).to_string(),
            "Job not found: test"
        );
        assert_eq!(
            DomainError::PdfGenerationFailed("test".to_string()).to_string(),
            "PDF generation failed: test"
        );
    }

    #[test]
    fn test_domain_error_to_anyhow() {
        let domain_err = DomainError::PlaylistNotFound("test".to_string());
        let anyhow_err: anyhow::Error = domain_err.into();
        assert_eq!(anyhow_err.to_string(), "Playlist not found: test");
    }

    #[test]
    fn test_anyhow_to_domain_error() {
        let anyhow_err = anyhow::anyhow!("test error");
        let domain_err: DomainError = anyhow_err.into();
        assert!(matches!(domain_err, DomainError::ValidationError(_)));
    }
}