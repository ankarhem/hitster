use anyhow::Result;
use hitster::application::worker::{GeneratePlaylistPdfsTask, RefetchPlaylistTask, Worker};
use hitster::application::{PlaylistService, worker};
use hitster::infrastructure::JobsRepository;
use hitster::infrastructure::playlist::PlaylistRepository;
use hitster::web::server::run;
use hitster::{PdfGenerator, SpotifyClient};
use sqlx::sqlite::SqliteConnectOptions;
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
        .acquire_timeout(std::time::Duration::from_secs(
            settings.db_pool_timeout_seconds,
        ))
        .connect_with(
            SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(&settings.database_path),
        )
        .await?;
    sqlx::migrate!("./migrations").run(&sqlite_pool).await?;

    let jobs_repository = Arc::new(JobsRepository::new(sqlite_pool.clone()));
    let playlist_repository = Arc::new(PlaylistRepository::new(sqlite_pool.clone()).await?);
    let pdf_generator = Arc::new(PdfGenerator::new());

    let pdf_worker_state = Arc::new(worker::GeneratePlaylistPdfsState {
        playlist_repository: playlist_repository.clone(),
        pdf_generator: pdf_generator.clone(),
    });
    let pdf_worker: Worker<
        JobsRepository,
        GeneratePlaylistPdfsTask<PlaylistRepository, PdfGenerator>,
    > = Worker::new(jobs_repository.clone(), pdf_worker_state);
    let refetch_worker_state = Arc::new(worker::RefetchPlaylistState {
        playlist_repository: playlist_repository.clone(),
        spotify_client: spotify_client.clone(),
    });
    let refetch_worker: Worker<
        JobsRepository,
        RefetchPlaylistTask<PlaylistRepository, SpotifyClient>,
    > = Worker::new(jobs_repository.clone(), refetch_worker_state);

    // application
    let playlist_service = PlaylistService::new(
        playlist_repository,
        spotify_client,
        jobs_repository,
        Arc::new(pdf_worker),
        Arc::new(refetch_worker),
    )
    .into();

    run(&settings.host, settings.port, playlist_service).await?;

    Ok(())
}
