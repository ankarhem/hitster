use crate::CardTemplate;

/// Template context for the cards page
#[derive(askama::Template, Debug)]
#[template(path = "cards.html")]
pub struct CardsTemplate {
    /// Page title
    pub title: String,
    /// Total number of cards
    pub total_cards: usize,
    /// Cards to render (front and back sides)
    pub cards: Vec<CardTemplate>,
}