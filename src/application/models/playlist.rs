use std::str::FromStr;

/// Wrapper for Spotify playlist ID with validation
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistId(String);

impl PlaylistId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn inner(&self) -> &String {
        &self.0
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
        
        Ok(PlaylistId(id.to_string()))
    }
}

/// Represents a Spotify playlist with tracks
#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub tracks: Vec<Track>,
}

/// Represents a song card with all necessary information
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
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
}