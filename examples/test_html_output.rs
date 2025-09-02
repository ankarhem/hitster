use hitster::html_generator::HtmlGenerator;
use hitster::SongCard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generator = HtmlGenerator::new();
    
    // Create test cards with real Spotify URLs
    let cards = vec![
        SongCard {
            title: "Bohemian Rhapsody".to_string(),
            artist: "Queen".to_string(),
            year: "1975".to_string(),
            spotify_url: "https://open.spotify.com/track/4u7EnebtmKWzUH433cf5Qv".to_string(),
        },
        SongCard {
            title: "Stairway to Heaven".to_string(),
            artist: "Led Zeppelin".to_string(),
            year: "1971".to_string(),
            spotify_url: "https://open.spotify.com/track/5UWwZkm2Ix8sllRQeIKn4F".to_string(),
        },
        SongCard {
            title: "Hotel California".to_string(),
            artist: "Eagles".to_string(),
            year: "1976".to_string(),
            spotify_url: "https://open.spotify.com/track/40riOy7x9W7GXjyGp4pjAv".to_string(),
        },
        SongCard {
            title: "Sweet Child O' Mine".to_string(),
            artist: "Guns N' Roses".to_string(),
            year: "1987".to_string(),
            spotify_url: "https://open.spotify.com/track/3a1lNhkzLczXvljwvORxL8".to_string(),
        },
    ];
    
    println!("Generating HTML with QR codes...");
    let result = generator.generate_html(cards.clone(), "Classic Rock Test", "test_qr_cards.html");
    
    match result {
        Ok(()) => {
            println!("✅ HTML generated successfully!");
            println!("📄 File: test_qr_cards.html");
            println!("🎵 Generated {} cards", cards.len());
            
            // Check file size
            if let Ok(metadata) = std::fs::metadata("test_qr_cards.html") {
                println!("📊 File size: {} bytes", metadata.len());
            }
            
            println!("🔍 Open test_qr_cards.html in your browser to verify:");
            println!("   - Cards are properly laid out");
            println!("   - QR codes are visible and scannable");
            println!("   - Print layout works correctly");
        }
        Err(e) => {
            println!("❌ Failed to generate HTML: {}", e);
        }
    }
    
    Ok(())
}