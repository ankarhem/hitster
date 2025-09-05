use crate::domain::{Playlist, SpotifyId};

#[trait_variant::make(ISpotifyClient: Send)]
pub trait _ISpotifyClient: Send + Sync {
    async fn get_playlist(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>>;
}