use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use crate::application::HitsterService;
use tracing::{info, error, instrument};

#[derive(Clone)]
pub struct WebServer {
    hitster_service: HitsterService,
}

impl WebServer {
    #[instrument(skip(hitster_service))]
    pub fn new(hitster_service: HitsterService) -> Self {
        Self { hitster_service }
    }

    #[instrument(skip(self), fields(port))]
    pub async fn run(&self, port: u16) -> Result<()> {
        let app = Router::new()
            .route("/playlist/:playlist_id", get(playlist_cards))
            .with_state(self.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        info!("🚀 Web server running at http://localhost:{}", port);
        info!("📋 Endpoints:");
        info!("   GET /playlist/<id>             - HTML cards for playlist");
        info!("   Example: http://localhost:{}/playlist/3vnwX8FuGWpGgQX4hBa8sE", port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[instrument(skip(server), fields(playlist_id))]
async fn playlist_cards(
    Path(playlist_id): Path<String>,
    State(server): State<WebServer>,
) -> impl IntoResponse {
    match server.hitster_service.generate_playlist_cards(&playlist_id, None).await {
        Ok(html) => {
            info!("Served playlist: {}", playlist_id);
            Html(html).into_response()
        },
        Err(e) => {
            error!("Failed to serve playlist {}: {}", playlist_id, e);
            let error_html = "<html>
                <head><title>Error</title></head>
                <body>
                    <h1>Error generating playlist cards</h1>
                </body>
              </html>";
            Html(error_html).into_response()
        }
    }
}