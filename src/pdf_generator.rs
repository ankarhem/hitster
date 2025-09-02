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
        use lopdf::{Dictionary, Object};
        
        // Create a simple PDF document
        let mut doc = lopdf::Document::with_version("1.5");
        
        // Create pages root
        let pages_id = doc.add_object(Dictionary::new());
        
        // Create pages (2x5 grid on A4)
        let cards_per_row = 2;
        let cards_per_col = 5;
        let cards_per_page = cards_per_row * cards_per_col;
        
        let pages = (cards.len() + cards_per_page - 1) / cards_per_page;
        
        for page_num in 0..pages {
            // Create A4 page (595 x 842 points)
            let (page_id, content_id) = self.create_page(&mut doc, pages_id, page_num)?;
            
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, cards.len());
            
            for card_idx in start_idx..end_idx {
                let card_index_on_page = card_idx - start_idx;
                let row = card_index_on_page / cards_per_row;
                let col = card_index_on_page % cards_per_row;
                
                let x = self.margin + col as f64 * (self.card_width + self.margin);
                let y = 297.0 - self.margin - (row + 1) as f64 * (self.card_height + self.margin);
                
                self.add_card_to_page(&mut doc, page_id, content_id, &cards[card_idx], title, x, y)?;
            }
        }
        
        // Set up the document catalog
        let catalog_id = doc.add_object(Dictionary::from_iter([
            ("Type", "Catalog".into()),
            ("Pages", pages_id.into()),
        ]));
        
        // Set up pages dictionary
        let mut pages_dict = Dictionary::new();
        pages_dict.set("Type", "Pages");
        pages_dict.set("Kids", lopdf::Object::Array(Vec::<lopdf::Object>::new())); // Will be populated with page IDs
        pages_dict.set("Count", pages as i32);
        doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
        
        // Set the document trailer
        doc.trailer.set("Root", catalog_id);
        
        // Save the document
        doc.save(output_path)?;
        Ok(())
    }
    
    fn create_page(&self, doc: &mut lopdf::Document, pages_id: lopdf::ObjectId, _page_num: usize) -> Result<(lopdf::ObjectId, lopdf::ObjectId)> {
        use lopdf::Dictionary;
        
        // Create content stream for the page
        let content_id = doc.add_object(Dictionary::new());
        
        // Create page dictionary
        let mut page_dict = Dictionary::new();
        page_dict.set("Type", "Page");
        page_dict.set("Parent", pages_id);
        page_dict.set("Contents", content_id);
        page_dict.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]); // A4 size
        page_dict.set("Resources", Dictionary::new());
        
        // Add page to document
        let page_id = doc.add_object(page_dict);
        
        Ok((page_id, content_id))
    }
    
    fn add_card_to_page(&self, doc: &mut lopdf::Document, _page_id: lopdf::ObjectId, content_id: lopdf::ObjectId, card: &SongCard, title: &str, x: f64, y: f64) -> Result<()> {
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
        
        // Convert operations to bytes and add to page content
        use lopdf::content::Content;
        let content = Content {
            operations: operations.clone(),
        };
        doc.add_page_contents(content_id, content.encode().unwrap())?;
        Ok(())
    }
    
    fn mm_to_points(&self, x: f64, y: f64) -> (f64, f64) {
        // Convert mm to points (1 mm = 2.83465 points)
        (x * 2.83465, y * 2.83465)
    }
    
    fn create_text_operation(&self, text: &str, x: f64, y: f64, font_size: f64) -> Vec<lopdf::content::Operation> {
        use lopdf::content::*;
        
        let (x_pt, y_pt) = self.mm_to_points(x, y);
        
        vec![
            Operation::new("BT", vec![]), // Begin text
            Operation::new("Tf", vec!["F1".into(), font_size.into()]), // Font and size
            Operation::new("Td", vec![x_pt.into(), y_pt.into()]), // Position
            Operation::new("Tj", vec![text.into()]), // Text
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
            SongCard {
                title: "Another Song".to_string(),
                artist: "Another Artist".to_string(),
                year: "2022".to_string(),
                spotify_url: "another_id".to_string(),
            },
        ];
        
        let result = generator.generate_pdf(cards, "Test Playlist", "test_output.pdf");
        assert!(result.is_ok());
        
        // Check if the file was created
        assert!(std::path::Path::new("test_output.pdf").exists());
        
        // Clean up
        std::fs::remove_file("test_output.pdf").unwrap();
    }
}