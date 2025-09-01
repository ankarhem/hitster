use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::Write;

use hitster::{SpotifyService, SongCard};

#[derive(Parser)]
#[command(name = "hitster")]
#[command(about = "Generate CSV card data from Spotify playlists")]
struct Cli {
    #[arg(short, long)]
    playlist_url: String,
    
    #[arg(short, long)]
    title: String,
    
    #[arg(short, long, default_value = "hitster-cards.csv")]
    output: String,
}

fn create_csv(cards: Vec<SongCard>, title: &str, output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;
    
    writeln!(file, "Card Title,Song Name,Artist,Year,Spotify URL")?;
    
    for card in cards {
        writeln!(file, "{},{},{},{},{}", title, card.title, card.artist, card.year, card.spotify_url)?;
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    
    println!("Initializing Spotify service...");
    let spotify_service = SpotifyService::new().await?;
    
    println!("Fetching playlist tracks...");
    let cards = spotify_service.get_playlist_tracks(&cli.playlist_url).await?;
    
    println!("Generating CSV with {} cards...", cards.len());
    create_csv(cards, &cli.title, &cli.output)?;
    
    println!("CSV generated successfully: {}", cli.output);
    println!("You can use this CSV to generate QR codes and print cards.");
    
    Ok(())
}