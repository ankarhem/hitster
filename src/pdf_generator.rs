use anyhow::Result;

use crate::SongCard;
use oxidize_pdf::{Document, Page, Font, Color};
use image::RgbImage;

pub struct PdfGenerator {
    card_width: f64,
    card_height: f64,
    margin: f64,
}

impl PdfGenerator {
    pub fn new() -> Self {
        Self {
            card_width: 90.0,  // Standard business card width in mm
            card_height: 55.0, // Standard business card height in mm
            margin: 5.0,
        }
    }

    pub fn generate_pdf(&self, cards: Vec<SongCard>, title: &str, output_path: &str) -> Result<()> {
        
        // Create PDF document
        let mut doc = Document::new();
        
        // Create pages (2x5 grid on A4)
        let cards_per_row = 2;
        let cards_per_col = 5;
        let cards_per_page = cards_per_row * cards_per_col;
        
        let pages = (cards.len() + cards_per_page - 1) / cards_per_page;
        
        for page_num in 0..pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, cards.len());
            
            // Create a new A4 page
            let mut page = Page::a4();
            
            // Add cards to the page
            for card_idx in start_idx..end_idx {
                let card_index_on_page = card_idx - start_idx;
                let row = card_index_on_page / cards_per_row;
                let col = card_index_on_page % cards_per_row;
                
                let x = self.margin + col as f64 * (self.card_width + self.margin);
                let y = self.margin + row as f64 * (self.card_height + self.margin);
                
                self.add_card_to_page(&mut page, &cards[card_idx], title, x, y)?;
            }
            
            // Add the page to the document
            doc.add_page(page);
        }
        
        // Save the document
        doc.save(output_path)?;
        Ok(())
    }
    
    fn add_card_to_page(&self, page: &mut Page, card: &SongCard, title: &str, x: f64, y: f64) -> Result<()> {
        
        // Convert mm to points (1 mm = 2.83465 points)
        let x_pt = x * 2.83465;
        let y_pt = y * 2.83465;
        let width_pt = self.card_width * 2.83465;
        let height_pt = self.card_height * 2.83465;
        
        // Add card border
        page.graphics()
            .set_stroke_color(Color::black())
            .set_line_width(1.0)
            .rect(x_pt, y_pt, width_pt, height_pt)
            .stroke();
        
        // Add text content
        let text = page.text();
        
        // Title
        text.set_font(Font::Helvetica, 12.0)
            .at(x_pt + 14.17, y_pt + height_pt - 28.35) // 5mm, 10mm from top-right
            .write(title)?;
        
        // Song title
        text.set_font(Font::Helvetica, 10.0)
            .at(x_pt + 14.17, y_pt + height_pt - 70.87) // 5mm, 25mm from top-right
            .write(&card.title)?;
        
        // Artist and year
        text.set_font(Font::Helvetica, 8.0)
            .at(x_pt + 14.17, y_pt + height_pt - 113.39) // 5mm, 40mm from top-right
            .write(&format!("{} ({})", card.artist, card.year))?;
        
        // Add QR code
        self.add_qr_code(page, &card.spotify_url, x_pt + width_pt - 45.35, y_pt + 14.17)?;
        
        Ok(())
    }
    
    fn add_qr_code(&self, page: &mut Page, url: &str, x: f64, y: f64) -> Result<()> {
        use qrcode::QrCode;
        use oxidize_pdf::Image;
        
        // Generate QR code as character-based art first
        let qr = QrCode::new(url)?;
        let qr_string = qr.render::<char>()
            .quiet_zone(true)
            .module_dimensions(20, 20)
            .build();
        
        // Convert character-based QR to image
        let lines: Vec<&str> = qr_string.split('\n').collect();
        let width = lines.first().map(|l| l.len()).unwrap_or(0) as u32;
        let height = lines.len() as u32;
        let module_size = 4; // pixels per module
        
        // Create RGB image buffer
        let mut rgb_image = RgbImage::new(width * module_size, height * module_size);
        
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
                                image::Rgb([0, 0, 0]) // Black
                            } else {
                                image::Rgb([255, 255, 255]) // White
                            };
                            rgb_image.put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
        
        // Convert QR code to PNG bytes
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, rgb_image.width(), rgb_image.height());
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(rgb_image.as_raw())?;
        }
        
        // Create PDF image from PNG data
        let pdf_image = Image::from_png_data(png_data)?;
        
        // Register the image with a unique name
        let image_name = format!("qr_code_{}", url.replace("/", "_").replace(":", "_"));
        page.add_image(&image_name, pdf_image);
        
        // Calculate QR code size in points (make it 30mm x 30mm)
        let qr_size = 30.0 * 2.83465; // 30mm in points
        
        // Draw the QR code image using its registered name
        page.draw_image(&image_name, x, y, qr_size, qr_size)?;
        
        Ok(())
    }
}

impl Default for PdfGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pdf_generation() {
        let generator = PdfGenerator::new();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "test_id".to_string(),
            },
        ];
        
        let result = generator.generate_pdf(cards, "Test Playlist", "test_output.pdf");
        assert!(result.is_ok());
        
        // Check if the file was created
        assert!(std::path::Path::new("test_output.pdf").exists());
        
        // Print file info for debugging
        if let Ok(metadata) = std::fs::metadata("test_output.pdf") {
            println!("PDF file size: {} bytes", metadata.len());
        }
        
        // Clean up
        std::fs::remove_file("test_output.pdf").unwrap();
    }
}