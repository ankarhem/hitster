//! QR code generation utilities
//! 
//! This module handles the generation of QR codes as base64-encoded PNG images
//! for embedding in HTML documents.

use anyhow::Result;
use base64::Engine;
use qrcode::QrCode;
use image::{ImageBuffer, Luma};

/// Generate a QR code as a base64-encoded data URL
/// 
/// This function creates a QR code for the given URL and converts it to
/// a base64-encoded PNG image suitable for embedding in HTML.
/// 
/// # Arguments
/// 
/// * `url` - The URL to encode in the QR code
/// 
/// # Returns
/// 
/// A base64 data URL string (e.g., "data:image/png;base64,...")
/// 
/// # Errors
/// 
/// Returns an error if QR code generation or PNG encoding fails
pub fn generate_qr_data_url(url: &str) -> Result<String> {
    // Generate QR code as character-based art first
    let qr = QrCode::new(url)?;
    let qr_string = qr.render::<char>()
        .quiet_zone(true)
        .module_dimensions(10, 10)
        .build();
    
    // Convert character-based QR to image
    let lines: Vec<&str> = qr_string.split('\n').collect();
    let width = lines.first().map(|l| l.len()).unwrap_or(0) as u32;
    let height = lines.len() as u32;
    let module_size = 4; // pixels per module
    
    // Create grayscale image buffer
    let mut gray_image = ImageBuffer::<Luma<u8>, Vec<u8>>::new(width * module_size, height * module_size);
    
    // Draw QR code modules
    for (img_y, line) in lines.iter().enumerate() {
        for (img_x, ch) in line.chars().enumerate() {
            let is_black = ch != ' ';
            
            // Fill the module area
            for dy in 0..module_size {
                for dx in 0..module_size {
                    let px = img_x as u32 * module_size + dx;
                    let py = img_y as u32 * module_size + dy;
                    
                    if px < width * module_size && py < height * module_size {
                        let color = if is_black {
                            image::Luma([0]) // Black
                        } else {
                            image::Luma([255]) // White
                        };
                        gray_image.put_pixel(px, py, color);
                    }
                }
            }
        }
    }
    
    // Convert QR code to PNG bytes
    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_data, gray_image.width(), gray_image.height());
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(gray_image.as_raw())?;
    }
    
    // Convert to base64 data URL
    let base64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(format!("data:image/png;base64,{}", base64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_generation() {
        let result = generate_qr_data_url("https://example.com");
        assert!(result.is_ok());
        
        let url = result.unwrap();
        assert!(url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_qr_with_empty_url() {
        let result = generate_qr_data_url("");
        assert!(result.is_ok());
    }
}