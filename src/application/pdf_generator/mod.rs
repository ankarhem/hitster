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


        // 6x4 grid = 24 cards per page
        for tracks_on_page in playlist.tracks.chunks(24) {
            let mut page = Page::a4();

            let width = page.width() / 4.0 - 4.0;
            let height = page.height() / 6.0 - 6.0;
            for (idx_col, rows) in tracks_on_page.chunks(4).enumerate() {
                for (idx_row, track) in rows.iter().enumerate() {
                    let pos_x = 0.0 + (idx_row as f64 * width);
                    let pos_y = 842.0 - (idx_col as f64 * height);
                    page.graphics()
                        .set_stroke_color(Color::black())
                        .rectangle(pos_x, pos_y, width, height)
                        .stroke();
                }
            }

            doc.add_page(page);
        }

        let bytes = doc.to_bytes()?;
        Ok(bytes)
    }

    async fn generate_back_cards(&self, playlist: &Playlist) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}