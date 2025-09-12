use crate::domain::Playlist;
use anyhow::Result;
use oxidize_pdf::{Color, Document, Page};

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
            
            // 4 columns, 6 rows
            let cols = 4;
            let rows = 6;
            
            // Calculate card dimensions with margins
            let margin = 10.0; // 10mm margin
            let spacing = 2.0; // 2mm spacing between cards
            
            let available_width = page_width - 2.0 * margin;
            let available_height = page_height - 2.0 * margin;
            
            let card_width = (available_width - (cols - 1) as f64 * spacing) / cols as f64;
            let card_height = (available_height - (rows - 1) as f64 * spacing) / rows as f64;
            
            for (index, _track) in tracks_on_page.iter().enumerate() {
                let row = index / cols;
                let col = index % cols;
                
                let pos_x = margin + col as f64 * (card_width + spacing);
                let pos_y = margin + row as f64 * (card_height + spacing);
                
                page.graphics()
                    .set_stroke_color(Color::black())
                    .rectangle(pos_x, pos_y, card_width, card_height)
                    .stroke();
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