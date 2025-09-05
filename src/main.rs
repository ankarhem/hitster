use anyhow::Result;
use hitster::{SpotifyClient};
use hitster::application::{JobService, PlaylistService};
use hitster::infrastructure::JobsRepository;
use hitster::infrastructure::playlist::PlaylistRepository;
use hitster::web::server::run;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("hitster=debug")
        .init();
    
    let settings = hitster::Settings::new()?;

    // infrastructure
    let spotify_client = SpotifyClient::new(&settings).await?;
    let jobs_repository = JobsRepository::new();
    let playlist_repository = PlaylistRepository::new(&settings).await?;
    
    // application
    let jobs_service = JobService::new(jobs_repository.clone());
    let playlist_service = PlaylistService::new(playlist_repository, spotify_client);
    
    let web_server = run(3000, jobs_service, playlist_service).await?;
    
    Ok(())
}