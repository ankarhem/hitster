use anyhow::Result;
use clap::Parser;

use hitster::{SpotifyService, PlaylistId, HtmlGenerator};

#[derive(Parser)]
#[command(name = "hitster")]
#[command(about = "Generate HTML cards from Spotify playlists")]
struct Cli {
    #[arg(short, long)]
    playlist_url: String,
    
    #[arg(short, long)]
    title: String,
    
    #[arg(short, long, default_value = "hitster-cards.html")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("Reading configuration...");
    let settings = hitster::Settings::new()?;
    
    println!("Initializing Spotify service...");
    let spotify_service = SpotifyService::new(&settings).await?;
    
    println!("Fetching playlist tracks...");
    let playlist_id: PlaylistId = cli.playlist_url.parse()?;
    let cards = spotify_service.get_playlist_tracks_by_id(playlist_id).await?;
    
    println!("Generating HTML with {} cards...", cards.len());
    let html_generator = HtmlGenerator::new();
    html_generator.generate_html(cards, &cli.title, &cli.output)?;
    
    println!("HTML generated successfully: {}", cli.output);
    println!("Each card contains a QR code that links to the song on Spotify.");
    println!("Open the HTML file in your browser and print or save as PDF.");
    
    Ok(())
}