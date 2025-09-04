use axum::{
    extract::{Path, State},
    response::Html,
};
use crate::web::{WebServer, AppError};
use crate::application::models::{JobType, PlaylistId};
use crate::web::templates::{CardsTemplate, CardTemplate};
use askama::Template;
use std::str::FromStr;

/// Controller for view-related endpoints
pub struct ViewController {}

impl ViewController {
    /// GET / -> index page where you can enter the playlist
    pub async fn index() -> Html<String> {
        Html("<h1>Enter Playlist URL</h1><form method=\"post\" action=\"/api/playlist\"><input type=\"text\" name=\"id\" placeholder=\"Spotify Playlist URL\"><button type=\"submit\">Submit</button></form>".to_string())
    }

    /// GET /playlist/<PlaylistId> -> Show the page with the playlist contents etc.
    pub async fn playlist(
        State(server): State<WebServer>,
        Path(playlist_id): Path<String>,
    ) -> Result<Html<String>, AppError> {
        let playlist_id = PlaylistId::from_str(&playlist_id)?;
        let playlist = server.playlist_service.get_playlist(&playlist_id).await?;
        
        let job_id = server.job_service.create_job(JobType::GeneratePlaylistPdf {
            id: playlist_id.clone(),
        }).await?;
        
        let cards: Vec<CardTemplate> = playlist.tracks.iter().map(|track| CardTemplate {
            title: track.title.clone(),
            artist: track.artist.clone(),
            year: track.year.clone(),
            qr_code: format!("QR code for {}", track.title), // Placeholder
        }).collect();
        
        let template = CardsTemplate {
            title: playlist.name.clone(),
            total_cards: playlist.tracks.len(),
            cards,
            job_id: job_id.to_string(),
            playlist_id: playlist_id.as_str().to_string(),
            has_completed_job: false,
        };
        
        Ok(Html(template.render()?))
    }
}