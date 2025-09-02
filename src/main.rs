use anyhow::Result;
use clap::{Parser, Subcommand};

use hitster::{SpotifyService, PlaylistId, HtmlGenerator, WebServer};

#[derive(Parser)]
#[command(name = "hitster")]
#[command(about = "Generate HTML cards from Spotify playlists or run web server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate HTML file from playlist
    Generate {
        #[arg(short, long)]
        playlist_url: String,
        
        #[arg(short, long)]
        title: String,
        
        #[arg(short, long, default_value = "hitster-cards.html")]
        output: String,
    },
    /// Run web server for interactive card generation
    Serve {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Generate { playlist_url, title, output } => {
            println!("Reading configuration...");
            let settings = hitster::Settings::new()?;
            
            println!("Initializing Spotify service...");
            let spotify_service = SpotifyService::new(&settings).await?;
            
            println!("Fetching playlist tracks...");
            let playlist_id: PlaylistId = playlist_url.parse()?;
            let cards = spotify_service.get_playlist_tracks_by_id(playlist_id).await?;
            
            println!("Generating HTML with {} cards...", cards.len());
            let html_generator = HtmlGenerator::new();
            html_generator.generate_html(cards, &title, &output)?;
            
            println!("HTML generated successfully: {}", output);
            println!("Each card contains a QR code that links to the song on Spotify.");
            println!("Open the HTML file in your browser and print or save as PDF.");
        }
        Commands::Serve { port } => {
            println!("Reading configuration...");
            let settings = hitster::Settings::new()?;
            
            println!("Initializing Spotify service...");
            let spotify_service = SpotifyService::new(&settings).await?;
            
            println!("Starting web server...");
            let web_server = WebServer::new(spotify_service);
            web_server.run(port).await?;
        }
    }
    
    Ok(())
}