use sqlx::{FromRow};
use chrono::{DateTime, Utc};
use crate::domain;
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
    pub position: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct JobEntity {
    pub id: Uuid,
    pub status: JobStatusEntity,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub kind: JobKindEntity,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatusEntity {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, sqlx::Type)]
pub enum JobKindEntity {
    GeneratePdfs,
    RefetchPlaylist,
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
        }
    }
}

impl From<(PlaylistEntity, Vec<TrackEntity>)> for domain::Playlist {
    fn from((playlist_entity, track_entities): (PlaylistEntity, Vec<TrackEntity>)) -> Self {
        let mut playlist: domain::Playlist = playlist_entity.into();
        playlist.tracks = track_entities.into_iter().map(domain::Track::from).collect();
        playlist
    }
}

impl From<JobEntity> for domain::Job {
    fn from(entity: JobEntity) -> Self {
        Self {
            id: entity.id.into(),
            kind: entity.kind.into(),
            status: entity.status.into(),
            created_at: entity.created_at,
            completed_at: entity.completed_at,
            payload: entity.payload,
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

impl From<JobKindEntity> for domain::JobKind {
    fn from(kind: JobKindEntity) -> Self {
        match kind {
            JobKindEntity::GeneratePdfs => domain::JobKind::GeneratePdfs,
            JobKindEntity::RefetchPlaylist => domain::JobKind::RefetchPlaylist,
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
            kind: job.kind.into(),
            payload: job.payload,
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

impl From<domain::JobKind> for JobKindEntity {
    fn from(kind: domain::JobKind) -> Self {
        match kind {
            domain::JobKind::GeneratePdfs => JobKindEntity::GeneratePdfs,
            domain::JobKind::RefetchPlaylist => JobKindEntity::RefetchPlaylist,
        }
    }
}
