use crate::application::IPlaylistRepository;
use crate::domain::{Job, JobId, JobType, Playlist, PlaylistId, SpotifyId};
use crate::Settings;

#[derive(Clone)]
pub struct PlaylistRepository {}

impl PlaylistRepository {
    pub async fn new(settings: &Settings) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl IPlaylistRepository for PlaylistRepository {
    async fn create(&self, playlist: &Playlist) -> anyhow::Result<Playlist> {
        todo!()
    }

    async fn get(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>> {
        todo!()
    }

    async fn get_by_spotify_id(&self, spotify_id: &SpotifyId) -> anyhow::Result<Option<Playlist>> {
        todo!()
    }

    async fn get_jobs(&self, playlist_id: &PlaylistId) -> anyhow::Result<Option<Vec<Job>>> {
        todo!()
    }
}