use anyhow::Result;
use hitster::{SpotifyService, WebServer};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Reading configuration...");
    let settings = hitster::Settings::new()?;
    
    println!("Initializing Spotify service...");
    let spotify_service = SpotifyService::new(&settings).await?;
    
    println!("Starting web server...");
    let web_server = WebServer::new(spotify_service);
    web_server.run(3000).await?;
    
    Ok(())
}