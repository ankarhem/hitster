use crate::domain::Playlist;
use anyhow::Result;
use oxidize_pdf::{Color, Document, Font, Page};

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
        doc.set_title("My First PDF");
        doc.set_author("Rust Developer");


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
                let text_margin = 8.0;
                let line_height = 16.0;
                let padding = 10.0;
                
                // Handle artist name - split by commas for multiple artists
                let artists: Vec<&str> = track.artist.split(',').map(|s| s.trim()).collect();
                let mut current_line = 0;
                
                for (idx, artist) in artists.iter().enumerate() {
                    let artist_string = match idx == artists.len() - 1 {
                        true => artist.to_string(),
                        false => format!("{},", artist),
                    };
                    if !artist.is_empty() {
                        let _ = page.text()
                            .set_font(Font::Helvetica, 16.0)
                            .at(pos_x + text_margin + padding, 
                                pos_y + card_height - text_margin - line_height - padding - (current_line as f64 * line_height))
                            .write(&artist_string);
                        current_line += 1;
                    }
                }
                
                // Title - place it after all artist lines
                let _ = page.text()
                    .set_font(Font::Helvetica, 12.0)
                    .at(pos_x + text_margin + padding, 
                        pos_y + card_height - text_margin - line_height - padding - (current_line as f64 * line_height) - 4.0)
                    .write(&track.title);
                
                // Year at bottom
                let _ = page.text()
                    .set_font(Font::Helvetica, 32.0)
                    .at(pos_x + text_margin + padding, pos_y + line_height + padding)
                    .write(&track.year.to_string());
            }

            doc.add_page(page);
        }

        let bytes = doc.to_bytes()?;
        Ok(bytes)
    }

    async fn generate_back_cards(&self, _playlist: &Playlist) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}