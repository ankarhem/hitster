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

            let card_width = page_width / cols as f64;
            let card_height = page_height / rows as f64;
            
            for (index, _track) in tracks_on_page.iter().enumerate() {
                dbg!(index);
                
                let row = index / cols + 1;
                let col = index % cols;

                let pos_x = col as f64 * card_width;
                let pos_y = page.height() - row as f64 * card_height;

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