use anyhow::Result;
use hitster::{SpotifyService, WebServer, Database};
use hitster::application::{JobService, PlaylistService};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("hitster=debug")
        .init();
    
    let settings = hitster::Settings::new()?;
    let spotify_service = Arc::new(SpotifyService::new(&settings).await?);
    
    // Initialize database
    let database = Arc::new(Database::new(&settings.database_url).await?);
    
    // Initialize job service
    let job_service = JobService::new(database.clone());
    
    // Initialize playlist service
    let playlist_service = PlaylistService::new(database.clone(), spotify_service.clone());
    
    let web_server = WebServer::new(job_service, playlist_service);
    web_server.run(3000).await?;
    
    Ok(())
}