use axum::{
    extract::{Path, State},
    response::{Redirect, Json},
    Form,
};
use crate::web::{AppError};
use serde::{Deserialize, Serialize};
use crate::application::job_service::IJobService;
use crate::application::playlist_service::IPlaylistService;
use crate::domain::{PdfSide};
use crate::web::server::Services;

pub struct PlaylistController {}

#[derive(Deserialize)]
pub struct CreatePlaylistForm {
    id: String,
}

#[derive(Serialize)]
pub struct JobResponse {
    job_id: String,
}

pub async fn create_playlist<JobsService, PlaylistService>(
    State(server): State<Services<JobsService, PlaylistService>>,
    Form(form): Form<CreatePlaylistForm>,
) -> Result<Redirect, AppError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = form.id.parse()?;

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
pub async fn refetch_playlist<JobsService, PlaylistService>(
    State(services): State<Services<JobsService, PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<()>, AppError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    services.playlist_service.refetch_playlist(&playlist_id).await?;
    Ok(Json(()))
}

/// POST /api/playlist/<PlaylistId>/pdfs -> Respond with job id
pub async fn generate_pdfs<JobsService, PlaylistService>(
    State(services): State<Services<JobsService, PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<JobResponse>, AppError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    let job_id = services.playlist_service.generate_playlist_pdfs(&playlist_id).await?;

    Ok(Json(JobResponse {
        job_id: job_id.to_string(),
    }))
}

/// GET /api/playlist/<PlaylistId>/pdfs/{front|back} -> Returns the corresponding pdfs
pub async fn get_pdf<JobsService, PlaylistService>(
    State(server): State<Services<JobsService, PlaylistService>>,
    Path((playlist_id, side)): Path<(String, String)>,
) -> Result<Vec<u8>, AppError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    let side = match side.as_str() {
        "front" => PdfSide::Front,
        "back" => PdfSide::Back,
        _ => return Err(AppError::ValidationError("Invalid side".to_string())),
    };

    let pdf = server.playlist_service.get_playlist_pdf(&playlist_id, side).await?;
    Ok(pdf.into_bytes())
}