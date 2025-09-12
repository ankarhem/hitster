use crate::domain::Playlist;
use anyhow::Result;
use printpdf::PdfDocument;
use std::collections::BTreeMap;
use askama::Template;
use tracing::info;

mod html;

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
        let template = html::PdfTemplate {
            title: playlist.name.clone(),
            track_chunks: playlist
                .clone()
                .tracks
                .chunks(24)
                .map(|tracks| {
                    tracks.chunks(6).map(|row| {
                        row
                            .iter()
                            .map(|track| {
                                html::PdfTrack {
                                    title: track.title.clone(),
                                    artist: track.artist.clone(),
                                    year: track.year,
                                    qr_code: "qr_code".to_string(),
                                }})
                            .collect()
                    }).collect()
                })
                .collect::<Vec<_>>(),
        };
        let html_content = template.render()?;
        let images = BTreeMap::new();
        let fonts = BTreeMap::new();
        let mut warnings = Vec::new();

        let doc = PdfDocument::from_html(&html_content, &images, &fonts, &Default::default(), &mut warnings)
            .map_err(|e| anyhow::anyhow!(e))?;

        info!("PDF generation warnings: {:?}", warnings);
        
        let pdf_bytes = doc.save(&Default::default(), &mut warnings);
        Ok(pdf_bytes)
    }

    async fn generate_back_cards(&self, playlist: &Playlist) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}