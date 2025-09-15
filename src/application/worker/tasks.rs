use crate::application::worker::IWorkerTask;
use crate::application::{IPdfGenerator, IPlaylistRepository, ISpotifyClient};
use crate::domain::PlaylistId;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GeneratePlaylistPdfsTask<PlaylistRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository,
    PdfGenerator: IPdfGenerator,
{
    pub playlist_id: PlaylistId,
    _marker: std::marker::PhantomData<(PlaylistRepository, PdfGenerator)>,
}

impl<PlaylistRepository, PdfGenerator> GeneratePlaylistPdfsTask<PlaylistRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository,
    PdfGenerator: IPdfGenerator,
{
    pub fn new(playlist_id: PlaylistId) -> Self {
        Self {
            playlist_id,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct GeneratePlaylistPdfsState<PlaylistRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository,
    PdfGenerator: IPdfGenerator,
{
    pub playlist_repository: Arc<PlaylistRepository>,
    pub pdf_generator: Arc<PdfGenerator>,
}

impl<PlaylistRepository, PdfGenerator> Clone
    for GeneratePlaylistPdfsState<PlaylistRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository,
    PdfGenerator: IPdfGenerator,
{
    fn clone(&self) -> Self {
        Self {
            playlist_repository: self.playlist_repository.clone(),
            pdf_generator: self.pdf_generator.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GeneratePlaylistPdfsResult {
    pub front: PathBuf,
    pub back: PathBuf,
}
impl<PlaylistRepository, PdfGenerator> IWorkerTask
    for GeneratePlaylistPdfsTask<PlaylistRepository, PdfGenerator>
where
    PlaylistRepository: IPlaylistRepository,
    PdfGenerator: IPdfGenerator,
{
    type State = GeneratePlaylistPdfsState<PlaylistRepository, PdfGenerator>;
    type Output = GeneratePlaylistPdfsResult;

    async fn run(&self, state: &Self::State) -> anyhow::Result<GeneratePlaylistPdfsResult> {
        let playlist = state
            .playlist_repository
            .get(&self.playlist_id)
            .await?
            .ok_or(anyhow!("playlist not found for id: {}", &self.playlist_id))?;

        let front_pdf_data_fut = state.pdf_generator.generate_front_cards(&playlist);
        let back_pdf_data_fut = state.pdf_generator.generate_back_cards(&playlist);
        let (front_pdf_data, back_pdf_data) =
            tokio::try_join!(front_pdf_data_fut, back_pdf_data_fut)?;

        // Create output directory if it doesn't exist
        let output_dir = std::path::PathBuf::from("generated_pdfs");
        tokio::fs::create_dir_all(&output_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let base_filename = format!("{}_{}", playlist.id, timestamp);

        let front_path = output_dir.join(format!("{}_front.pdf", base_filename));
        let back_path = output_dir.join(format!("{}_back.pdf", base_filename));

        tokio::fs::write(&front_path, front_pdf_data).await?;
        tokio::fs::write(&back_path, back_pdf_data).await?;

        Ok(GeneratePlaylistPdfsResult {
            front: front_path,
            back: back_path,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefetchPlaylistTask<PlaylistRepository, SpotifyClient>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
{
    pub playlist_id: PlaylistId,
    _marker: std::marker::PhantomData<(PlaylistRepository, SpotifyClient)>,
}

impl<PlaylistRepository, SpotifyClient> RefetchPlaylistTask<PlaylistRepository, SpotifyClient>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
{
    pub fn new(playlist_id: PlaylistId) -> Self {
        Self {
            playlist_id,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct RefetchPlaylistState<PlaylistRepository, SpotifyClient>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
{
    pub playlist_repository: Arc<PlaylistRepository>,
    pub spotify_client: Arc<SpotifyClient>,
}

impl<PlaylistRepository, SpotifyClient> Clone
    for RefetchPlaylistState<PlaylistRepository, SpotifyClient>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
{
    fn clone(&self) -> Self {
        Self {
            playlist_repository: self.playlist_repository.clone(),
            spotify_client: self.spotify_client.clone(),
        }
    }
}

impl<PlaylistRepository, SpotifyClient> IWorkerTask
    for RefetchPlaylistTask<PlaylistRepository, SpotifyClient>
where
    PlaylistRepository: IPlaylistRepository,
    SpotifyClient: ISpotifyClient,
{
    type State = RefetchPlaylistState<PlaylistRepository, SpotifyClient>;
    type Output = ();

    async fn run(&self, state: &Self::State) -> anyhow::Result<Self::Output> {
        let current_playlist = match state.playlist_repository.get(&self.playlist_id).await? {
            Some(playlist) => playlist,
            None => {
                anyhow::bail!("Playlist with ID {} not found", &self.playlist_id);
            }
        };

        // Get the Spotify ID from the current playlist
        let spotify_id = match current_playlist.spotify_id.clone() {
            Some(spotify_id) => spotify_id,
            None => {
                anyhow::bail!(
                    "Playlist {} has no associated Spotify ID",
                    &self.playlist_id
                );
            }
        };

        // Fetch fresh data from Spotify
        let fresh_playlist = match state
            .spotify_client
            .get_playlist_with_tracks(&spotify_id)
            .await?
        {
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
        state.playlist_repository.update(&updated_playlist).await?;

        Ok(())
    }
}
