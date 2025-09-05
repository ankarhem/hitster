use crate::infrastructure::{Playlist, Track};
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, instrument};

pub struct PdfGenerator;

impl PdfGenerator {
    #[instrument(skip(playlist, tracks))]
    pub async fn generate_pdfs(playlist: &Playlist, tracks: &[Track]) -> Result<(String, String)> {
        // Create output directory if it doesn't exist
        let output_dir = PathBuf::from("generated_pdfs");
        fs::create_dir_all(&output_dir).await?;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let base_filename = format!("{}_{}", playlist.id, timestamp);
        
        // Generate front PDF
        let front_path = output_dir.join(format!("{}_front.pdf", base_filename));
        let front_pdf_content = Self::generate_front_pdf(playlist, tracks).await?;
        fs::write(&front_path, front_pdf_content).await?;
        
        // Generate back PDF
        let back_path = output_dir.join(format!("{}_back.pdf", base_filename));
        let back_pdf_content = Self::generate_back_pdf(tracks.len()).await?;
        fs::write(&back_path, back_pdf_content).await?;
        
        info!("Generated PDFs for playlist {}: {:?}, {:?}", playlist.id, front_path, back_path);
        
        Ok((
            front_path.to_string_lossy().to_string(),
            back_path.to_string_lossy().to_string(),
        ))
    }
    
    async fn generate_front_pdf(playlist: &Playlist, tracks: &[Track]) -> Result<Vec<u8>> {
        // Simple HTML-based PDF generation for now
        // In a real implementation, you'd use a proper PDF library like printpdf
        let _html = Self::generate_front_html(playlist, tracks);
        
        // For now, return a simple PDF placeholder
        // This would be replaced with actual PDF generation
        Ok(format!("PDF content for {} tracks", tracks.len()).into_bytes())
    }
    
    async fn generate_back_pdf(track_count: usize) -> Result<Vec<u8>> {
        // Generate back PDF with uniform card backs
        let _html = Self::generate_back_html(track_count);
        
        // For now, return a simple PDF placeholder
        // This would be replaced with actual PDF generation
        Ok(format!("PDF back content for {} tracks", track_count).into_bytes())
    }
    
    fn generate_front_html(playlist: &Playlist, tracks: &[Track]) -> String {
        let mut html = String::new();
        
        html.push_str(&format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>{} - Front Cards</title>
    <style>
        @page {{ size: A4; margin: 0; }}
        body {{ margin: 0; padding: 20px; font-family: Arial, sans-serif; }}
        .card-grid {{ 
            display: grid; 
            grid-template-columns: repeat(3, 1fr); 
            grid-template-rows: repeat(4, 1fr);
            gap: 10px;
            height: 257mm;
            width: 177mm;
        }}
        .card {{ 
            border: 1px solid #ccc; 
            padding: 8px;
            font-size: 10px;
            display: flex;
            flex-direction: column;
            page-break-inside: avoid;
        }}
        .card-title {{ font-weight: bold; margin-bottom: 4px; }}
        .card-artist {{ color: #666; margin-bottom: 2px; }}
        .card-year {{ color: #999; font-size: 9px; }}
        .card-qr {{ margin-top: auto; text-align: center; }}
        .qr-placeholder {{ 
            width: 30px; 
            height: 30px; 
            background: #f0f0f0; 
            border: 1px solid #ddd;
            display: inline-block;
        }}
    </style>
</head>
<body>
"#,
            playlist.name
        ));
        
        // Generate cards in chunks of 12 per page
        for (page, chunk) in tracks.chunks(12).enumerate() {
            if page > 0 {
                html.push_str(r#"<div style="page-break-before: always;"></div>"#);
            }
            
            html.push_str(r#"<div class="card-grid">"#);
            
            for track in chunk {
                html.push_str(&format!(
                    r#"
<div class="card">
    <div class="card-title">{}</div>
    <div class="card-artist">{}</div>
    <div class="card-year">{}</div>
    <div class="card-qr">
        <div class="qr-placeholder" title="QR Code: {}"></div>
    </div>
</div>
"#,
                    html_escape::encode_text(&track.title),
                    html_escape::encode_text(&track.artist),
                    track.year,
                    track.spotify_url
                ));
            }
            
            html.push_str("</div>");
        }
        
        html.push_str("</body></html>");
        html
    }
    
    fn generate_back_html(track_count: usize) -> String {
        let mut html = String::new();
        
        html.push_str(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Card Backs</title>
    <style>
        @page { size: A4; margin: 0; }
        body { margin: 0; padding: 20px; font-family: Arial, sans-serif; }
        .card-grid { 
            display: grid; 
            grid-template-columns: repeat(3, 1fr); 
            grid-template-rows: repeat(4, 1fr);
            gap: 10px;
            height: 257mm;
            width: 177mm;
        }
        .card-back { 
            border: 1px solid #ccc; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-size: 14px;
            font-weight: bold;
            text-align: center;
            page-break-inside: avoid;
        }
    </style>
</head>
<body>
"#
        );
        
        // Generate back cards in chunks of 12 per page
        for page in 0..track_count.div_ceil(12) {
            if page > 0 {
                html.push_str(r#"<div style="page-break-before: always;"></div>"#);
            }
            
            html.push_str(r#"<div class="card-grid">"#);
            
            let cards_on_page = if page == track_count.div_ceil(12) - 1 {
                let remainder = track_count % 12;
                if remainder == 0 { 12 } else { remainder }
            } else {
                12
            };
            
            for _i in 0..cards_on_page {
                html.push_str(r#"<div class="card-back">HITSTER</div>"#);
            }
            
            html.push_str("</div>");
        }
        
        html.push_str("</body></html>");
        html
    }
}

/// Public function to generate PDFs
pub async fn generate_pdfs(playlist: &Playlist, tracks: &[Track]) -> Result<(String, String)> {
    PdfGenerator::generate_pdfs(playlist, tracks).await
}