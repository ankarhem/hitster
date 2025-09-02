use anyhow::Result;
use crate::SongCard;
use base64::Engine;

#[derive(Clone)]
pub struct HtmlGenerator {
    card_width_mm: f64,
    card_height_mm: f64,
    margin_mm: f64,
}

impl HtmlGenerator {
    pub fn new() -> Self {
        Self {
            card_width_mm: 90.0,  // Standard business card width in mm
            card_height_mm: 55.0, // Standard business card height in mm
            margin_mm: 5.0,
        }
    }

    pub fn generate_html(&self, cards: Vec<SongCard>, title: &str, output_path: &str) -> Result<()> {
        let html_content = self.build_html_content(cards, title);
        std::fs::write(output_path, html_content)?;
        Ok(())
    }

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

    fn generate_cards_html(&self, cards: &[SongCard]) -> String {
        cards
            .iter()
            .map(|card| self.generate_single_card_html(card))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_single_card_html(&self, card: &SongCard) -> String {
        let qr_data_url = match self.generate_qr_data_url(&card.spotify_url) {
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

    fn generate_qr_data_url(&self, url: &str) -> Result<String> {
        use qrcode::QrCode;
        use image::{ImageBuffer, Luma};
        
        // Generate QR code as character-based art first
        let qr = QrCode::new(url)?;
        let qr_string = qr.render::<char>()
            .quiet_zone(true)
            .module_dimensions(10, 10)
            .build();
        
        // Convert character-based QR to image
        let lines: Vec<&str> = qr_string.split('\n').collect();
        let width = lines.first().map(|l| l.len()).unwrap_or(0) as u32;
        let height = lines.len() as u32;
        let module_size = 4; // pixels per module
        
        // Create grayscale image buffer
        let mut gray_image = ImageBuffer::<Luma<u8>, Vec<u8>>::new(width * module_size, height * module_size);
        
        // Draw QR code modules
        for (img_y, line) in lines.iter().enumerate() {
            for (img_x, ch) in line.chars().enumerate() {
                let is_black = ch != ' ';
                
                // Fill the module area
                for dy in 0..module_size {
                    for dx in 0..module_size {
                        let px = img_x as u32 * module_size + dx;
                        let py = img_y as u32 * module_size + dy;
                        
                        if px < width * module_size && py < height * module_size {
                            let color = if is_black {
                                image::Luma([0]) // Black
                            } else {
                                image::Luma([255]) // White
                            };
                            gray_image.put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
        
        // Convert QR code to PNG bytes
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, gray_image.width(), gray_image.height());
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(gray_image.as_raw())?;
        }
        
        // Convert to base64 data URL
        let base64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
        Ok(format!("data:image/png;base64,{}", base64))
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