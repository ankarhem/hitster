use std::str::FromStr;

/// Wrapper for Spotify playlist ID with validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistId(pub String);

impl PlaylistId {
    /// Create a new PlaylistId from a string with validation
    pub fn new(id: String) -> Result<Self, anyhow::Error> {
        if id.trim().is_empty() {
            return Err(anyhow::anyhow!("PlaylistId cannot be empty"));
        }
        
        // Basic validation for Spotify playlist ID format
        // Spotify IDs are typically 22 characters and contain alphanumeric characters
        if id.len() != 22 || !id.chars().all(|c| c.is_alphanumeric()) {
            return Err(anyhow::anyhow!("Invalid Spotify playlist ID format"));
        }
        
        Ok(Self(id))
    }
    
    /// Get the PlaylistId as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the PlaylistId as a String
    pub fn into_string(self) -> String {
        self.0
    }
    
    /// Get a reference to the inner String
    pub fn inner(&self) -> &String {
        &self.0
    }
    
    /// Check if this is a valid Spotify playlist ID
    pub fn is_valid(&self) -> bool {
        self.0.len() == 22 && self.0.chars().all(|c| c.is_alphanumeric())
    }
    
    /// Get the URL for this playlist on Spotify
    pub fn to_spotify_url(&self) -> String {
        format!("https://open.spotify.com/playlist/{}", self.0)
    }
    
    /// Get the Spotify URI for this playlist
    pub fn to_spotify_uri(&self) -> String {
        format!("spotify:playlist:{}", self.0)
    }
}

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PlaylistId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        
        if trimmed.is_empty() {
            return Err(anyhow::anyhow!("Empty playlist ID"));
        }
        
        // Extract ID from URL if needed
        let id = if trimmed.contains("open.spotify.com/playlist/") {
            // Handle https://open.spotify.com/playlist/ID format
            let parts: Vec<&str> = trimmed.split('/').collect();
            if let Some(last_part) = parts.last() {
                last_part.split('?').next().unwrap_or(last_part)
            } else {
                return Err(anyhow::anyhow!("Invalid Spotify playlist URL"));
            }
        } else if trimmed.contains("spotify:playlist:") {
            // Handle spotify:playlist:ID format
            let parts: Vec<&str> = trimmed.split(':').collect();
            if let Some(id) = parts.last() {
                id.split('?').next().unwrap_or(id)
            } else {
                return Err(anyhow::anyhow!("Invalid Spotify playlist URI"));
            }
        } else if !trimmed.contains('/') && !trimmed.contains(':') {
            // Handle raw ID format
            trimmed.split('?').next().unwrap_or(trimmed)
        } else {
            return Err(anyhow::anyhow!("Invalid Spotify playlist format"));
        };
        
        if id.is_empty() {
            return Err(anyhow::anyhow!("Empty playlist ID after parsing"));
        }
        
        // Use the new constructor for validation
        Self::new(id.to_string())
    }
}

/// Represents a Spotify playlist with tracks
#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub tracks: Vec<Track>,
}

impl Playlist {
    /// Create a new playlist
    pub fn new(id: PlaylistId, name: String) -> Self {
        Self {
            id,
            name,
            tracks: Vec::new(),
        }
    }
    
    /// Create a new playlist with tracks
    pub fn with_tracks(id: PlaylistId, name: String, tracks: Vec<Track>) -> Self {
        Self {
            id,
            name,
            tracks,
        }
    }
    
    /// Get the number of tracks in the playlist
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
    
    /// Check if the playlist is empty
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
    
    /// Get the URL for this playlist on Spotify
    pub fn spotify_url(&self) -> String {
        self.id.to_spotify_url()
    }
    
    /// Get the Spotify URI for this playlist
    pub fn spotify_uri(&self) -> String {
        self.id.to_spotify_uri()
    }
}

/// Represents a song card with all necessary information
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
}

impl Track {
    /// Create a new track
    pub fn new(title: String, artist: String, year: String, spotify_url: String) -> Self {
        Self {
            title,
            artist,
            year,
            spotify_url,
        }
    }
    
    /// Get a display name for the track
    pub fn display_name(&self) -> String {
        format!("{} - {}", self.artist, self.title)
    }
    
