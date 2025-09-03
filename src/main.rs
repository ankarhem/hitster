use anyhow::Result;
use hitster::{SpotifyService, HitsterService, WebServer};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("hitster=debug,tower_http=debug")
        .init();
    
    let settings = hitster::Settings::new()?;
    let spotify_service = SpotifyService::new(&settings).await?;
    
    let hitster_service = HitsterService::new(spotify_service)?;
    
    let web_server = WebServer::new(hitster_service);
    web_server.run(3000).await?;
    
    Ok(())
}