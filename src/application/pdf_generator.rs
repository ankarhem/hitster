use crate::domain::Playlist;
use anyhow::Result;
use printpdf::PdfDocument;
use std::collections::BTreeMap;

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
        let html_content = Self::generate_front_cards_html(playlist)?;
        
        let images = BTreeMap::new();
        let fonts = BTreeMap::new();
        let mut warnings = Vec::new();
        
        let doc = PdfDocument::from_html(&html_content, &images, &fonts, &Default::default(), &mut warnings)
            .map_err(|e| anyhow::anyhow!(e))?;
        
        let pdf_bytes = doc.save(&Default::default(), &mut warnings);
        Ok(pdf_bytes)
    }

    async fn generate_back_cards(&self, playlist: &Playlist) -> Result<Vec<u8>> {
        let html_content = Self::generate_back_cards_html(playlist)?;
        
        let images = BTreeMap::new();
        let fonts = BTreeMap::new();
        let mut warnings = Vec::new();
        
        let doc = PdfDocument::from_html(&html_content, &images, &fonts, &Default::default(), &mut warnings)
            .map_err(|e| anyhow::anyhow!(e))?;
        
        let pdf_bytes = doc.save(&Default::default(), &mut warnings);
        Ok(pdf_bytes)
    }
}

impl PdfGenerator {
    fn generate_front_cards_html(playlist: &Playlist) -> Result<String> {
        let mut html = String::new();
        
        // HTML header with styles for 4x6 grid
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {
            margin: 0;
            padding: 15mm;
            width: 180mm;  /* 210mm - 30mm margins */
            height: 267mm; /* 297mm - 30mm margins */
            box-sizing: border-box;
        }
        .card-grid {
            display: grid;
            grid-template-columns: repeat(4, 45mm);
            grid-template-rows: repeat(6, 42mm);
            gap: 2mm;
            width: 100%;
            height: 100%;
        }
        .card {
            border: 1px solid black;
            display: flex;
            align-items: center;
            justify-content: center;
            text-align: center;
            font-family: Arial, sans-serif;
            font-size: 12px;
        }
        @page {
            size: 210mm 297mm;
            margin: 0;
        }
    </style>
</head>
<body>"#);

        let mut cards_on_page = 0;
        
        for (i, _track) in playlist.tracks.iter().enumerate() {
            // Start new page if needed (24 cards per page)
            if i > 0 && i % 24 == 0 {
                html.push_str("</div></body></html>");
                html.push_str(r#"<html><head><style>
                body { margin: 0; padding: 15mm; width: 180mm; height: 267mm; box-sizing: border-box; }
                .card-grid { display: grid; grid-template-columns: repeat(4, 45mm); grid-template-rows: repeat(6, 42mm); gap: 2mm; width: 100%; height: 100%; }
                .card { border: 1px solid black; display: flex; align-items: center; justify-content: center; text-align: center; font-family: Arial, sans-serif; font-size: 12px; }
                @page { size: 210mm 297mm; margin: 0; }
                </style></head><body>"#);
                cards_on_page = 0;
            }
            
            // Start new grid for each page
            if cards_on_page == 0 {
                html.push_str(r#"<div class="card-grid">"#);
            }
            
            // Add card
            html.push_str(r#"<div class="card">QR CODE</div>"#);
            
            cards_on_page += 1;
            
            // Close grid at end of page
            if cards_on_page == 24 || i == playlist.tracks.len() - 1 {
                html.push_str("</div>");
            }
        }
        
        html.push_str("</body></html>");
        Ok(html)
    }
    
    fn generate_back_cards_html(playlist: &Playlist) -> Result<String> {
        let mut html = String::new();
        
        // HTML header with styles for 4x6 grid
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {
            margin: 0;
            padding: 15mm;
            width: 180mm;  /* 210mm - 30mm margins */
            height: 267mm; /* 297mm - 30mm margins */
            box-sizing: border-box;
        }
        .card-grid {
            display: grid;
            grid-template-columns: repeat(4, 45mm);
            grid-template-rows: repeat(6, 42mm);
            gap: 2mm;
            width: 100%;
            height: 100%;
        }
        .card {
            border: 1px solid black;
            padding: 2mm;
            display: flex;
            flex-direction: column;
            font-family: Arial, sans-serif;
        }
        .track-title {
            font-size: 14px;
            font-weight: bold;
            margin-bottom: 1mm;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        .track-artist {
            font-size: 11px;
            margin-bottom: 1mm;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        .track-year {
            font-size: 9px;
            color: #666;
        }
        @page {
            size: 210mm 297mm;
            margin: 0;
        }
    </style>
</head>
<body>"#);

        let mut cards_on_page = 0;
        
        for (i, track) in playlist.tracks.iter().enumerate() {
            // Start new page if needed (24 cards per page)
            if i > 0 && i % 24 == 0 {
                html.push_str("</div></body></html>");
                html.push_str(r#"<html><head><style>
                body { margin: 0; padding: 15mm; width: 180mm; height: 267mm; box-sizing: border-box; }
                .card-grid { display: grid; grid-template-columns: repeat(4, 45mm); grid-template-rows: repeat(6, 42mm); gap: 2mm; width: 100%; height: 100%; }
                .card { border: 1px solid black; padding: 2mm; display: flex; flex-direction: column; font-family: Arial, sans-serif; }
                .track-title { font-size: 14px; font-weight: bold; margin-bottom: 1mm; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
                .track-artist { font-size: 11px; margin-bottom: 1mm; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
                .track-year { font-size: 9px; color: #666; }
                @page { size: 210mm 297mm; margin: 0; }
                </style></head><body>"#);
                cards_on_page = 0;
            }
            
            // Start new grid for each page
            if cards_on_page == 0 {
                html.push_str(r#"<div class="card-grid">"#);
            }
            
            // Add card with track info (escape HTML content)
            let title = html_escape::encode_text(&track.title);
            let artist = html_escape::encode_text(&track.artist);
            
            html.push_str(&format!(r#"
        <div class="card">
            <div class="track-title">{}</div>
            <div class="track-artist">{}</div>
            <div class="track-year">{}</div>
        </div>"#, title, artist, track.year));
            
            cards_on_page += 1;
            
            // Close grid at end of page
            if cards_on_page == 24 || i == playlist.tracks.len() - 1 {
                html.push_str("</div>");
            }
        }
        
        html.push_str("</body></html>");
        Ok(html)
    }
}