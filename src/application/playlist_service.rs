use crate::application::worker::{GeneratePlaylistPdfsResult, IWorker};
use crate::application::{
    IJobsRepository, IPdfGenerator, IPlaylistRepository, ISpotifyClient, worker,
};
use crate::domain::{Job, JobId, JobStatus, Pdf, Playlist, PlaylistId, SpotifyId};
use std::future::Future;
use std::sync::Arc;
use tracing::info;

pub trait IPlaylistService: Clone + Send + Sync + 'static {
    fn create_from_spotify(
        &self,
        id: &SpotifyId,
    ) -> impl Future<Output = anyhow::Result<Option<Playlist>>> + Send;
    fn create_partial_playlist_from_spotify(
        &self,
        id: &SpotifyId,
    ) -> impl Future<Output = anyhow::Result<(Option<Playlist>, Option<Job>)>> + Send;
    fn get_playlist(
        &self,
        id: &PlaylistId,
    ) -> impl Future<Output = anyhow::Result<Option<Playlist>>> + Send;
    fn generate_playlist_pdfs(
        &self,
        id: &PlaylistId,
    ) -> impl Future<Output = anyhow::Result<Job>> + Send;
    fn get_playlist_pdfs(
        &self,
        id: &PlaylistId,
    ) -> impl Future<Output = anyhow::Result<[Pdf; 2]>> + Send;
    fn refetch_playlist(&self, id: &PlaylistId)
    -> impl Future<Output = anyhow::Result<Job>> + Send;
    fn get_latest_job(
        &self,
        playlist_id: &PlaylistId,
    ) -> impl Future<Output = anyhow::Result<Option<Job>>> + Send;
    fn get_job_by_id(
        &self,
        job_id: &JobId,
    ) -> impl Future<Output = anyhow::Result<Option<Job>>> + Send;
}

#[derive(Clone)]
pub struct PlaylistService<
    SC: ISpotifyClient,
    PR: IPlaylistRepository,
    JR: IJobsRepository,
    PG: IPdfGenerator,
> {
    spotify_client: Arc<SC>,
    playlist_repository: Arc<PR>,
    jobs_repository: Arc<JR>,
    pdf_worker: Arc<worker::Worker<JR, worker::GeneratePlaylistPdfsTask<PR, PG>>>,
    refetch_worker: Arc<worker::Worker<JR, worker::RefetchPlaylistTask<PR, SC>>>,
}

impl<SC: ISpotifyClient, PR: IPlaylistRepository, JR: IJobsRepository, PG: IPdfGenerator>
    PlaylistService<SC, PR, JR, PG>
{
    pub fn new(
        playlist_repository: Arc<PR>,
        spotify_client: Arc<SC>,
        jobs_repository: Arc<JR>,
        pdf_worker: Arc<worker::Worker<JR, worker::GeneratePlaylistPdfsTask<PR, PG>>>,
        refetch_worker: Arc<worker::Worker<JR, worker::RefetchPlaylistTask<PR, SC>>>,
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

impl<SC: ISpotifyClient, PR: IPlaylistRepository, JR: IJobsRepository, PG: IPdfGenerator>
    IPlaylistService for PlaylistService<SC, PR, JR, PG>
{
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>> {
        if let Some(existing) = self.playlist_repository.get_by_spotify_id(id).await? {
            info!(
                "Playlist with Spotify ID {} already exists with ID {}",
                id, existing.id
            );
            return Ok(Some(existing));
        }

        let playlist = match self.spotify_client.get_playlist_with_tracks(id).await? {
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

    async fn create_partial_playlist_from_spotify(
        &self,
        id: &SpotifyId,
    ) -> anyhow::Result<(Option<Playlist>, Option<Job>)> {
        if let Some(existing) = self.playlist_repository.get_by_spotify_id(id).await? {
            info!(
                "Playlist with Spotify ID {} already exists with ID {}",
                id, existing.id
            );
            return Ok((Some(existing), None));
        }

        let playlist = match self.spotify_client.get_playlist(id).await? {
            Some(p) => p,
            None => {
                info!("Playlist with Spotify ID {} not found", id);
                return Ok((None, None));
            }
        };
        let created = self.playlist_repository.create(&playlist).await?;

        let job = IPlaylistService::refetch_playlist(self, &playlist.id).await?;

        Ok((Some(created), Some(job)))
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

        let (_, pdfs) = jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .filter(|j| j.result.is_some())
            .filter_map(|j| {
                let result: serde_json::Value = j.result.clone()?;
                let pdfs: GeneratePlaylistPdfsResult = serde_json::from_value(result).ok()?;
                Some((j, pdfs))
            })
            .max_by_key(|(j, _)| j.completed_at)
            .ok_or(anyhow::anyhow!("No generation job found"))?;

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

    async fn get_latest_job(&self, playlist_id: &PlaylistId) -> anyhow::Result<Option<Job>> {
        let jobs = self.jobs_repository.get_by_playlist_id(playlist_id).await?;
        Ok(jobs.into_iter().max_by_key(|j| j.created_at))
    }

    async fn get_job_by_id(&self, job_id: &JobId) -> anyhow::Result<Option<Job>> {
        let job = self.jobs_repository.get(job_id).await?;

        Ok(job)
    }
}
