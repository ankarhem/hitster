use crate::application::{IJobsRepository, IPlaylistRepository, IPdfGenerator, ISpotifyClient, worker};
use crate::domain::{Job, JobStatus, Pdf, Playlist, PlaylistId, SpotifyId};
use std::sync::Arc;
use tracing::info;
use crate::application::worker::{GeneratePlaylistPdfsResult, IWorker};

#[trait_variant::make(IPlaylistService: Send)]
pub trait _IPlaylistService: Send + Sync {
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>>;
    async fn get_playlist(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>>;
    async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<Job>;
    async fn get_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<[Pdf; 2]>;
    async fn refetch_playlist(&self, id: &PlaylistId) -> anyhow::Result<Job>;
}

#[derive(Clone)]
pub struct PlaylistService<SpotifyClient, PlaylistRepository, JobsRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
    JobsRepository: IJobsRepository,
    PdfGenerator: IPdfGenerator,
{
    spotify_client: Arc<SpotifyClient>,
    playlist_repository: Arc<PlaylistRepository>,
    jobs_repository: Arc<JobsRepository>,
    pdf_worker: Arc<worker::Worker<JobsRepository, worker::GeneratePlaylistPdfsTask<PlaylistRepository, PdfGenerator>>>,
    refetch_worker: Arc<worker::Worker<JobsRepository, worker::RefetchPlaylistTask<PlaylistRepository, SpotifyClient>>>,
}

impl<SpotifyClient, PlaylistRepository, JobsRepository, PdfGenerator>
    PlaylistService<SpotifyClient, PlaylistRepository, JobsRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository + 'static,
    SpotifyClient: ISpotifyClient + 'static,
    JobsRepository: IJobsRepository + 'static,
    PdfGenerator: IPdfGenerator + 'static,
{
    pub fn new(
        playlist_repository: Arc<PlaylistRepository>,
        spotify_client: Arc<SpotifyClient>,
        jobs_repository: Arc<JobsRepository>,
        pdf_worker: Arc<worker::Worker<JobsRepository, worker::GeneratePlaylistPdfsTask<PlaylistRepository, PdfGenerator>>>,
        refetch_worker: Arc<worker::Worker<JobsRepository, worker::RefetchPlaylistTask<PlaylistRepository, SpotifyClient>>>,
    ) -> Self {
        Self {
            spotify_client,
            playlist_repository,
            jobs_repository,
            pdf_worker,
            refetch_worker,
        }
    }
}

impl<SpotifyClient, PlaylistRepository, JobsRepository, PdfGenerator> IPlaylistService
    for PlaylistService<SpotifyClient, PlaylistRepository, JobsRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository + 'static,
    SpotifyClient: ISpotifyClient + 'static,
    JobsRepository: IJobsRepository + 'static,
    PdfGenerator: IPdfGenerator + 'static,
{
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>> {
        if let Some(existing) = self.playlist_repository.get_by_spotify_id(id).await? {
            info!(
                "Playlist with Spotify ID {} already exists with ID {}",
                id, existing.id
            );
            return Ok(Some(existing));
        }

        let playlist = match self.spotify_client.get_playlist(id).await? {
            Some(p) => p,
            None => {
                info!("Playlist with Spotify ID {} not found", id);
                return Ok(None);
            }
        };

        let created = self.playlist_repository.create(&playlist).await?;
        info!(
            "Created new playlist with ID {} from Spotify ID {}",
            created.id, id
        );
        Ok(Some(created))
    }

    async fn get_playlist(&self, id: &PlaylistId) -> anyhow::Result<Option<Playlist>> {
        self.playlist_repository.get(id).await
    }

    async fn generate_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<Job> {
        let playlist = match self.playlist_repository.get(id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!("Playlist with ID {} not found", id);
            }
        };

        let task = worker::GeneratePlaylistPdfsTask::new(playlist.id);

        let job = self.pdf_worker.enqueue(task).await?;

        Ok(job)
    }

    async fn get_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<[Pdf; 2]> {
        // Look for completed PDF generation jobs for this playlist
        let jobs = self.jobs_repository.get_by_playlist_id(id).await?;

        let (_, pdfs) = jobs.iter()
            .filter(|j| j.status == JobStatus::Completed)
            .filter(|j| j.result.is_some())
            .filter_map(|j| {
                let result: serde_json::Value = j.result.clone()?;
                let pdfs: GeneratePlaylistPdfsResult  = serde_json::from_value(result).ok()?;
                Some((j, pdfs))
            })
            .max_by_key(|(j, _)| j.completed_at).ok_or(anyhow::anyhow!("No generation job found"))?;

        let front: Pdf = tokio::fs::read(pdfs.front).await?.into();
        let back: Pdf = tokio::fs::read(pdfs.back).await?.into();

        Ok([front, back])
    }

    async fn refetch_playlist(&self, id: &PlaylistId) -> anyhow::Result<Job> {
        let playlist = match self.playlist_repository.get(id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!("Playlist with ID {} not found", id);
            }
        };

        let task = worker::RefetchPlaylistTask::new(playlist.id);
        let job = self.refetch_worker.enqueue(task).await?;

        Ok(job)
    }
}
