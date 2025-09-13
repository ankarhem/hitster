use crate::domain;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

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
    pub album_cover_url: Option<String>,
    pub position: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct JobEntity {
    pub id: Uuid,
    pub status: JobStatusEntity,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum JobStatusEntity {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "processing")]
    Processing,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
}


impl From<PlaylistEntity> for domain::Playlist {
    fn from(entity: PlaylistEntity) -> Self {
        Self {
            id: entity.id.into(),
            spotify_id: entity.spotify_id.and_then(|s| s.parse().ok()),
            name: entity.name,
            tracks: Vec::new(), // Tracks will be loaded separately
            created_at: Some(entity.created_at),
            updated_at: entity.updated_at,
        }
    }
}

impl From<TrackEntity> for domain::Track {
    fn from(entity: TrackEntity) -> Self {
        Self {
            title: entity.title,
            artist: entity.artist,
            year: entity.year,
            spotify_url: entity.spotify_url,
            album_cover_url: entity.album_cover_url,
        }
    }
}

impl From<(PlaylistEntity, Vec<TrackEntity>)> for domain::Playlist {
    fn from((playlist_entity, track_entities): (PlaylistEntity, Vec<TrackEntity>)) -> Self {
        let mut playlist: domain::Playlist = playlist_entity.into();
        playlist.tracks = track_entities
            .into_iter()
            .map(domain::Track::from)
            .collect();
        playlist
    }
}

impl From<JobEntity> for domain::Job {
    fn from(entity: JobEntity) -> Self {
        Self {
            id: entity.id.into(),
            status: entity.status.into(),
            created_at: entity.created_at,
            completed_at: entity.completed_at,
            payload: entity.payload,
            result: entity.result,
        }
    }
}

impl From<JobStatusEntity> for domain::JobStatus {
    fn from(status: JobStatusEntity) -> Self {
        match status {
            JobStatusEntity::Pending => domain::JobStatus::Pending,
            JobStatusEntity::Processing => domain::JobStatus::Processing,
            JobStatusEntity::Completed => domain::JobStatus::Completed,
            JobStatusEntity::Failed => domain::JobStatus::Failed,
        }
    }
}

// --------------  Reverse Conversions ----------------

impl From<domain::Job> for JobEntity {
    fn from(job: domain::Job) -> Self {
        Self {
            id: Uuid::from(job.id),
            status: job.status.into(),
            created_at: job.created_at,
            completed_at: job.completed_at,
            payload: job.payload,
            result: job.result,
        }
    }
}

impl From<domain::JobStatus> for JobStatusEntity {
    fn from(status: domain::JobStatus) -> Self {
        match status {
            domain::JobStatus::Pending => JobStatusEntity::Pending,
            domain::JobStatus::Processing => JobStatusEntity::Processing,
            domain::JobStatus::Completed => JobStatusEntity::Completed,
            domain::JobStatus::Failed => JobStatusEntity::Failed,
        }
    }
}

impl From<domain::Track> for TrackEntity {
    fn from(track: domain::Track) -> Self {
        Self {
            id: Uuid::new_v4(), // Will be set when saving to database
            playlist_id: Uuid::nil(), // Will be set when saving to database
            title: track.title,
            artist: track.artist,
            year: track.year,
            spotify_url: track.spotify_url,
            album_cover_url: track.album_cover_url,
            position: 0, // Will be set when saving to database
        }
    }
}
