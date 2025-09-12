use crate::domain::Playlist;
use anyhow::Result;
use askama::filters::format;
use oxidize_pdf::{Color, Document, Font, Page};
use qrcode::render::svg;


#[trait_variant::make(IPdfGenerator: Send)]
pub trait _IPdfGenerator: Send + Sync {
    async fn generate_front_cards(&self, playlist: &Playlist) -> anyhow::Result<Vec<u8>>;
    async fn generate_back_cards(&self, playlist: &Playlist) -> anyhow::Result<Vec<u8>>;
}
pub struct PdfGenerator;

impl PdfGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IPdfGenerator for PdfGenerator {
    async fn generate_front_cards(&self, playlist: &Playlist) -> Result<Vec<u8>> {
        let mut doc = Document::new();
        doc.set_title(format!("{} - Front", playlist.name));

        // 4x6 grid = 24 cards per page
        for tracks_on_page in playlist.tracks.chunks(24) {
            let mut page = Page::a4();

            let page_width = page.width();
            let page_height = page.height();
            
            // 3 columns, 4 rows
            let cols = 3;
            let rows = 4;

            let card_width = page_width / cols as f64;
            let card_height = page_height / rows as f64;
            
            for (index, track) in tracks_on_page.iter().enumerate() {
                let row = index / cols + 1;
                let col = index % cols;

                let pos_x = col as f64 * card_width;
                let pos_y = page.height() - row as f64 * card_height;

                // Draw rectangle border
                page.graphics()
                    .set_stroke_color(Color::black())
                    .rectangle(pos_x, pos_y, card_width, card_height)
                    .stroke();

                // Add text content
                let padding = 18.0;
                let line_height = 16.0;
                // Gap between artist and title
                let gap = 4.0;

                // Handle artist name with smart wrapping
                let max_artist_chars = 20; // Approximate character limit for artist lines
                let artist_lines = wrap_text(&track.artist, max_artist_chars);
                let mut current_line = 0;
                
                for (idx, artist_line) in artist_lines.iter().enumerate() {
                    let artist_string = if idx == artist_lines.len() - 1 {
                        artist_line.clone()
                    } else {
                        // Add comma for all but the last line when we split by commas
                        if track.artist.contains(',') && idx < artist_lines.len() - 1 {
                            format!("{},", artist_line)
                        } else {
                            artist_line.clone()
                        }
                    };
                    
                    let _ = page.text()
                        .set_font(Font::Helvetica, 16.0)
                        .at(pos_x + padding,
                            pos_y + card_height - line_height - padding - (current_line as f64 * line_height))
                        .write(&artist_string);
                    current_line += 1;
                }
                
                // Add gap between artist and title
                current_line += 1;
                
                // Handle title with smart wrapping
                let max_title_chars = 30; // Approximate character limit for title lines
                let title_lines = wrap_text(&track.title, max_title_chars);
                
                for title_line in title_lines.iter() {
                    let _ = page.text()
                        .set_font(Font::Helvetica, 12.0)
                        .at(pos_x + padding,
                            pos_y + card_height - padding - gap - (current_line as f64 * line_height))
                        .write(title_line);
                    current_line += 1;
                }
                
                // Year at bottom
                let _ = page.text()
                    .set_font(Font::Helvetica, 32.0)
                    .at(pos_x + padding, pos_y + line_height + padding)
                    .write(&track.year.to_string());
            }

            doc.add_page(page);
        }

        let bytes = doc.to_bytes()?;
        Ok(bytes)
    }

    async fn generate_back_cards(&self, playlist: &Playlist) -> Result<Vec<u8>> {
        let mut doc = Document::new();
        doc.set_title(format!("{} - Back", playlist.name));

        // 4x6 grid = 24 cards per page (same as front)
        for tracks_on_page in playlist.tracks.chunks(24) {
            let mut page = Page::a4();

            let page_width = page.width();
            let page_height = page.height();
            
            // 4 columns, 6 rows
            let cols = 4;
            let rows = 6;

            let card_width = page_width / cols as f64;
            let card_height = page_height / rows as f64;
            
            for (index, track) in tracks_on_page.iter().enumerate() {
                let row = index / cols + 1;
                let col = index % cols;

                let pos_x = col as f64 * card_width;
                let pos_y = page.height() - row as f64 * card_height;

                // Draw rectangle border
                page.graphics()
                    .set_stroke_color(Color::black())
                    .rectangle(pos_x, pos_y, card_width, card_height)
                    .stroke();

                let qr_image = generate_qr_code_image(&track.spotify_url)?;
                // Create QR code png image
                let _ = page.add_image(
                    &track.spotify_url,
                    qr_image,
                );
                page.draw_image(
                    &track.spotify_url,
                    pos_x + 5.0,
                    pos_y + 5.0,
                    card_width - 10.0,
                    card_height - 10.0,
                )?;
            }

            doc.add_page(page);
        }

        let bytes = doc.to_bytes()?;
        Ok(bytes)
    }
}

fn generate_qr_code_image(url: &str) -> Result<oxidize_pdf::Image> {
    let code = qrcode::QrCode::new(url)?;
    let image = code.render::<image::Rgba<u8>>()
        .min_dimensions(200, 200)
        .build();

    let image_w = image.width();
    let image_h = image.height();

    let pdf_image = oxidize_pdf::Image::from_rgba_data(
        image.into_raw(),
        image_w,
        image_h,
    )?;

    Ok(pdf_image)
}
fn wrap_text(text: &str, max_chars_per_line: usize) -> Vec<String> {
    let mut lines = Vec::new();

    // First split by commas for multiple artists
    let comma_parts: Vec<&str> = text.split(',').map(|s| s.trim()).collect();

    for part in comma_parts {
        if part.is_empty() {
            continue;
        }

        // If this part is short enough, add it as a single line
        if part.len() <= max_chars_per_line {
            lines.push(part.to_string());
        } else {
            // Split long parts by spaces for word wrapping
            let words: Vec<&str> = part.split_whitespace().collect();
            let mut current_line = String::new();

            for word in words {
                // If adding this word would exceed the line length
                if current_line.len() + word.len() + 1 > max_chars_per_line && !current_line.is_empty() {
                    lines.push(current_line.trim().to_string());
                    current_line = String::new();
                }

                if current_line.is_empty() {
                    current_line = word.to_string();
                } else {
                    current_line.push(' ');
                    current_line.push_str(word);
                }
            }

            if !current_line.is_empty() {
                lines.push(current_line.trim().to_string());
            }
        }
    }

    lines
}
