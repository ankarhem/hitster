//! Card component template
//! 
//! This module contains the CardTemplate enum for rendering individual cards.

/// Template context for individual card (front or back)
#[derive(askama::Template, Debug)]
#[template(path = "components/card.html")]
pub enum CardTemplate {
    /// Front side of the card (QR code only)
    #[template(block = "front")]
    Front { qr_data_url: String },
    /// Back side of the card (song information)
    #[template(block = "back")]
    Back { title: String, artist: String, year: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_template_variants() {
        let front = CardTemplate::Front { qr_data_url: "test_qr".to_string() };
        let back = CardTemplate::Back { 
            title: "Test".to_string(), 
            artist: "Artist".to_string(), 
            year: "2023".to_string() 
        };
        
        match front {
            CardTemplate::Front { qr_data_url } => {
                assert_eq!(qr_data_url, "test_qr");
            }
            CardTemplate::Back { .. } => panic!("Expected front card"),
        }
        
        match back {
            CardTemplate::Back { title, artist, year } => {
                assert_eq!(title, "Test");
                assert_eq!(artist, "Artist");
                assert_eq!(year, "2023");
            }
            CardTemplate::Front { .. } => panic!("Expected back card"),
        }
    }
}