use crate::application::{IJobsRepository, IPlaylistRepository, ISpotifyClient};
use crate::domain::{Job, JobKind, JobStatus, Pdf, Playlist, PlaylistId, SpotifyId};
use std::sync::Arc;
use tracing::info;

#[trait_variant::make(IPlaylistService: Send)]
pub trait _IPlaylistService: Send + Sync {
    async fn create_from_spotify(&self, id: &SpotifyId) -> anyhow::Result<Option<Playlist>>;
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

        // Create a job for PDF generation
        let job = Job::new(
            JobKind::GeneratePdfs,
            serde_json::json!({"playlist_id": id.to_string()}),
        );

        // Save the job to the database
        let job = self.jobs_repository.create(job).await?;

        // In a real implementation, this would be handled by a background worker
        // For now, we'll generate the PDFs synchronously
        let (front_path, back_path) = crate::pdf_generator::generate_pdfs(&playlist).await?;

        // Update the job with the results
        let mut completed_job = job.clone();
        completed_job.status = JobStatus::Completed;
        completed_job.completed_at = Some(chrono::Utc::now());
        completed_job.payload = serde_json::json!({
            "playlist_id": id.to_string(),
            "front_path": front_path,
            "back_path": back_path
        });

        self.jobs_repository.update(completed_job).await?;

        info!("Generated PDFs for playlist {}: {}, {}", id, front_path, back_path);

        Ok(job)
    }

    async fn get_playlist_pdfs(&self, id: &PlaylistId) -> anyhow::Result<[Pdf; 2]> {
        // Look for completed PDF generation jobs for this playlist
        let jobs = self.jobs_repository.get_by_playlist_id(id).await?;
        
        let latest_job = jobs.into_iter()
            .filter(|job| job.status == JobStatus::Completed && job.kind == JobKind::GeneratePdfs)
            .max_by_key(|job| job.completed_at);

        match latest_job {
            Some(job) => {
                let front_path = job.payload["front_path"].as_str().unwrap_or("");
                let back_path = job.payload["back_path"].as_str().unwrap_or("");
                
                let front_pdf = tokio::fs::read(front_path).await?;
                let back_pdf = tokio::fs::read(back_path).await?;
                
                Ok([
                    Pdf::new(front_pdf),
                    Pdf::new(back_pdf),
                ])
            }
            None => {
                anyhow::bail!("No PDFs found for playlist {}", id);
            }
        }
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
        let spotify_id = match current_playlist.spotify_id.clone() {
            Some(spotify_id) => spotify_id,
            None => {
                anyhow::bail!("Playlist {} has no associated Spotify ID", id);
            }
        };

        // Create a job for playlist refetching
        let job = Job::new(
            JobKind::RefetchPlaylist,
            serde_json::json!({"playlist_id": id.to_string()}),
        );

        // Save the job to the database
        let job = self.jobs_repository.create(job).await?;

        // Fetch fresh data from Spotify
        let fresh_playlist = match self.spotify_client.get_playlist(&spotify_id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!(
                    "Playlist with Spotify ID {} not found in Spotify",
                    spotify_id
                );
            }
        };

        // Create an updated playlist with the fresh data but preserve the original ID
        let mut updated_playlist = fresh_playlist;
        updated_playlist.id = current_playlist.id;
        updated_playlist.spotify_id = current_playlist.spotify_id;
        updated_playlist.created_at = current_playlist.created_at;
        updated_playlist.updated_at = Some(chrono::Utc::now());

        // Update the playlist in the repository
        self.playlist_repository.update(&updated_playlist).await?;

        // Update the job with completion status
        let mut completed_job = job.clone();
        completed_job.status = JobStatus::Completed;
        completed_job.completed_at = Some(chrono::Utc::now());
        completed_job.payload = serde_json::json!({
            "playlist_id": id.to_string(),
            "tracks_count": updated_playlist.tracks.len(),
            "message": "Playlist refetched successfully"
        });

        self.jobs_repository.update(completed_job).await?;

        info!(
            "Refetched playlist {} (Spotify ID: {}) with {} tracks",
            id, spotify_id, updated_playlist.tracks.len()
        );

        Ok(job)
    }
}
