//! HTML generation for Spotify playlist cards using Askama templating
//! 
//! This module handles the generation of HTML documents containing
//! printable cards with QR codes for Spotify songs.

use crate::SongCard;
use crate::qr_generator;
use anyhow::Result;
use askama::Template;

/// Template context for card generation
#[derive(Template)]
#[template(path = "cards.html")]
pub struct CardsTemplate {
    /// Page title
    pub title: String,
    /// Total number of cards
    pub total_cards: usize,
    /// List of song cards with QR codes, chunked into pages of 12 cards each
    pub pages: Vec<Vec<CardContext>>,
}

/// Individual card context for template rendering
#[derive(Debug, Clone, serde::Serialize)]
pub struct CardContext {
    /// Song title
    pub title: String,
    /// Artist name
    pub artist: String,
    /// Release year
    pub year: String,
    /// Base64-encoded QR code data URL
    pub qr_data_url: String,
}

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

    /// Build HTML content from song cards using Askama template
    /// 
    /// Creates a complete HTML document string with printable cards
    /// using the Askama template engine.
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
        let card_contexts = self.create_card_contexts(cards)?;
        let context = self.create_template_context(title.to_string(), card_contexts);
        
        let html = context.render()?;
        Ok(html)
    }

    /// Create template context from song cards
    /// 
    /// # Arguments
    /// 
    /// * `cards` - Vector of song cards
    /// 
    /// # Returns
    /// 
    /// Vector of card contexts ready for template rendering
    fn create_card_contexts(&self, cards: Vec<SongCard>) -> Result<Vec<CardContext>> {
        cards
            .into_iter()
            .map(|card| {
                let qr_data_url = match qr_generator::generate_qr_data_url(&card.spotify_url) {
                    Ok(url) => url,
                    Err(_) => String::new(), // Fallback to empty string
                };

                Ok(CardContext {
                    title: card.title,
                    artist: card.artist,
                    year: card.year,
                    qr_data_url,
                })
            })
            .collect()
    }

    /// Create the main template context
    /// 
    /// # Arguments
    /// 
    /// * `title` - Page title
    /// * `cards` - Vector of card contexts
    /// 
    /// # Returns
    /// 
    /// Complete template context
    fn create_template_context(&self, title: String, cards: Vec<CardContext>) -> CardsTemplate {
        let total_cards = cards.len();
        let pages = cards.chunks(12).map(|chunk| chunk.to_vec()).collect();
        CardsTemplate { title, total_cards, pages }
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create HTML generator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_html_generator() {
        let generator = HtmlGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_create_card_contexts() {
        let generator = HtmlGenerator::new().unwrap();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://example.com".to_string(),
            },
        ];
        
        let contexts = generator.create_card_contexts(cards).unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].title, "Test Song");
        assert_eq!(contexts[0].artist, "Test Artist");
        assert_eq!(contexts[0].year, "2023");
    }

    #[test]
    fn test_create_template_context() {
        let generator = HtmlGenerator::new().unwrap();
        let cards = vec![CardContext {
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            year: "2023".to_string(),
            qr_data_url: "data:image/png;base64,test".to_string(),
        }];
        
        let context = generator.create_template_context("Test Playlist".to_string(), cards);
        assert_eq!(context.title, "Test Playlist");
        assert_eq!(context.total_cards, 1);
        assert_eq!(context.pages.len(), 1);
        assert_eq!(context.pages[0].len(), 1);
    }

    #[tokio::test]
    async fn test_build_html_content() {
        let generator = HtmlGenerator::new().unwrap();
        let cards = vec![
            SongCard {
                title: "Test Song".to_string(),
                artist: "Test Artist".to_string(),
                year: "2023".to_string(),
                spotify_url: "https://example.com".to_string(),
            },
        ];
        
        let html = generator.build_html_content(cards, "Test Playlist").unwrap();
        assert!(html.contains("Test Playlist"));
        assert!(html.contains("Test Song"));
        assert!(html.contains("Test Artist"));
        assert!(html.contains("2023"));
        assert!(html.contains("1 song"));
    }
}