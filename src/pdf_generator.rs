use anyhow::Result;

use crate::SongCard;

pub struct PdfGenerator {
    card_width: f64,
    card_height: f64,
    margin: f64,
}

impl PdfGenerator {
    pub fn new() -> Self {
        Self {
            card_width: 90.0,  // Standard business card width in mm
            card_height: 55.0, // Standard business card height in mm
            margin: 5.0,
        }
    }

    pub fn generate_pdf(&self, cards: Vec<SongCard>, title: &str, output_path: &str) -> Result<()> {
        use lopdf::{Dictionary, Object, content::Content};
        
        // Create a simple PDF document
        let mut doc = lopdf::Document::with_version("1.5");
        
        // Create pages (2x5 grid on A4)
        let cards_per_row = 2;
        let cards_per_col = 5;
        let cards_per_page = cards_per_row * cards_per_col;
        
        let pages = (cards.len() + cards_per_page - 1) / cards_per_page;
        let mut page_ids = Vec::new();
        
        for page_num in 0..pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, cards.len());
            
            // Create content operations for this page
            let mut operations = Vec::new();
            
            for card_idx in start_idx..end_idx {
                let card_index_on_page = card_idx - start_idx;
                let row = card_index_on_page / cards_per_row;
                let col = card_index_on_page % cards_per_row;
                
                let x = self.margin + col as f64 * (self.card_width + self.margin);
                let y = 297.0 - self.margin - (row + 1) as f64 * (self.card_height + self.margin);
                
                operations.extend(self.create_card_operations(&cards[card_idx], title, x, y));
            }
            
            // Create page with content
            let content = Content { operations };
            let content_id = doc.add_object(lopdf::Stream::new(Dictionary::new(), content.encode().unwrap()));
            
            let page_dict = Dictionary::from_iter([
                ("Type", Object::Name("Page".into())),
                ("MediaBox", Object::Array(vec![0.into(), 0.into(), 595.into(), 842.into()])), // A4 size
                ("Contents", content_id.into()),
                ("Resources", Object::Dictionary(Dictionary::from_iter([
                    ("Font", Object::Dictionary(Dictionary::from_iter([
                        ("F1", Object::Dictionary(Dictionary::from_iter([
                            ("Type", Object::Name("Font".into())),
                            ("Subtype", Object::Name("Type1".into())),
                            ("BaseFont", Object::Name("Helvetica".into())),
                            ("Encoding", Object::Name("WinAnsiEncoding".into())),
                        ]))),
                    ]))),
                ]))),
            ]);
            
            let page_id = doc.add_object(page_dict);
            page_ids.push(page_id);
        }
        
        // Create pages root
        let page_refs: Vec<Object> = page_ids.iter().map(|&id| Object::Reference(id)).collect();
        let pages_dict = Dictionary::from_iter([
            ("Type", Object::Name("Pages".into())),
            ("Kids", Object::Array(page_refs)),
            ("Count", (page_ids.len() as i32).into()),
            ("MediaBox", Object::Array(vec![0.into(), 0.into(), 595.into(), 842.into()])), // A4 size
        ]);
        
        let pages_id = doc.add_object(pages_dict);
        
        // Update pages to reference parent
        for &page_id in &page_ids {
            if let Some(Object::Dictionary(dict)) = doc.objects.get_mut(&page_id) {
                dict.set("Parent", Object::Reference(pages_id));
            }
        }
        
        // Create catalog
        let catalog_dict = Dictionary::from_iter([
            ("Type", Object::Name("Catalog".into())),
            ("Pages", Object::Reference(pages_id)),
        ]);
        
        let catalog_id = doc.add_object(catalog_dict);
        
        // Set document catalog
        doc.trailer.set("Root", Object::Reference(catalog_id));
        
        // Save the document
        doc.save(output_path)?;
        Ok(())
    }
    
    fn create_card_operations(&self, card: &SongCard, title: &str, x: f64, y: f64) -> Vec<lopdf::content::Operation> {
        use lopdf::content::*;
        
        let mut operations = Vec::new();
        
        // Add card border
        let (x_pt, y_pt) = self.mm_to_points(x, y);
        let (width_pt, height_pt) = self.mm_to_points(self.card_width, self.card_height);
        
        operations.push(Operation::new("q", vec![])); // Save graphics state
        operations.push(Operation::new("w", vec![1.0.into()])); // Line width
        operations.push(Operation::new("re", vec![
            x_pt.into(),
            y_pt.into(),
            width_pt.into(),
            height_pt.into()
        ]));
        operations.push(Operation::new("S", vec![])); // Stroke
        operations.push(Operation::new("Q", vec![])); // Restore graphics state
        
        // Add text content
        operations.extend(self.create_text_operation(title, x + 5.0, y + self.card_height - 10.0, 12.0));
        operations.extend(self.create_text_operation(&card.title, x + 5.0, y + self.card_height - 25.0, 10.0));
        operations.extend(self.create_text_operation(&format!("{} ({})", card.artist, card.year), x + 5.0, y + self.card_height - 40.0, 8.0));
        
        // Add QR code placeholder text
        operations.extend(self.create_text_operation("QR Code", x + self.card_width - 25.0, y + 10.0, 8.0));
        
        operations
    }
    
    fn mm_to_points(&self, x: f64, y: f64) -> (f64, f64) {
        // Convert mm to points (1 mm = 2.83465 points)
        (x * 2.83465, y * 2.83465)
    }
    
    fn create_text_operation(&self, text: &str, x: f64, y: f64, font_size: f64) -> Vec<lopdf::content::Operation> {
        use lopdf::content::*;
        use lopdf::Object;
        
        let (x_pt, y_pt) = self.mm_to_points(x, y);
        
        vec![
            Operation::new("BT", vec![]), // Begin text
            Operation::new("Tf", vec!["F1".into(), font_size.into()]), // Font and size
            Operation::new("Td", vec![x_pt.into(), y_pt.into()]), // Position
            Operation::new("Tj", vec![Object::string_literal(text)]), // Text - FIXED
            Operation::new("ET", vec![]), // End text
        ]
    }
}

impl Default for PdfGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pdf_generation() {
        let generator = PdfGenerator::new();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "test_id".to_string(),
            },
        ];
        
        let result = generator.generate_pdf(cards, "Test Playlist", "test_output.pdf");
        assert!(result.is_ok());
        
        // Check if the file was created
        assert!(std::path::Path::new("test_output.pdf").exists());
        
        // Print file info for debugging
        if let Ok(metadata) = std::fs::metadata("test_output.pdf") {
            println!("PDF file size: {} bytes", metadata.len());
            
            // Read and print first few bytes to see PDF header
            let mut file = std::fs::File::open("test_output.pdf").unwrap();
            let mut buffer = [0; 50];
            let bytes_read = std::io::Read::read(&mut file, &mut buffer).unwrap();
            println!("First {} bytes: {:?}", bytes_read, &buffer[..bytes_read]);
        }
        
        // Don't clean up for now so we can test the PDF
        // std::fs::remove_file("test_output.pdf").unwrap();
    }
}