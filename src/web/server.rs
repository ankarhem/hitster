use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use crate::application::job_service::IJobService;
use crate::application::playlist_service::IPlaylistService;
use crate::web::controllers;

#[derive(Debug, Default)]
pub struct Services<JobsService, PlaylistService>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    pub job_service: Arc<JobsService>,
    pub playlist_service: Arc<PlaylistService>,
}

impl<JobsService, PlaylistService> Clone for Services<JobsService, PlaylistService>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    fn clone(&self) -> Self {
        Self {
            job_service: self.job_service.clone(),
            playlist_service: self.playlist_service.clone(),
        }
    }
}

pub async fn run<JobsService, PlaylistService>(
    port: u16,
    jobs_service: JobsService,
    playlist_service: PlaylistService,
) -> anyhow::Result<()>
where
    JobsService: IJobService + 'static,
    PlaylistService: IPlaylistService + 'static,
{
    
    let services = Services {
        job_service: Arc::new(jobs_service),
        playlist_service: Arc::new(playlist_service),
    };
    
    let app = Router::new()
        // View endpoints
        .route("/", get(controllers::view::index))
        .route("/playlist/:playlist_id", get(controllers::view::view_playlist))
        
        // Playlist API endpoints
        .route("/api/playlist", post(controllers::playlist::create_playlist))
        .route("/api/playlist/:playlist_id/refetch-playlist", post(controllers::playlist::refetch_playlist))
        .route("/api/playlist/:playlist_id/pdfs", post(controllers::playlist::generate_pdfs))
        .route("/api/playlist/:playlist_id/pdfs/:side", get(controllers::playlist::get_pdf))
        
        // Jobs API endpoints
        .route("/api/jobs/:job_id", get(controllers::jobs::get_job))
        
        .with_state(services);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}