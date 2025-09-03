//! HTML generation for Spotify playlist cards using Askama templating
//! 
//! This module handles the generation of HTML documents containing
//! printable cards with QR codes for Spotify songs.

use crate::application::models::SongCard;
use crate::templates::{CardsTemplate, CardTemplate};
use crate::qr_generator;
use anyhow::Result;
use askama::Template;

/// HTML generator using Askama templates
/// 
/// This struct manages template rendering for generating
/// printable HTML cards from Spotify playlists.
/// 
/// No need to store template engine - Askama handles it at compile time.
#[derive(Clone)]
pub struct HtmlGenerator {}

impl HtmlGenerator {
    /// Create a new HTML generator with default settings
    /// 
    /// # Errors
    /// 
    /// Returns an error if template compilation fails
    pub fn new() -> Result<Self> {
        // Askama templates are compiled at build time, so no runtime setup needed
        Ok(Self {})
    }

    /// Build HTML content from song cards using enum templates
    /// 
    /// Creates a complete HTML document string with printable cards
    /// using the new enum-based template system.
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Vector of song cards to generate HTML for
    /// * `title` - Title for the HTML document
    /// 
    /// # Returns
    /// 
    /// A complete HTML document string
    /// 
    /// # Errors
    /// 
    /// Returns an error if template rendering fails
    pub fn build_html_content(&self, cards: Vec<SongCard>, title: &str) -> Result<String> {
        // Get the total number of cards before moving them
        let total_cards = cards.len();
        
        // Create card templates for each song (front and back)
        let card_templates = self.create_card_templates(cards)?;
        
        // Create the main template
        let template = CardsTemplate {
            title: title.to_string(),
            total_cards,
            cards: card_templates,
        };
        
        Ok(template.render()?)
    }

    /// Create card templates from song cards
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Vector of song cards
    /// 
    /// # Returns
    /// 
    /// Vector of card templates with front and back sides alternating
    fn create_card_templates(&self, cards: Vec<SongCard>) -> Result<Vec<CardTemplate>> {
        let mut all_cards = Vec::new();
        
        // First, create all front cards
        for card in &cards {
            let qr_data_url = qr_generator::generate_qr_data_url(&card.spotify_url)
                .unwrap_or_default();
            
            all_cards.push(CardTemplate::Front { qr_data_url });
        }
        
        // Then, create all back cards
        for card in cards {
            all_cards.push(CardTemplate::Back {
                title: html_escape::encode_text(&card.title).to_string(),
                artist: html_escape::encode_text(&card.artist).to_string(),
                year: card.year,
            });
        }
        
        Ok(all_cards)
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create HTML generator")
    }
}