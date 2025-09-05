use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use crate::application::{JobService, PlaylistService};
use crate::web::controllers::{ViewController, PlaylistController, JobsController};
use tracing::info;

#[derive(Clone)]
pub struct WebServer {
    pub job_service: JobService,
    pub playlist_service: PlaylistService,
}

impl WebServer {
    pub fn new(job_service: JobService, playlist_service: PlaylistService) -> Self {
        Self { job_service, playlist_service }
    }

    pub async fn run(self, port: u16) -> anyhow::Result<()> {
        let app = Router::new()
            // View endpoints
            .route("/", get(ViewController::index))
            .route("/playlist/:playlist_id", get(ViewController::playlist))
            
            // Playlist API endpoints
            .route("/api/playlist", post(PlaylistController::create_playlist))
            .route("/api/playlist/:playlist_id/refetch-playlist", post(PlaylistController::refetch_playlist))
            .route("/api/playlist/:playlist_id/pdfs", post(PlaylistController::generate_pdfs))
            .route("/api/playlist/:playlist_id/pdfs/:side", get(PlaylistController::get_pdf))
            
            // Jobs API endpoints
            .route("/api/jobs/:job_id", get(JobsController::get_job))
            
            .with_state(self);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!("Listening on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }
}