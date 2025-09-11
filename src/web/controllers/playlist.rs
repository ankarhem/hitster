use crate::application::playlist_service::IPlaylistService;
use crate::web::error::ApiError;
use crate::web::server::Services;
use axum::{
    Form,
    extract::{Path, State},
    response::{Json, Redirect},
};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_PLAYLIST_ID_LENGTH: usize = 200;

pub struct PlaylistController {}

#[derive(Deserialize)]
pub struct CreatePlaylistForm {
    #[serde(rename = "id")]
    playlist_id: String,
}

#[derive(Serialize)]
pub struct JobResponse {
    job_id: Uuid,
}

pub async fn create_playlist<PlaylistService>(
    State(server): State<Services<PlaylistService>>,
    Form(form): Form<CreatePlaylistForm>,
) -> Result<Redirect, ApiError>
where
    PlaylistService: IPlaylistService,
{
    // Input validation
    if form.playlist_id.len() > MAX_PLAYLIST_ID_LENGTH {
        return Err(ApiError::ValidationError("Playlist ID too long".to_string()));
    }
    
    if form.playlist_id.trim().is_empty() {
        return Err(ApiError::ValidationError("Playlist ID cannot be empty".to_string()));
    }

    let spotify_id = form.playlist_id.parse()?;

    let playlist = server
        .playlist_service
        .create_from_spotify(&spotify_id)
        .await?;
    
    if let Some(playlist) = playlist {
        Ok(Redirect::to(&format!("/playlist/{}", playlist.id)))
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn refetch_playlist<PlaylistService>(
    State(services): State<Services<PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<()>, ApiError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    services
        .playlist_service
        .refetch_playlist(&playlist_id)
        .await?;
    Ok(Json(()))
}

pub async fn generate_pdfs<PlaylistService>(
    State(services): State<Services<PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<JobResponse>, ApiError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    let job = services
        .playlist_service
        .generate_playlist_pdfs(&playlist_id)
        .await?;

    Ok(Json(JobResponse {
        job_id: job.id.into(),
    }))
}

#[derive(Serialize)]
pub struct PdfResponse {
    front: String,
    back: String,
}

pub async fn get_pdfs<PlaylistService>(
    State(server): State<Services<PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<PdfResponse>, ApiError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;

    let pdfs = server
        .playlist_service
        .get_playlist_pdfs(&playlist_id)
        .await?;

    Ok(Json(PdfResponse {
        front: format!(
            "data:application/pdf;base64,{}",
            BASE64_STANDARD.encode(&pdfs[0])
        ),
        back: format!(
            "data:application/pdf;base64,{}",
            BASE64_STANDARD.encode(&pdfs[1])
        ),
    }))
}
