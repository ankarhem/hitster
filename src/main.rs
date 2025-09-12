use anyhow::Result;
use hitster::SpotifyClient;
use hitster::application::PlaylistService;
use hitster::infrastructure::JobsRepository;
use hitster::infrastructure::playlist::PlaylistRepository;
use hitster::web::server::run;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("hitster=debug")
        .init();

    let settings = hitster::Settings::new()?;

    // infrastructure
    let spotify_client = Arc::new(SpotifyClient::new(&settings).await?);

    // Database setup with connection pooling
    let sqlite_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(settings.db_pool_max_connections)
        .acquire_timeout(std::time::Duration::from_secs(settings.db_pool_timeout_seconds))
        .connect_with(
            SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(&settings.database_path),
        )
        .await?;
    sqlx::migrate!("./migrations").run(&sqlite_pool).await?;

    let jobs_repository = JobsRepository::new(sqlite_pool.clone()).into();
    let playlist_repository = PlaylistRepository::new(sqlite_pool.clone()).await?.into();

    // application
    let playlist_service =
        PlaylistService::new(playlist_repository, spotify_client, jobs_repository).into();

    run(&settings.host, settings.port, playlist_service).await?;

    Ok(())
}
