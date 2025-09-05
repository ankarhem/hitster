use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::domain::{Playlist, PlaylistId, SpotifyId, Track, Job, JobId, JobStatus, JobType};
use uuid::Uuid;
use std::str::FromStr;

#[derive(FromRow, Debug, Clone)]
pub struct PlaylistEntity {
    pub id: Uuid,
    pub spotify_id: Option<String>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(FromRow, Debug, Clone)]
pub struct TrackEntity {
    pub id: Uuid,
    pub playlist_id: Uuid,
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub spotify_url: String,
    pub position: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct JobEntity {
    pub id: Uuid,
    pub playlist_id: Uuid,
    pub status: JobStatus,
    pub front_pdf_path: Option<String>,
    pub back_pdf_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<PlaylistEntity> for Playlist {
    fn from(entity: PlaylistEntity) -> Self {
        Self {
            id: PlaylistId::from(entity.id),
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
            year: entity.year,
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

impl From<JobEntity> for Job {
    fn from(entity: JobEntity) -> Self {
        let job_type = JobType::GeneratePlaylistPdf {
            id: PlaylistId::from(entity.playlist_id),
        };
        
        Self {
            id: JobId::from(entity.id),
            job_type,
            status: entity.status,
            created_at: entity.created_at,
            completed_at: entity.completed_at,
            front_pdf_path: entity.front_pdf_path,
            back_pdf_path: entity.back_pdf_path,
        }
    }
}
