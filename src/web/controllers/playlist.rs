use crate::application::playlist_service::IPlaylistService;
use crate::domain::spotify_id::SpotifyId;
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
use crate::domain::PlaylistId;

const MAX_PLAYLIST_ID_LENGTH: usize = 200;
const MIN_PLAYLIST_ID_LENGTH: usize = 16; // Spotify IDs are typically 22 characters

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
    let input = form.playlist_id.trim();
    if input.len() > MAX_PLAYLIST_ID_LENGTH {
        return Err(ApiError::ValidationError(
            "Spotify URL/ID is too long (maximum 100 characters)".to_string(),
        ));
    }

    if input.len() < MIN_PLAYLIST_ID_LENGTH {
        return Err(ApiError::ValidationError(
            "Spotify URL/ID is too short".to_string(),
        ));
    }

    // Parse the Spotify ID (this will do additional format validation)
    let spotify_id = SpotifyId::parse(&input)
        .map_err(|e| ApiError::ValidationError(format!("Invalid Spotify playlist format: {}", e)))?;

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
    let playlist_id: PlaylistId = playlist_id.parse()?;
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
    let playlist_id: PlaylistId = playlist_id.parse()?;
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
    let playlist_id: PlaylistId = playlist_id.parse()?;

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