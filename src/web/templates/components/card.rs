//! Card component template
//! 
//! This module contains the CardTemplate enum for rendering individual cards.

/// Template context for individual card (front or back)
#[derive(askama::Template, Debug)]
#[template(path = "components/card.html")]
pub struct CardTemplate { 
    pub title: String,
    pub artist: String,
    pub year: String,
    pub qr_code: String,
}