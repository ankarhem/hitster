use std::fmt::Formatter;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::SpotifyId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistId(Uuid);
impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PlaylistId {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self(Uuid::new_v4()))
    }
}

impl From<PlaylistId> for Uuid {
    fn from(value: PlaylistId) -> Self {
        value.0
    }
}

impl FromStr for PlaylistId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(Self(uuid))
    }
}

impl From<Uuid> for PlaylistId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Represents a Spotify playlist with tracks
#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: PlaylistId,
    pub spotify_id: Option<SpotifyId>,
    pub name: String,
    pub tracks: Vec<Track>,
}


impl Playlist {
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}

/// Represents a song card with all necessary information
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub spotify_url: String,
}