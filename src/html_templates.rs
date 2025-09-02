//! HTML template utilities
//! 
//! This module contains HTML templates and styling for the generated cards.

/// HTML template for the welcome page
pub const WELCOME_TEMPLATE: &str = r#"<!DOCTYPE html>
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
                            placeholder="3vnwX8FuGWpGgQX4hBa8sE or https://open.spotify.com/playlist/3vnwX8FuGWpGgQX4hBa8sE"
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
</html>"#;

/// HTML template for error pages
pub fn error_template(error_message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
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
</html>"#,
        error_message
    )
}