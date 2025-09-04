use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub spotify_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub playlist_id: String,
    pub status: JobStatus,
    pub front_pdf_path: Option<String>,
    pub back_pdf_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub playlist_id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
    pub position: i32,
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        // Create database connection options with automatic file creation
        let options = SqliteConnectOptions::new()
            .filename(database_url)
            .create_if_missing(true);
        
        let pool = SqlitePool::connect_with(options).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    // Playlist operations
    pub async fn create_playlist(
        &self,
        spotify_id: &str,
        name: &str,
    ) -> anyhow::Result<Playlist> {
        let id = Uuid::new_v4().to_string();
        let result = sqlx::query_as(
            r#"
            INSERT INTO playlists (id, spotify_id, name)
            VALUES (?, ?, ?)
            RETURNING id, spotify_id, name, created_at, updated_at
            "#,
        )
        .bind(&id)
        .bind(spotify_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_playlist_by_spotify_id(&self, spotify_id: &str) -> anyhow::Result<Option<Playlist>> {
        let result = sqlx::query_as(
            r#"
            SELECT id, spotify_id, name, created_at, updated_at
            FROM playlists
            WHERE spotify_id = ?
            "#,
        )
        .bind(spotify_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_playlist_by_id(&self, id: &str) -> anyhow::Result<Option<Playlist>> {
        let result = sqlx::query_as(
            r#"
            SELECT id, spotify_id, name, created_at, updated_at
            FROM playlists
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Transactional method to create a playlist and its tracks in a single transaction
    pub async fn create_playlist_with_tracks(
        &self,
        spotify_id: &str,
        name: &str,
        tracks: &[NewTrack],
    ) -> anyhow::Result<Playlist> {
        let mut tx = self.pool.begin().await?;
        
        // Create playlist
        let playlist_id = Uuid::new_v4().to_string();
        let playlist = sqlx::query_as(
            r#"
            INSERT INTO playlists (id, spotify_id, name)
            VALUES (?, ?, ?)
            RETURNING id, spotify_id, name, created_at, updated_at
            "#,
        )
        .bind(&playlist_id)
        .bind(spotify_id)
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;
        
        // Create all tracks
        for track in tracks {
            let track_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO tracks (id, playlist_id, title, artist, year, spotify_url, position)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&track_id)
            .bind(&playlist_id)
            .bind(&track.title)
            .bind(&track.artist)
            .bind(&track.year)
            .bind(&track.spotify_url)
            .bind(track.position)
            .execute(&mut *tx)
            .await?;
        }
        
        // Commit the transaction
        tx.commit().await?;
        
        Ok(playlist)
    }


    // Job operations
    pub async fn create_job(&self, playlist_id: &str) -> anyhow::Result<Job> {
        let id = Uuid::new_v4().to_string();
        let result = sqlx::query_as(
            r#"
            INSERT INTO jobs (id, playlist_id, status)
            VALUES (?, ?, ?)
            RETURNING id, playlist_id, status, front_pdf_path, back_pdf_path, created_at, completed_at
            "#,
        )
        .bind(&id)
        .bind(playlist_id)
        .bind(JobStatus::Pending)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_job_by_id(&self, job_id: &str) -> anyhow::Result<Option<Job>> {
        let result = sqlx::query_as(
            r#"
            SELECT id, playlist_id, status, front_pdf_path, back_pdf_path, created_at, completed_at
            FROM jobs
            WHERE id = ?
            "#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_latest_job_for_playlist(&self, playlist_id: &str) -> anyhow::Result<Option<Job>> {
        let result = sqlx::query_as(
            r#"
            SELECT id, playlist_id, status, front_pdf_path, back_pdf_path, created_at, completed_at
            FROM jobs
            WHERE playlist_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(playlist_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_job_status(
        &self,
        job_id: &str,
        status: JobStatus,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?, completed_at = CASE WHEN ? = 'completed' THEN CURRENT_TIMESTAMP ELSE NULL END
            WHERE id = ?
            "#,
        )
        .bind(&status)
        .bind(match status {
            JobStatus::Completed => "completed",
            _ => "other",
        })
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_job_pdf_paths(
        &self,
        job_id: &str,
        front_pdf_path: Option<&str>,
        back_pdf_path: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET front_pdf_path = ?, back_pdf_path = ?
            WHERE id = ?
            "#,
        )
        .bind(front_pdf_path)
        .bind(back_pdf_path)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Track operations
    pub async fn create_tracks(
        &self,
        tracks: &[NewTrack],
    ) -> anyhow::Result<()> {
        for track in tracks {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO tracks (id, playlist_id, title, artist, year, spotify_url, position)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&id)
            .bind(&track.playlist_id)
            .bind(&track.title)
            .bind(&track.artist)
            .bind(&track.year)
            .bind(&track.spotify_url)
            .bind(track.position)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_tracks_by_playlist_id(&self, playlist_id: &str) -> anyhow::Result<Vec<Track>> {
        let result = sqlx::query_as(
            r#"
            SELECT id, playlist_id, title, artist, year, spotify_url, position
            FROM tracks
            WHERE playlist_id = ?
            ORDER BY position ASC
            "#,
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_tracks_for_playlist(&self, playlist_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM tracks
            WHERE playlist_id = ?
            "#,
        )
        .bind(playlist_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct NewTrack {
    pub playlist_id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub spotify_url: String,
    pub position: i32,
}