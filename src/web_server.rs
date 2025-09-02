use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, Router},
};
use std::net::SocketAddr;
use crate::{SpotifyService, HtmlGenerator, PlaylistId};

#[derive(Clone)]
pub struct WebServer {
    spotify_service: SpotifyService,
    html_generator: HtmlGenerator,
}

impl WebServer {
    pub fn new(spotify_service: SpotifyService) -> Self {
        Self {
            spotify_service,
            html_generator: HtmlGenerator::new(),
        }
    }

    pub async fn run(&self, port: u16) -> Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/playlist/:playlist_id", get(playlist_cards))
            .with_state(self.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        println!("🚀 Web server running at http://localhost:{}", port);
        println!("📋 Endpoints:");
        println!("   GET /                           - Welcome page");
        println!("   GET /playlist/<id>             - HTML cards for playlist");
        println!("   Example: http://localhost:{}/playlist/37i9dQZF1DXcBWIGoYBM5M", port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }

    pub async fn get_playlist_cards(&self, playlist_id: &str, title: Option<String>) -> Result<String> {
        let playlist_id: PlaylistId = playlist_id.parse()?;
        let title = title.unwrap_or_else(|| format!("Playlist: {}", playlist_id));
        let cards = self.spotify_service.get_playlist_tracks_by_id(playlist_id.clone()).await?;
        let html = self.html_generator.build_html_content(cards, &title);
        
        Ok(html)
    }
}

// Handler functions
async fn root() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hitster Cards</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <div class="max-w-2xl mx-auto">
            <h1 class="text-4xl font-bold text-center mb-8 text-gray-800">🎵 Hitster Cards</h1>
            
            <div class="bg-white rounded-lg shadow-lg p-6 mb-6">
                <h2 class="text-2xl font-semibold mb-4">Generate Cards from Spotify Playlist</h2>
                <p class="text-gray-600 mb-4">
                    Enter a Spotify playlist ID or URL to generate printable cards with QR codes.
                </p>
                
                <div class="space-y-4">
                    <div>
                        <label for="playlistInput" class="block text-sm font-medium text-gray-700 mb-2">
                            Spotify Playlist ID or URL:
                        </label>
                        <input 
                            type="text" 
                            id="playlistInput" 
                            placeholder="37i9dQZF1DXcBWIGoYBM5M or https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                    </div>
                    
                    <div>
                        <label for="titleInput" class="block text-sm font-medium text-gray-700 mb-2">
                            Custom Title (optional):
                        </label>
                        <input 
                            type="text" 
                            id="titleInput" 
                            placeholder="My Awesome Playlist"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                    </div>
                    
                    <button 
                        onclick="generateCards()" 
                        class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 transition-colors"
                    >
                        Generate Cards
                    </button>
                </div>
            </div>
            
            <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <h3 class="font-semibold text-blue-800 mb-2">How to use:</h3>
                <ol class="list-decimal list-inside text-blue-700 space-y-1">
                    <li>Find a Spotify playlist URL (e.g., from Spotify app)</li>
                    <li>Enter the playlist ID or full URL above</li>
                    <li>Click "Generate Cards" to create printable cards</li>
                    <li>Print the page or save as PDF from your browser</li>
                    <li>Each card contains a QR code that links to the song on Spotify</li>
                </ol>
            </div>
        </div>
    </div>

    <script>
        async function generateCards() {
            const playlistInput = document.getElementById('playlistInput').value;
            const titleInput = document.getElementById('titleInput').value;
            
            if (!playlistInput) {
                alert('Please enter a playlist ID or URL');
                return;
            }
            
            // Extract playlist ID from URL if necessary
            let playlistId = playlistInput;
            if (playlistInput.includes('spotify.com/playlist/')) {
                const match = playlistInput.match(/playlist\/([a-zA-Z0-9]+)/);
                if (match) {
                    playlistId = match[1];
                }
            }
            
            // Build URL with optional title parameter
            let url = `/playlist/${playlistId}`;
            if (titleInput) {
                url += `?title=${encodeURIComponent(titleInput)}`;
            }
            
            window.location.href = url;
        }
        
        // Allow Enter key to generate cards
        document.getElementById('playlistInput').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                generateCards();
            }
        });
        
        document.getElementById('titleInput').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                generateCards();
            }
        });
    </script>
</body>
</html>
    "#)
}

async fn playlist_cards(
    Path(playlist_id): Path<String>,
    State(server): State<WebServer>,
) -> impl IntoResponse {
    let title = None;
    
    match server.get_playlist_cards(&playlist_id, title).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            let error_html = format!(
                r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error - Hitster Cards</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <div class="max-w-2xl mx-auto">
            <div class="bg-red-50 border border-red-200 rounded-lg p-6">
                <h1 class="text-2xl font-bold text-red-800 mb-4">❌ Error</h1>
                <p class="text-red-700 mb-4">Failed to generate cards: {}</p>
                <a href="/" class="inline-block bg-red-600 text-white py-2 px-4 rounded-md hover:bg-red-700 transition-colors">
                    ← Back to Home
                </a>
            </div>
        </div>
    </div>
</body>
</html>
                "#,
                e
            );
            Html(error_html).into_response()
        }
    }
}

