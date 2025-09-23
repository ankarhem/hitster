use crate::domain::{Playlist, SpotifyId};

#[trait_variant::make(ISpotifyClient: Send)]
pub trait _ISpotifyClient: Clone + Send + Sync + 'static {
    async fn get_playlist(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>>;
    async fn get_playlist_with_tracks(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>>;
}
