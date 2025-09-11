use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use crate::application::playlist_service::IPlaylistService;
use crate::web::controllers;

#[derive(Debug, Default)]
pub struct Services<PlaylistService>
where
    PlaylistService: IPlaylistService,
{
    pub playlist_service: Arc<PlaylistService>,
}

impl<PlaylistService> Clone for Services<PlaylistService>
where
    PlaylistService: IPlaylistService,
{
    fn clone(&self) -> Self {
        Self {
            playlist_service: self.playlist_service.clone(),
        }
    }
}

pub async fn run<PlaylistService>(
    port: u16,
    playlist_service: Arc<PlaylistService>,
) -> anyhow::Result<()>
where
    PlaylistService: IPlaylistService + 'static,
{
    
    let services = Services {
        playlist_service,
    };
    
    let app = Router::new()
        // View endpoints
        .route("/", get(controllers::view::index))
        .route("/playlist/:playlist_id", get(controllers::view::view_playlist))
        
        // Playlist API endpoints
        .route("/api/playlist", post(controllers::playlist::create_playlist))
        .route("/api/playlist/:playlist_id/refetch-playlist", post(controllers::playlist::refetch_playlist))
        .route("/api/playlist/:playlist_id/generate-pdfs", post(controllers::playlist::generate_pdfs))
        .route("/api/playlist/:playlist_id/pdfs", get(controllers::playlist::get_pdfs))
        
        .with_state(services);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}