use crate::domain::{Playlist, PlaylistId, Pdf, PdfSide, JobId, SpotifyId};
use tracing::info;
use crate::application::{IPlaylistRepository, ISpotifyClient};

#[trait_variant::make(IPlaylistService: Send)]
pub trait _IPlaylistService: Send + Sync {
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Playlist>;
    async fn get_playlist(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>>;
    async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<JobId>;
    async fn get_playlist_pdf(&self, id: &PlaylistId, side: PdfSide) -> anyhow::Result<Pdf>;
    async fn refetch_playlist(&self, id: &PlaylistId) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct PlaylistService<SpotifyClient, PlaylistRepository>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
{
    spotify_client: SpotifyClient,
    playlist_repository: PlaylistRepository,
}

impl<SpotifyClient, PlaylistRepository> PlaylistService<SpotifyClient, PlaylistRepository>
where
      PlaylistRepository: IPlaylistRepository,
      SpotifyClient: ISpotifyClient,
{
    pub fn new(playlist_repository: PlaylistRepository, spotify_client: SpotifyClient) -> Self {
        Self {
            spotify_client,
            playlist_repository,
        }
    }
}

impl<SpotifyClient, PlaylistRepository>IPlaylistService for PlaylistService<SpotifyClient, PlaylistRepository>
where
PlaylistRepository: IPlaylistRepository,
SpotifyClient: ISpotifyClient,
{
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Playlist> {
        if let Some(existing) = self.playlist_repository.get_by_spotify_id(id).await? {
            info!("Playlist with Spotify ID {} already exists with ID {}", id, existing.id);
            return Ok(existing);
        }

        let playlist = match self.spotify_client.get_playlist(id).await? {
            Some(p) => p,
            None => {
                anyhow::bail!("Playlist with Spotify ID {} not found in Spotify", id);
            }
        };

        let created = self.playlist_repository.create(&playlist).await?;
        info!("Created new playlist with ID {} from Spotify ID {}", created.id, id);
        Ok(created)
    }

    async fn get_playlist(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>> {
        todo!()
    }

    async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<JobId> {
        todo!()
    }

    async fn get_playlist_pdf(&self, id: &PlaylistId, side: PdfSide) -> anyhow::Result<Pdf> {
        todo!()
    }

    async fn refetch_playlist(&self, id: &PlaylistId) -> anyhow::Result<()> {
        todo!()
    }
}