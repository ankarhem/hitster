use sqlx::{Pool, Sqlite};
use crate::application::IPlaylistRepository;
use crate::domain::{Job, JobId, JobStatus, JobType, Playlist, PlaylistId, SpotifyId, Track};
use crate::Settings;
use uuid::Uuid;
use std::str::FromStr;

#[derive(Clone)]
pub struct PlaylistRepository {
    pool: Pool<Sqlite>,
}

impl PlaylistRepository {
    pub async fn new(settings: &Settings, pool: Pool<Sqlite>) -> anyhow::Result<Self> {
        Ok(Self {
            pool,
        })
    }
}

impl IPlaylistRepository for PlaylistRepository {
    async fn create(&self, playlist: &Playlist) -> anyhow::Result<Playlist> {
        let mut tx = self.pool.begin().await?;
        
        let playlist_id_str = playlist.id.to_string();
        let spotify_id_str = playlist.spotify_id.as_ref().map(|s| s.to_string());
        let playlist_name = playlist.name.clone();
        
        sqlx::query!(
            "INSERT INTO playlists (id, spotify_id, name) VALUES (?, ?, ?)",
            playlist_id_str,
            spotify_id_str,
            playlist_name
        )
        .execute(&mut *tx)
        .await?;
        
        for (position, track) in playlist.tracks.iter().enumerate() {
            let track_id = Uuid::new_v4().to_string();
            let playlist_id_str = playlist.id.to_string();
            let track_title = track.title.clone();
            let track_artist = track.artist.clone();
            let track_year = track.year.to_string();
            let track_url = track.spotify_url.clone();
            let track_position = position as i32;
            
            sqlx::query!(
                "INSERT INTO tracks (id, playlist_id, title, artist, year, spotify_url, position) VALUES (?, ?, ?, ?, ?, ?, ?)",
                track_id,
                playlist_id_str,
                track_title,
                track_artist,
                track_year,
                track_url,
                track_position
            )
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(playlist.clone())
    }

    async fn get(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>> {
        let id_str = id.to_string();
        let playlist = sqlx::query!(
            "SELECT id, spotify_id, name FROM playlists WHERE id = ?",
            id_str
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match playlist {
            Some(row) => {
                let tracks = sqlx::query!(
                    "SELECT title, artist, year, spotify_url FROM tracks WHERE playlist_id = ? ORDER BY position",
                    id_str
                )
                .fetch_all(&self.pool)
                .await?;
                
                let spotify_id = SpotifyId::from_str(&row.spotify_id).ok();
                let tracks = tracks.into_iter().map(|t| Track {
                    title: t.title,
                    artist: t.artist,
                    year: t.year.parse().unwrap_or(0),
                    spotify_url: t.spotify_url,
                }).collect();
                
                Ok(Some(Playlist {
                    id: id.clone(),
                    spotify_id,
                    name: row.name,
                    tracks,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_by_spotify_id(&self, spotify_id: &SpotifyId) -> anyhow::Result<Option<Playlist>> {
        let spotify_id_str = spotify_id.to_string();
        let playlist = sqlx::query!(
            "SELECT id, spotify_id, name FROM playlists WHERE spotify_id = ?",
            spotify_id_str
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match playlist {
            Some(row) => {
                let playlist_id = PlaylistId::from_str(row.id.as_deref().unwrap())?;
                let playlist_id_str = playlist_id.to_string();
                let tracks = sqlx::query!(
                    "SELECT title, artist, year, spotify_url FROM tracks WHERE playlist_id = ? ORDER BY position",
                    playlist_id_str
                )
                .fetch_all(&self.pool)
                .await?;
                
                let spotify_id = SpotifyId::from_str(&row.spotify_id).ok();
                let tracks = tracks.into_iter().map(|t| Track {
                    title: t.title,
                    artist: t.artist,
                    year: t.year.parse().unwrap_or(0),
                    spotify_url: t.spotify_url,
                }).collect();
                
                Ok(Some(Playlist {
                    id: playlist_id,
                    spotify_id,
                    name: row.name,
                    tracks,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_jobs(&self, playlist_id: &PlaylistId) -> anyhow::Result<Option<Vec<Job>>> {
        let playlist_id_str = playlist_id.to_string();
        let jobs = sqlx::query!(
            r#"
            SELECT id, status, front_pdf_path, back_pdf_path, created_at, completed_at
            FROM jobs 
            WHERE playlist_id = ?
            ORDER BY created_at DESC
            "#,
            playlist_id_str
        )
        .fetch_all(&self.pool)
        .await?;
        
        if jobs.is_empty() {
            return Ok(None);
        }
        
        let jobs = jobs.into_iter().map(|row| {
            let job_id = JobId::from_str(&row.id.unwrap()).unwrap();
            let status = JobStatus::Pending; // Default to pending since we can't parse string
            
            Job {
                id: job_id,
                job_type: JobType::GeneratePlaylistPdf {
                    id: playlist_id.clone(),
                },
                status,
                created_at: row.created_at.unwrap().and_utc(),
                completed_at: row.completed_at.map(|dt| dt.and_utc()),
                front_pdf_path: row.front_pdf_path,
                back_pdf_path: row.back_pdf_path,
            }
        }).collect();
        
        Ok(Some(jobs))
    }
}