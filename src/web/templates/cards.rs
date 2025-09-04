use crate::CardTemplate;
use crate::infrastructure::Job;

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
    /// Job information for PDF generation
    pub job: Option<Job>,
    /// Helper fields for template
    pub job_id: String,
    pub playlist_id: String,
    pub has_completed_job: bool,
}