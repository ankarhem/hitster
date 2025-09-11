use crate::application::{IJobsRepository, IPlaylistRepository, ISpotifyClient};
use crate::domain::{Job, Pdf, Playlist, PlaylistId, SpotifyId};
use std::sync::Arc;
use tracing::info;

#[trait_variant::make(IPlaylistService: Send)]
pub trait _IPlaylistService: Send + Sync {
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Playlist>;
    async fn get_playlist(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>>;
    async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<Job>;
    async fn get_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<[Pdf; 2]>;
    async fn refetch_playlist(&self, id: &PlaylistId) -> anyhow::Result<Job>;
}

#[derive(Clone)]
pub struct PlaylistService<SpotifyClient, PlaylistRepository, JobsRepository>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
    JobsRepository: IJobsRepository,
{
    spotify_client: Arc<SpotifyClient>,
    playlist_repository: Arc<PlaylistRepository>,
    #[allow(dead_code)]
    jobs_repository: Arc<JobsRepository>,
}

impl<SpotifyClient, PlaylistRepository, JobsRepository>
    PlaylistService<SpotifyClient, PlaylistRepository, JobsRepository>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
    JobsRepository: IJobsRepository,
{
    pub fn new(
        playlist_repository: Arc<PlaylistRepository>,
        spotify_client: Arc<SpotifyClient>,
        jobs_repository: Arc<JobsRepository>,
    ) -> Self {
        Self {
            spotify_client,
            playlist_repository,
            jobs_repository,
        }
    }
}

impl<SpotifyClient, PlaylistRepository, JobsRepository> IPlaylistService
    for PlaylistService<SpotifyClient, PlaylistRepository, JobsRepository>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
    JobsRepository: IJobsRepository,
{
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Playlist> {
        if let Some(existing) = self.playlist_repository.get_by_spotify_id(id).await? {
            info!(
                "Playlist with Spotify ID {} already exists with ID {}",
                id, existing.id
            );
            return Ok(existing);
        }

        let playlist = match self.spotify_client.get_playlist(id).await? {
            Some(p) => p,
            None => {
                anyhow::bail!("Playlist with Spotify ID {} not found in Spotify", id);
            }
        };

        let created = self.playlist_repository.create(&playlist).await?;
        info!(
            "Created new playlist with ID {} from Spotify ID {}",
            created.id, id
        );
        Ok(created)
    }

    async fn get_playlist(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>> {
        self.playlist_repository.get(id).await
    }

    async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<Job> {
        let _playlist = match self.playlist_repository.get(id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!("Playlist with ID {} not found", id);
            }
        };

        todo!()
    }

    async fn get_playlist_pdfs(&self, _id: &PlaylistId) -> anyhow::Result<[Pdf; 2]> {
        todo!()
    }

    async fn refetch_playlist(&self, id: &PlaylistId) -> anyhow::Result<Job> {
        // Get the current playlist to preserve its ID and get Spotify ID
        let current_playlist = match self.playlist_repository.get(id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!("Playlist with ID {} not found", id);
            }
        };

        // Get the Spotify ID from the current playlist
        let spotify_id = match current_playlist.spotify_id {
            Some(spotify_id) => spotify_id,
            None => {
                anyhow::bail!("Playlist {} has no associated Spotify ID", id);
            }
        };

        // Fetch fresh data from Spotify
        let _fresh_playlist = match self.spotify_client.get_playlist(&spotify_id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!(
                    "Playlist with Spotify ID {} not found in Spotify",
                    spotify_id
                );
            }
        };

        todo!()
    }
}
