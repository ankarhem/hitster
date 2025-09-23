use crate::domain::{Job, Playlist, PlaylistId, SpotifyId};
use std::future::Future;

pub trait IPlaylistRepository: Clone + Send + Sync + 'static {
    fn create(&self, playlist: &Playlist) -> impl Future<Output = anyhow::Result<Playlist>> + Send;
    fn get(&self, id: &PlaylistId)
    -> impl Future<Output = anyhow::Result<Option<Playlist>>> + Send;
    fn get_by_spotify_id(
        &self,
        spotify_id: &SpotifyId,
    ) -> impl Future<Output = anyhow::Result<Option<Playlist>>> + Send;
    fn get_jobs(
        &self,
        playlist_id: &PlaylistId,
    ) -> impl Future<Output = anyhow::Result<Option<Vec<Job>>>> + Send;
    fn update(&self, playlist: &Playlist) -> impl Future<Output = anyhow::Result<Playlist>> + Send;
}
