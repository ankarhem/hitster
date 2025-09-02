use anyhow::Result;
use hitster::{SpotifyService, WebServer};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("hitster=debug,tower_http=debug")
        .init();
    
    tracing::info!("Starting Hitster application");
    
    tracing::debug!("Reading configuration...");
    let settings = hitster::Settings::new()?;
    tracing::info!("Configuration loaded successfully");
    
    tracing::debug!("Initializing Spotify service...");
    let spotify_service = SpotifyService::new(&settings).await?;
    tracing::info!("Spotify service initialized");
    
    tracing::debug!("Starting web server...");
    let web_server = WebServer::new(spotify_service);
    web_server.run(3000).await?;
    
    Ok(())
}