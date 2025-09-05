use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::domain::{Playlist, PlaylistId, SpotifyId, Track};
use std::str::FromStr;

#[derive(FromRow, Debug, Clone)]
pub struct PlaylistEntity {
    pub id: String,
    pub spotify_id: Option<String>,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(FromRow, Debug, Clone)]
pub struct TrackEntity {
    pub id: String,
    pub playlist_id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
    pub position: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct JobEntity {
    pub id: String,
    pub playlist_id: String,
    pub status: String,
    pub front_pdf_path: Option<String>,
    pub back_pdf_path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<PlaylistEntity> for Playlist {
    fn from(entity: PlaylistEntity) -> Self {
        Self {
            id: PlaylistId::from_str(&entity.id).unwrap_or_else(|_| PlaylistId::new().unwrap()),
            spotify_id: entity.spotify_id.and_then(|s| SpotifyId::from_str(&s).ok()),
            name: entity.name,
            tracks: Vec::new(), // Tracks will be loaded separately
        }
    }
}

impl From<TrackEntity> for Track {
    fn from(entity: TrackEntity) -> Self {
        Self {
            title: entity.title,
            artist: entity.artist,
            year: entity.year.parse().unwrap_or(0),
            spotify_url: entity.spotify_url,
        }
    }
}

impl From<(PlaylistEntity, Vec<TrackEntity>)> for Playlist {
    fn from((playlist_entity, track_entities): (PlaylistEntity, Vec<TrackEntity>)) -> Self {
        let mut playlist = Playlist::from(playlist_entity);
        playlist.tracks = track_entities.into_iter().map(Track::from).collect();
        playlist
    }
}
