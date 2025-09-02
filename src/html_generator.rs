//! HTML generation for Spotify playlist cards
//! 
//! This module handles the generation of HTML documents containing
//! printable cards with QR codes for Spotify songs.

use anyhow::Result;
use crate::SongCard;
use crate::qr_generator;

/// Configuration for HTML generation
#[derive(Clone)]
pub struct HtmlGenerator {
    /// Card width in millimeters
    card_width_mm: f64,
    /// Card height in millimeters
    card_height_mm: f64,
    /// Margin between cards in millimeters
    margin_mm: f64,
}

impl HtmlGenerator {
    /// Create a new HTML generator with default settings
    /// 
    /// Uses standard business card dimensions (90mm x 55mm)
    pub fn new() -> Self {
        Self {
            card_width_mm: 90.0,  // Standard business card width in mm
            card_height_mm: 55.0, // Standard business card height in mm
            margin_mm: 5.0,
        }
    }

    /// Generate HTML file from song cards
    /// 
    /// Creates a complete HTML document with printable cards and saves it to a file.
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Vector of song cards to generate HTML for
    /// * `title` - Title for the HTML document
    /// * `output_path` - File path to save the HTML
    /// 
    /// # Errors
    /// 
    /// Returns an error if file writing fails
    pub fn generate_html(&self, cards: Vec<SongCard>, title: &str, output_path: &str) -> Result<()> {
        let html_content = self.build_html_content(cards, title);
        std::fs::write(output_path, html_content)?;
        Ok(())
    }

    /// Build HTML content from song cards
    /// 
    /// Creates a complete HTML document string with printable cards.
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Vector of song cards to generate HTML for
    /// * `title` - Title for the HTML document
    /// 
    /// # Returns
    /// 
    /// A complete HTML document string
    pub fn build_html_content(&self, cards: Vec<SongCard>, title: &str) -> String {
        let cards_html = self.generate_cards_html(&cards);
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hitster Cards - {title}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        @page {{
            size: A4;
            margin: 10mm;
        }}
        
        .card {{
            width: {card_width_mm}mm;
            height: {card_height_mm}mm;
            border: 1px solid #000;
            page-break-inside: avoid;
        }}
        
        .qr-code {{
            width: 25mm;
            height: 25mm;
        }}
        
        @media print {{
            body {{
                margin: 0;
                padding: 0;
            }}
            
            .card-grid {{
                grid-template-columns: repeat(2, 1fr);
                gap: {margin_mm}mm;
            }}
        }}
    </style>
</head>
<body class="bg-gray-100 p-4">
    <div class="max-w-7xl mx-auto">
        <h1 class="text-3xl font-bold text-center mb-8 text-gray-800">Hitster Cards - {title}</h1>
        <div class="card-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 justify-items-center">
            {cards_html}
        </div>
    </div>
</body>
</html>"#,
            title = title,
            card_width_mm = self.card_width_mm,
            card_height_mm = self.card_height_mm,
            margin_mm = self.margin_mm,
            cards_html = cards_html
        )
    }

    /// Generate HTML for all cards
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Slice of song cards to generate HTML for
    /// 
    /// # Returns
    /// 
    /// HTML string for all cards
    fn generate_cards_html(&self, cards: &[SongCard]) -> String {
        cards
            .iter()
            .map(|card| self.generate_single_card_html(card))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate HTML for a single card
    /// 
    /// # Arguments
    /// 
    /// * `card` - The song card to generate HTML for
    /// 
    /// # Returns
    /// 
    /// HTML string for a single card
    fn generate_single_card_html(&self, card: &SongCard) -> String {
        let qr_data_url = match qr_generator::generate_qr_data_url(&card.spotify_url) {
            Ok(url) => url,
            Err(_) => "".to_string(), // Fallback to no QR code if generation fails
        };

        format!(
            r#"<div class="card bg-white rounded-lg shadow-md p-4 flex flex-col justify-between">
                <div class="flex justify-between items-start mb-2">
                    <div class="flex-1">
                        <h3 class="text-lg font-bold text-gray-900 mb-1">{title}</h3>
                        <p class="text-sm text-gray-600">{artist} ({year})</p>
                    </div>
                    {qr_html}
                </div>
                <div class="text-xs text-gray-500 mt-2">
                    Scan QR code to play on Spotify
                </div>
            </div>"#,
            title = html_escape::encode_text(&card.title),
            artist = html_escape::encode_text(&card.artist),
            year = html_escape::encode_text(&card.year),
            qr_html = if qr_data_url.is_empty() {
                "".to_string()
            } else {
                format!(
                    r#"<img src="{}" alt="QR Code" class="qr-code ml-2">"#,
                    qr_data_url
                )
            }
        )
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_generation() {
        let generator = HtmlGenerator::new();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "test_id".to_string(),
            },
        ];
        
        let result = generator.generate_html(cards, "Test Playlist", "test_output.html");
        assert!(result.is_ok());
        
        // Check if the file was created
        assert!(std::path::Path::new("test_output.html").exists());
        
        // Clean up
        std::fs::remove_file("test_output.html").unwrap();
    }

    #[test]
    fn test_build_html_content() {
        let generator = HtmlGenerator::new();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://example.com".to_string(),
            },
        ];
        
        let html = generator.build_html_content(cards, "Test Playlist");
        
        // Check that HTML contains expected elements
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Song"));
        assert!(html.contains("Test Artist"));
        assert!(html.contains("tailwindcss"));
    }
}