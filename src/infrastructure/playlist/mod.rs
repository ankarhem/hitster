use crate::application::IPlaylistRepository;
use crate::domain::{Job, Playlist, PlaylistId, SpotifyId};
use crate::infrastructure::entities::{JobEntity, PlaylistEntity, TrackEntity};
use sqlx::{Pool, Sqlite, types::Uuid};

#[derive(Clone)]
pub struct PlaylistRepository {
    pool: Pool<Sqlite>,
}

impl PlaylistRepository {
    pub async fn new(pool: Pool<Sqlite>) -> anyhow::Result<Self> {
        Ok(Self { pool })
    }
}

impl IPlaylistRepository for PlaylistRepository {
    async fn create(&self, playlist: &Playlist) -> anyhow::Result<Playlist> {
        let mut tx = self.pool.begin().await?;

        let playlist_id_uuid: Uuid = playlist.id.clone().into();
        let spotify_id_str = playlist.spotify_id.as_ref().map(|s| s.to_string());
        let playlist_name = &playlist.name;

        sqlx::query!(
            "INSERT INTO playlists (id, spotify_id, name) VALUES (?, ?, ?)",
            playlist_id_uuid,
            spotify_id_str,
            playlist_name
        )
        .execute(&mut *tx)
        .await?;

        for (position, track) in playlist.tracks.iter().enumerate() {
            let track_id = Uuid::new_v4();
            let track_position = position as i32;

            sqlx::query!(
                "INSERT INTO tracks (id, playlist_id, title, artist, year, spotify_url, album_cover_url, position) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                track_id,
                playlist_id_uuid,
                track.title,
                track.artist,
                track.year,
                track.spotify_url,
                track.album_cover_url,
                track_position
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(playlist.clone())
    }

    async fn get(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>> {
        let playlist_entity = sqlx::query_as::<_, PlaylistEntity>(
            "SELECT id, spotify_id, name, created_at, updated_at FROM playlists WHERE id = ?",
        )
        .bind(Uuid::from(id.clone()))
        .fetch_optional(&self.pool)
        .await?;

        match playlist_entity {
            Some(playlist) => {
                let tracks = sqlx::query_as::<_, TrackEntity>(
                    "SELECT id, playlist_id, title, artist, year, spotify_url, album_cover_url, position FROM tracks WHERE playlist_id = ? ORDER BY position"
                )
                .bind(Uuid::from(id.clone()))
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(Playlist::from((playlist, tracks))))
            }
            None => Ok(None),
        }
    }

    async fn get_by_spotify_id(&self, spotify_id: &SpotifyId) -> anyhow::Result<Option<Playlist>> {
        let playlist_entity = sqlx::query_as::<_, PlaylistEntity>(
            "SELECT id, spotify_id, name, created_at, updated_at FROM playlists WHERE spotify_id = ?"
        )
        .bind(spotify_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match playlist_entity {
            Some(playlist) => {
                let tracks = sqlx::query_as::<_, TrackEntity>(
                    "SELECT id, playlist_id, title, artist, year, spotify_url, album_cover_url, position FROM tracks WHERE playlist_id = ? ORDER BY position"
                )
                .bind(playlist.id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(Playlist::from((playlist, tracks))))
            }
            None => Ok(None),
        }
    }

    async fn get_jobs(&self, playlist_id: &PlaylistId) -> anyhow::Result<Option<Vec<Job>>> {
        let playlist_id_str = playlist_id.to_string();

        let job_entities = sqlx::query_as::<_, JobEntity>(
            "SELECT id, status, created_at, completed_at, payload, result FROM jobs 
             WHERE json_extract(payload, '$.playlist_id') = ? 
             ORDER BY created_at DESC",
        )
        .bind(playlist_id_str)
        .fetch_all(&self.pool)
        .await?;

        if job_entities.is_empty() {
            return Ok(None);
        }

        let jobs = job_entities.into_iter().map(Job::from).collect();
        Ok(Some(jobs))
    }

    async fn update(&self, playlist: &Playlist) -> anyhow::Result<Playlist> {
        let mut tx = self.pool.begin().await?;

        let playlist_id_uuid: Uuid = playlist.id.clone().into();
        let spotify_id_str = playlist.spotify_id.as_ref().map(|s| s.to_string());
        let playlist_name = &playlist.name;
        let updated_at = playlist.updated_at;

        // Update playlist
        sqlx::query!(
            "UPDATE playlists SET spotify_id = ?, name = ?, updated_at = ? WHERE id = ?",
            spotify_id_str,
            playlist_name,
            updated_at,
            playlist_id_uuid
        )
        .execute(&mut *tx)
        .await?;

        // Delete existing tracks (we'll reinsert them)
        sqlx::query!("DELETE FROM tracks WHERE playlist_id = ?", playlist_id_uuid)
            .execute(&mut *tx)
            .await?;

        // Insert updated tracks
        for (position, track) in playlist.tracks.iter().enumerate() {
            let track_id = Uuid::new_v4();
            let track_position = position as i32;

            sqlx::query!(
                "INSERT INTO tracks (id, playlist_id, title, artist, year, spotify_url, album_cover_url, position) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                track_id,
                playlist_id_uuid,
                track.title,
                track.artist,
                track.year,
                track.spotify_url,
                track.album_cover_url,
                track_position
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(playlist.clone())
    }
}
