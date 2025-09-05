use sqlx::FromRow;
use chrono::{DateTime, Utc};
use crate::domain::{Playlist, PlaylistId, SpotifyId, Track, Job, JobId, JobStatus, JobType};
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

impl From<JobEntity> for Job {
    fn from(entity: JobEntity) -> Self {
        let job_type = JobType::GeneratePlaylistPdf {
            id: PlaylistId::from_str(&entity.playlist_id).unwrap_or_else(|_| PlaylistId::new().unwrap()),
        };
        let status = match entity.status.as_str() {
            "pending" => JobStatus::Pending,
            "processing" => JobStatus::Processing,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            _ => JobStatus::Pending,
        };
        
        Self {
            id: JobId::from_str(&entity.id).unwrap_or_else(|_| JobId::new()),
            job_type,
            status,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            completed_at: entity.completed_at,
            front_pdf_path: entity.front_pdf_path,
            back_pdf_path: entity.back_pdf_path,
        }
    }
}
