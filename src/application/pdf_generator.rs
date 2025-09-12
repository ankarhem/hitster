use crate::domain::Playlist;
use anyhow::Result;
use printpdf::{Mm, PdfDocument, BuiltinFont};
use qrcode::QrCode;

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
        let (doc, page1, layer1) = PdfDocument::new("Front Cards", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let card_width: f32 = 52.5;
        let card_height: f32 = 49.5;
        let margin: f32 = 0.0;
        let spacing: f32 = 1.0;

        let mut card_count = 0;

        for track in &playlist.tracks {
            let row = card_count / 3;
            let col = card_count % 3;

            // Calculate position
            let x = margin + (col as f32 * (card_width + spacing));
            let y = 297.0 - margin - (row as f32 * (card_height + spacing)) - card_height;

            // Start new page if needed
            if row > 0 && row % 4 == 0 && col == 0 {
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Front Cards");
                let current_layer_ref = doc.get_page(new_page).get_layer(new_layer);
                Self::draw_front_card(&current_layer_ref, &doc, track, x, y, card_width, card_height)?;
            } else {
                Self::draw_front_card(&current_layer, &doc, track, x, y, card_width, card_height)?;
            }

            card_count += 1;
        }

        let bytes = Vec::new();
        use std::io::BufWriter;
        let mut buf_writer = BufWriter::new(bytes);
        doc.save(&mut buf_writer)?;
        Ok(buf_writer.into_inner().unwrap_or_default())
    }

    async fn generate_back_cards(&self, playlist: &Playlist) -> Result<Vec<u8>> {
        let (doc, page1, layer1) = PdfDocument::new("Back Cards", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Card dimensions (same as front)
        let card_width: f32 = 55.0;
        let card_height: f32 = 65.0;
        let margin: f32 = 10.0;
        let spacing: f32 = 5.0;

        for (i, track) in playlist.tracks.iter().enumerate() {
            let row = i / 3;
            let col = i % 3;

            // Calculate position
            let x = margin + (col as f32 * (card_width + spacing));
            let y = 297.0 - margin - (row as f32 * (card_height + spacing)) - card_height;

            // Start new page if needed
            if row > 0 && row % 4 == 0 && col == 0 {
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Back Cards");
                let current_layer_ref = doc.get_page(new_page).get_layer(new_layer);
                Self::draw_back_card(&current_layer_ref, &doc, track, x, y, card_width, card_height)?;
            } else {
                Self::draw_back_card(&current_layer, &doc, track, x, y, card_width, card_height)?;
            }
        }

        let bytes = Vec::new();
        use std::io::BufWriter;
        let mut buf_writer = BufWriter::new(bytes);
        doc.save(&mut buf_writer)?;
        Ok(buf_writer.into_inner().unwrap_or_default())
    }
}

impl PdfGenerator {
    fn draw_front_card(
        layer: &printpdf::PdfLayerReference,
        doc: &printpdf::PdfDocumentReference,
        track: &crate::domain::Track,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        // Draw card border
        layer.add_rect(printpdf::Rect::new(
            Mm(x), Mm(y),
            Mm(x + width), Mm(y + height),
        ));

        // Generate and draw QR code
        if let Ok(_code) = QrCode::new(&track.spotify_url) {
            let qr_size = 30.0; // QR code size in mm
            let qr_x = x + (width - qr_size) / 2.0;
            let qr_y = y + (height - qr_size) / 2.0;
            
            // Draw QR code placeholder (simplified - in production you'd render actual QR code)
            layer.use_text(
                "QR CODE",
                8.0,
                Mm(qr_x + 5.0),
                Mm(qr_y + qr_size / 2.0),
                &doc.add_builtin_font(BuiltinFont::Helvetica)?,
            );
        }

        Ok(())
    }

    fn draw_back_card(
        layer: &printpdf::PdfLayerReference,
        doc: &printpdf::PdfDocumentReference,
        track: &crate::domain::Track,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        // Draw card border
        layer.add_rect(printpdf::Rect::new(
            Mm(x), Mm(y),
            Mm(x + width), Mm(y + height),
        ));

        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        // Add track information
        layer.use_text(&track.title, 8.0, Mm(x + 3.0), Mm(y + height - 10.0), &font);
        layer.use_text(&track.artist, 7.0, Mm(x + 3.0), Mm(y + height - 20.0), &font);
        layer.use_text(&track.year.to_string(), 6.0, Mm(x + 3.0), Mm(y + height - 28.0), &font);

        Ok(())
    }
}