    /// Get the track title with year
    pub fn title_with_year(&self) -> String {
        if self.year.is_empty() {
            self.title.clone()
        } else {
            format!("{} ({})", self.title, self.year)
        }
    }
    
    /// Check if the track has valid data
    pub fn is_valid(&self) -> bool {
        !self.title.is_empty() && !self.artist.is_empty() && !self.spotify_url.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_id_from_string() {
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");
    }

    #[test]
    fn test_playlist_id_new() {
        let id = PlaylistId::new("3vnwX8FuGWpGgQX4hBa8sE".to_string()).unwrap();
        assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");
        assert!(id.is_valid());
    }

    #[test]
    fn test_playlist_id_invalid_length() {
        let result = PlaylistId::new("short".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_id_invalid_characters() {
        let result = PlaylistId::new("3vnwX8FuGWpGgQX4hBa8sE!".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_id_from_url() {
        let id = PlaylistId::from_str("https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE?si=xyz").unwrap();
        assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");
    }

    #[test]
    fn test_playlist_id_from_spotify_uri() {
        let id = PlaylistId::from_str("spotify:playlist:3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");
    }

    #[test]
    fn test_playlist_id_from_spotify_uri_with_query() {
        let id = PlaylistId::from_str("spotify:playlist:3vnwX8FuGWpGgQX4hBa8sE?si=xyz").unwrap();
        assert_eq!(id.as_str(), "3vnwX8FuGWpGgQX4hBa8sE");
    }

    #[test]
    fn test_playlist_id_spotify_url() {
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        assert_eq!(id.to_spotify_url(), "https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE");
        assert_eq!(id.to_spotify_uri(), "spotify:playlist:3vnwX8FuGWpGgQX4hBa8sE");
    }

    #[test]
    fn test_playlist_id_empty_string() {
        let result = PlaylistId::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_id_invalid_url() {
        let result = PlaylistId::from_str("https://invalid.com/playlist/xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_id_invalid_format() {
        let result = PlaylistId::from_str("https://open.spotify.com/track/xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_creation() {
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let playlist = Playlist::new(id.clone(), "Test Playlist".to_string());
        assert_eq!(playlist.name, "Test Playlist");
        assert_eq!(playlist.track_count(), 0);
        assert!(playlist.is_empty());
        assert_eq!(playlist.spotify_url(), id.to_spotify_url());
    }

    #[test]
    fn test_playlist_with_tracks() {
        let id = PlaylistId::from_str("3vnwX8FuGWpGgQX4hBa8sE").unwrap();
        let tracks = vec![
            Track::new("Song 1".to_string(), "Artist 1".to_string(), "2023".to_string(), "url1".to_string()),
            Track::new("Song 2".to_string(), "Artist 2".to_string(), "2023".to_string(), "url2".to_string()),
        ];
        let playlist = Playlist::with_tracks(id, "Test Playlist".to_string(), tracks.clone());
        assert_eq!(playlist.track_count(), 2);
        assert!(!playlist.is_empty());
        assert_eq!(playlist.tracks, tracks);
    }

    #[test]
    fn test_track_creation() {
        let track = Track::new("Test Song".to_string(), "Test Artist".to_string(), "2023".to_string(), "test_url".to_string());
        assert_eq!(track.title, "Test Song");
        assert_eq!(track.artist, "Test Artist");
        assert_eq!(track.year, "2023");
        assert_eq!(track.spotify_url, "test_url");
        assert!(track.is_valid());
        assert_eq!(track.display_name(), "Test Artist - Test Song");
        assert_eq!(track.title_with_year(), "Test Song (2023)");
    }

    #[test]
    fn test_track_validation() {
        let valid_track = Track::new("Title".to_string(), "Artist".to_string(), "2023".to_string(), "url".to_string());
        assert!(valid_track.is_valid());

        let invalid_track = Track::new("".to_string(), "Artist".to_string(), "2023".to_string(), "url".to_string());
        assert!(!invalid_track.is_valid());
    }

    #[test]
    fn test_track_title_with_year() {
        let track_with_year = Track::new("Title".to_string(), "Artist".to_string(), "2023".to_string(), "url".to_string());
        assert_eq!(track_with_year.title_with_year(), "Title (2023)");

        let track_without_year = Track::new("Title".to_string(), "Artist".to_string(), "".to_string(), "url".to_string());
        assert_eq!(track_without_year.title_with_year(), "Title");
    }
}