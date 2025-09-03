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