use crate::domain::{Playlist, SpotifyId};
use std::future::Future;

pub trait ISpotifyClient: Clone + Send + Sync + 'static {
    fn get_playlist(
        &self,
        id: &SpotifyId,
    ) -> impl Future<Output = anyhow::Result<Option<Playlist>>> + Send;
    fn get_playlist_with_tracks(
        &self,
        id: &SpotifyId,
    ) -> impl Future<Output = anyhow::Result<Option<Playlist>>> + Send;
}
