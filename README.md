# Hitster Card Generator

A Rust application that takes a Spotify playlist URL and generates a CSV file with card data that can be used to create physical cards with QR codes.

## Features

- Fetches track data from Spotify playlists using the official [rspotify](https://github.com/ramsayleung/rspotify) crate
- Extracts song name, artist, release year, and Spotify URL
- Generates CSV output suitable for creating physical cards
- Each card contains:
  - Front: Title and QR code linking to the song
  - Back: Song name, artist, and release year

## Setup

1. Get Spotify API credentials:
   - Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
   - Create a new app
   - Note down your Client ID and Client Secret

2. Set up configuration:
   
   The application uses the `config` crate which supports multiple configuration sources in the following priority order:
   
   **Option A: Using .env file (recommended for development)**
   ```bash
   cp .env.example .env
   # Edit .env file with your actual credentials
   ```
   
   **Option B: Using TOML configuration file**
   ```bash
   cp config.example.toml config.toml
   # Edit config.toml file with your actual credentials
   ```
   
   **Option C: System environment variables**
   ```bash
   # Standard format
   export SPOTIFY_CLIENT_ID="your_client_id"
   export SPOTIFY_CLIENT_SECRET="your_client_secret"
   
   # Or with prefix
   export HITSTER_SPOTIFY_CLIENT_ID="your_client_id"
   export HITSTER_SPOTIFY_CLIENT_SECRET="your_client_secret"
   ```

3. Build the application:
   ```bash
   cargo build --release
   ```

## Usage

```bash
./target/release/hitster --playlist-url <spotify_playlist_url> --title <card_title> --output <output_file.csv>
```

Example:
```bash
./target/release/hitster \
  --playlist-url "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M" \
  --title "Today's Top Hits" \
  --output "top-hits.csv"
```

## Output

The application generates a CSV file with the following columns:
- Card Title: The title you provided
- Song Name: Name of the song
- Artist: Artist name(s)
- Year: Release year
- Spotify URL: Direct link to the song on Spotify

## Creating Physical Cards

1. Use the CSV to generate QR codes for each Spotify URL
2. Print cards with:
   - Front side: Card title and QR code
   - Back side: Song name, artist, and year
3. Cut and laminate the cards for durability

## License

MIT License