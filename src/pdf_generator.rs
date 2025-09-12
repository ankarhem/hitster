use crate::domain::Playlist;
use anyhow::Result;
use printpdf::{Mm, PdfDocument, BuiltinFont};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, instrument};

pub struct PdfGenerator;

impl PdfGenerator {
    #[instrument(skip(playlist), fields(playlist_id = %playlist.id))]
    pub async fn generate_pdfs(playlist: &Playlist) -> Result<(String, String)> {
        // Create output directory if it doesn't exist
        let output_dir = PathBuf::from("generated_pdfs");
        fs::create_dir_all(&output_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let base_filename = format!("{}_{}", playlist.id, timestamp);

        // Generate front PDF
        let front_path = output_dir.join(format!("{}_front.pdf", base_filename));
        let front_pdf_content = Self::generate_front_pdf(playlist).await?;
        fs::write(&front_path, front_pdf_content).await?;

        // Generate back PDF
        let back_path = output_dir.join(format!("{}_back.pdf", base_filename));
        let back_pdf_content = Self::generate_back_pdf(playlist.tracks.len()).await?;
        fs::write(&back_path, back_pdf_content).await?;

        info!(
            "Generated PDFs for playlist {}: {:?}, {:?}",
            playlist.id, front_path, back_path
        );

        Ok((
            front_path.to_string_lossy().to_string(),
            back_path.to_string_lossy().to_string(),
        ))
    }

    async fn generate_front_pdf(playlist: &Playlist) -> Result<Vec<u8>> {
        let (doc, page1, layer1) = PdfDocument::new("Hitster Cards", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        // Simple PDF with playlist info for now
        current_layer.use_text(
            &format!("Playlist: {}", playlist.name),
            16.0,
            Mm(20.0),
            Mm(277.0),
            &font,
        );

        current_layer.use_text(
            &format!("Total tracks: {}", playlist.tracks.len()),
            12.0,
            Mm(20.0),
            Mm(267.0),
            &font,
        );

        for (i, track) in playlist.tracks.iter().take(10).enumerate() {
            let y = 247.0 - (i as f32 * 10.0);
            current_layer.use_text(
                &format!("{}. {} - {} ({})", i + 1, track.title, track.artist, track.year),
                10.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
        }

        if playlist.tracks.len() > 10 {
            current_layer.use_text(
                &format!("... and {} more tracks", playlist.tracks.len() - 10),
                10.0,
                Mm(20.0),
                Mm(137.0),
                &font,
            );
        }

        let bytes = Vec::new();
        use std::io::BufWriter;
        let mut buf_writer = BufWriter::new(bytes);
        doc.save(&mut buf_writer)?;
        Ok(buf_writer.into_inner().unwrap_or_default())
    }

    async fn generate_back_pdf(track_count: usize) -> Result<Vec<u8>> {
        let (doc, page1, layer1) = PdfDocument::new("Hitster Card Backs", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);
        let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

        current_layer.use_text(
            "HITSTER - Card Backs",
            24.0,
            Mm(70.0),
            Mm(200.0),
            &font,
        );

        current_layer.use_text(
            &format!("Total cards: {}", track_count),
            16.0,
            Mm(70.0),
            Mm(170.0),
            &font,
        );

        let bytes = Vec::new();
        use std::io::BufWriter;
        let mut buf_writer = BufWriter::new(bytes);
        doc.save(&mut buf_writer)?;
        Ok(buf_writer.into_inner().unwrap_or_default())
    }
}

/// Public function to generate PDFs
pub async fn generate_pdfs(playlist: &Playlist) -> Result<(String, String)> {
    PdfGenerator::generate_pdfs(playlist).await
}
