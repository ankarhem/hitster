use anyhow::Result;
use sqlx::sqlite::SqliteConnectOptions;
use hitster::{SpotifyClient};
use hitster::application::{JobsService, PlaylistService};
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
    
    let sqlite_pool = sqlx::SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(&settings.database_url)
    ).await?;
    let jobs_repository = JobsRepository::new(sqlite_pool.clone());
    let playlist_repository = PlaylistRepository::new(&settings, sqlite_pool).await?;
    
    // application
    let jobs_service = JobsService::new(jobs_repository.clone());
    let playlist_service = PlaylistService::new(playlist_repository, spotify_client);
    
    run(3000, jobs_service, playlist_service).await?;
    
    Ok(())
}