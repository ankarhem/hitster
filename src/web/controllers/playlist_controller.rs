use axum::{
    extract::{Path, State},
    response::{Redirect, Json},
    Form,
};
use crate::web::{WebServer, AppError};
use serde::{Deserialize, Serialize};
use crate::application::models::{PlaylistId, PdfSide};
use std::str::FromStr;

/// Controller for playlist API endpoints
pub struct PlaylistController {}

#[derive(Deserialize)]
pub struct CreatePlaylistForm {
    id: String,
}

#[derive(Serialize)]
pub struct JobResponse {
    job_id: String,
}

impl PlaylistController {
    /// POST /api/playlist with form data -> Creates playlist and redirects to view page
    pub async fn create_playlist(
        State(server): State<WebServer>,
        Form(form): Form<CreatePlaylistForm>,
    ) -> Result<Redirect, AppError> {
        let playlist_id = PlaylistId::from_str(&form.id)?;
        
        // Check if playlist exists, if not create it
        match server.playlist_service.get_playlist(&playlist_id).await {
            Ok(_) => {},
            Err(_) => {
                // TODO: Create playlist from Spotify URL
            }
        }
        
        Ok(Redirect::to(&format!("/playlist/{}", playlist_id)))
    }

    /// POST /api/playlist/<PlaylistId>/refetch-playlist -> Update playlist and tracks in db
    pub async fn refetch_playlist(
        State(server): State<WebServer>,
        Path(playlist_id): Path<String>,
    ) -> Result<Json<()>, AppError> {
        let playlist_id = PlaylistId::from_str(&playlist_id)?;
        server.playlist_service.refetch_playlist(&playlist_id).await?;
        Ok(Json(()))
    }

    /// POST /api/playlist/<PlaylistId>/pdfs -> Respond with job id
    pub async fn generate_pdfs(
        State(server): State<WebServer>,
        Path(playlist_id): Path<String>,
    ) -> Result<Json<JobResponse>, AppError> {
        let playlist_id = PlaylistId::from_str(&playlist_id)?;
        let job_id = server.playlist_service.generate_playlist_pdfs(&playlist_id).await?;
        
        Ok(Json(JobResponse {
            job_id: job_id.to_string(),
        }))
    }

    /// GET /api/playlist/<PlaylistId>/pdfs/{front|back} -> Returns the corresponding pdfs
    pub async fn get_pdf(
        State(server): State<WebServer>,
        Path((playlist_id, side)): Path<(String, String)>,
    ) -> Result<Vec<u8>, AppError> {
        let playlist_id = PlaylistId::from_str(&playlist_id)?;
        let side = match side.as_str() {
            "front" => PdfSide::Front,
            "back" => PdfSide::Back,
            _ => return Err(AppError::ValidationError("Invalid side".to_string())),
        };
        
        let pdf = server.playlist_service.get_playlist_pdf(&playlist_id, side).await?;
        Ok(pdf.into_bytes())
    }
}