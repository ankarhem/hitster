use axum::{
    extract::{Path, State},
    response::{Redirect, Json},
    Form,
};
use serde::{Deserialize, Serialize};
use crate::application::job_service::IJobService;
use crate::application::playlist_service::IPlaylistService;
use crate::domain::{PdfSide};
use crate::web::error::ApiError;
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
) -> Result<Redirect, ApiError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = form.id.parse()?;

    match server.playlist_service.get_playlist(&playlist_id).await {
        Ok(_) => {},
        Err(_) => {
            todo!("create ")
        }
    }

    Ok(Redirect::to(&format!("/playlist/{}", playlist_id)))
}

pub async fn refetch_playlist<JobsService, PlaylistService>(
    State(services): State<Services<JobsService, PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<()>, ApiError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    services.playlist_service.refetch_playlist(&playlist_id).await?;
    Ok(Json(()))
}

pub async fn generate_pdfs<JobsService, PlaylistService>(
    State(services): State<Services<JobsService, PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Json<JobResponse>, ApiError>
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

pub async fn get_pdf<JobsService, PlaylistService>(
    State(server): State<Services<JobsService, PlaylistService>>,
    Path((playlist_id, side)): Path<(String, String)>,
) -> Result<Vec<u8>, ApiError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = playlist_id.parse()?;
    let side = match side.as_str() {
        "front" => PdfSide::Front,
        "back" => PdfSide::Back,
        _ => {
            todo!("fix when ApiError is not a placeholder")
        }
    };

    let pdf = server.playlist_service.get_playlist_pdf(&playlist_id, side).await?;
    Ok(pdf.into_bytes())
}