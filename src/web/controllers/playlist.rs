use crate::PlaylistTemplate;
use crate::application::playlist_service::IPlaylistService;
use crate::domain::spotify_id::SpotifyId;
use crate::domain::{JobId, JobStatus, PlaylistId};
use crate::web::error::ApiError;
use crate::web::extensions::HtmxExtension;
use crate::web::server::Services;
use anyhow::anyhow;
use askama::Template;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Response, Sse};
use axum::{
    Form,
    extract::{Path, State},
    response::{Html, Json, Redirect},
};
use futures_util::{self, Stream};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_stream::StreamExt;
use uuid::Uuid;

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
    headers: HeaderMap,
    State(services): State<Services<PlaylistService>>,
    Form(form): Form<CreatePlaylistForm>,
) -> Result<impl IntoResponse, ApiError>
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
    let spotify_id = SpotifyId::parse(input).map_err(|e| {
        ApiError::ValidationError(format!("Invalid Spotify playlist format: {}", e))
    })?;

    if headers.is_htmx_request() {
        let (playlist, job) = services
            .playlist_service
            .create_partial_playlist_from_spotify(&spotify_id)
            .await?;
        return match (playlist, job) {
            (Some(playlist), None) => {
                let location = format!("/playlist/{}", &playlist.id);
                return Ok(Redirect::to(&location).into_response());
            }
            (Some(playlist), Some(job)) => {
                let location = format!("/playlist/{}", &playlist.id);

                let template = PlaylistTemplate {
                    title: playlist.name.clone(),
                    total_tracks: playlist.tracks.len(),
                    tracks: vec![],
                    playlist_id: playlist.id.to_string(),
                    latest_job: Some(job.into()),
                    has_generated_pdfs: false,
                };
                let mut headers = HeaderMap::new();
                headers.insert("HX-Replace-Url", HeaderValue::from_str(&location).unwrap());

                let html = template
                    .render()
                    .map_err(|_| anyhow!("Failed to render playlist template"))?;
                Ok((headers, Html(html)).into_response())
            }
            (None, _) => Err(ApiError::NotFound)
        };
    }

    let playlist = services
        .playlist_service
        .create_from_spotify(&spotify_id)
        .await?;

    if let Some(playlist) = playlist {
        let location = format!("/playlist/{}", playlist.id);
        Ok(Redirect::to(&location).into_response())
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn refetch_playlist<PlaylistService>(
    State(services): State<Services<PlaylistService>>,
    Path(playlist_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id: PlaylistId = playlist_id.parse()?;
    let job = services
        .playlist_service
        .refetch_playlist(&playlist_id)
        .await?;

    // If the request is from HTMX reload the current page
    if headers.is_htmx_request() {
        let redirect_to = format!("/playlist/{}", playlist_id);
        let mut headers = HeaderMap::new();
        headers.insert("HX-Redirect", HeaderValue::from_str(&redirect_to).unwrap());
        return Ok((headers, axum::body::Body::empty()).into_response());
    }

    Ok(Json(JobResponse {
        job_id: job.id.into(),
    })
    .into_response())
}

pub async fn generate_pdfs<PlaylistService>(
    State(services): State<Services<PlaylistService>>,
    Path(playlist_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id: PlaylistId = playlist_id.parse()?;
    let job = services
        .playlist_service
        .generate_playlist_pdfs(&playlist_id)
        .await?;

    // If the request is from HTMX reload the current page
    if headers.is_htmx_request() {
        let redirect_to = format!("/playlist/{}", playlist_id);
        let mut headers = HeaderMap::new();
        headers.insert("HX-Redirect", HeaderValue::from_str(&redirect_to).unwrap());
        return Ok((headers, axum::body::Body::empty()).into_response());
    }

    Ok(Json(JobResponse {
        job_id: job.id.into(),
    })
    .into_response())
}

pub async fn download_pdf<PlaylistService>(
    State(services): State<Services<PlaylistService>>,
    Path((playlist_id, pdf_side)): Path<(String, String)>,
) -> Result<Response, ApiError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id: crate::domain::PlaylistId = playlist_id.parse()?;

    // Validate PDF type
    if pdf_side != "front" && pdf_side != "back" {
        return Err(ApiError::ValidationError(
            "Invalid PDF type. Must be 'front' or 'back'".to_string(),
        ));
    }

    // Get the PDFs from the service
    let pdfs = services
        .playlist_service
        .get_playlist_pdfs(&playlist_id)
        .await?;

    let pdf_data = match pdf_side.as_str() {
        "front" => pdfs[0].clone(),
        "back" => pdfs[1].clone(),
        _ => unreachable!(), // We already validated above
    };

    let filename = format!("hitster_cards_{}_{}.pdf", playlist_id, pdf_side);

    Ok((
        [
            (CONTENT_TYPE, HeaderValue::from_static("application/pdf")),
            (
                CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
            ),
        ],
        Vec::<u8>::from(pdf_data), // Convert Pdf to Vec<u8>
    )
        .into_response())
}

pub async fn get_job_status<PlaylistService>(
    State(services): State<Services<PlaylistService>>,
    Path((playlist_id, job_id)): Path<(String, String)>,
) -> Sse<impl Stream<Item = Result<Event, ApiError>>>
where
    PlaylistService: IPlaylistService + Send + Sync + 'static,
{
    let _playlist_id: PlaylistId = playlist_id.parse().unwrap();
    let job_id: JobId = job_id.parse().unwrap();

    let stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        Duration::from_millis(200),
    ))
    .then(move |_| {
        let job_id = job_id.clone();
        let playlist_service = services.playlist_service.clone();
        async move {
            let job = playlist_service.get_job_by_id(&job_id).await?;

            match job {
                Some(ref j) if j.status == JobStatus::Completed => {
                    Ok(Event::default().event("done").data("completed"))
                }
                Some(ref j) => Ok(Event::default().event("status").data(j.status.to_string())),
                None => Err(ApiError::NotFound),
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
