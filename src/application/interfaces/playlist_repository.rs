use crate::domain::{Job, Playlist, PlaylistId, SpotifyId};

#[trait_variant::make(IPlaylistRepository: Send)]
pub trait _IPlaylistRepository: Send + Sync {
    async fn create(&self, playlist: &Playlist) -> anyhow::Result<Playlist>;
    async fn get(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>>;
    async fn get_by_spotify_id(&self, spotify_id: &SpotifyId) -> anyhow::Result<Option<Playlist>>;
    async fn get_jobs(&self, playlist_id: &PlaylistId) -> anyhow::Result<Option<Vec<Job>>>;
}