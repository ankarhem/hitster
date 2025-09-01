use anyhow::Result;
use clap::Parser;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

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

#[derive(Debug, Serialize, Deserialize)]
struct SpotifyTrack {
    id: String,
    name: String,
    artists: Vec<SpotifyArtist>,
    album: SpotifyAlbum,
    external_urls: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpotifyArtist {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpotifyAlbum {
    release_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpotifyPlaylist {
    tracks: PlaylistTracks,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaylistTracks {
    items: Vec<PlaylistItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaylistItem {
    track: SpotifyTrack,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug)]
struct SongCard {
    title: String,
    artist: String,
    year: String,
    spotify_url: String,
}

async fn get_spotify_token() -> Result<String> {
    let client_id = env::var("SPOTIFY_CLIENT_ID")?;
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET")?;
    
    let auth = format!("{}:{}", client_id, client_secret);
    let auth_encoded = base64::encode(auth);
    
    let client = reqwest::Client::new();
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .header(header::AUTHORIZATION, format!("Basic {}", auth_encoded))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?;
    
    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
}

async fn get_playlist_tracks(token: &str, playlist_url: &str) -> Result<Vec<SongCard>> {
    let playlist_id = extract_playlist_id(playlist_url)?;
    
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await?;
    
    let playlist: SpotifyPlaylist = response.json().await?;
    
    let mut cards = Vec::new();
    for item in playlist.tracks.items {
        let track = item.track;
        let artist_names: Vec<String> = track.artists.iter().map(|a| a.name.clone()).collect();
        let year = track.album.release_date.split('-').next().unwrap_or("Unknown").to_string();
        
        cards.push(SongCard {
            title: track.name,
            artist: artist_names.join(", "),
            year,
            spotify_url: track.external_urls.get("spotify").cloned().unwrap_or_default(),
        });
    }
    
    Ok(cards)
}

fn extract_playlist_id(url: &str) -> Result<String> {
    let parts: Vec<&str> = url.split('/').collect();
    let last_part = parts.last().ok_or_else(|| anyhow::anyhow!("Invalid playlist URL"))?;
    let id_parts: Vec<&str> = last_part.split('?').collect();
    let id = id_parts.first().ok_or_else(|| anyhow::anyhow!("Invalid playlist URL"))?;
    Ok(id.to_string())
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
    
    println!("Getting Spotify access token...");
    let token = get_spotify_token().await?;
    
    println!("Fetching playlist tracks...");
    let cards = get_playlist_tracks(&token, &cli.playlist_url).await?;
    
    println!("Generating CSV with {} cards...", cards.len());
    create_csv(cards, &cli.title, &cli.output)?;
    
    println!("CSV generated successfully: {}", cli.output);
    println!("You can use this CSV to generate QR codes and print cards.");
    
    Ok(())
}