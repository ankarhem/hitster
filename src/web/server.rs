use axum::{
    extract::{Path, State, Form},
    response::{Html, IntoResponse, Redirect, Json},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use crate::application::{HitsterService, JobService, PlaylistService};
use crate::web::{templates::{CardsTemplate, CardTemplate, IndexTemplate}, qr_code, AppError};
use askama::Template;
use tracing::{info, error, instrument};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct WebServer {
    hitster_service: HitsterService,
    job_service: JobService,
    playlist_service: PlaylistService,
}

#[derive(Deserialize, Debug)]
pub struct PlaylistForm {
    playlist_url: String,
}

#[derive(Serialize)]
struct JobResponse {
    job_id: String,
}

#[derive(Serialize)]
struct JobStatusResponse {
    id: String,
    status: String,
    front_pdf_path: Option<String>,
    back_pdf_path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerateRequest {
    playlist_id: String,
}

impl WebServer {
    #[instrument(skip(hitster_service, job_service, playlist_service))]
    pub fn new(hitster_service: HitsterService, job_service: JobService, playlist_service: PlaylistService) -> Self {
        Self { hitster_service, job_service, playlist_service }
    }

    #[instrument(skip(self), fields(port))]
    pub async fn run(&self, port: u16) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(index))
            .route("/submit-playlist", post(submit_playlist))
            .route("/cards/:playlist_id", get(cards_page))
            .route("/api/generate/:job_id", post(start_generation))
            .route("/api/jobs/:job_id", get(get_job_status))
            .route("/api/download/:playlist_id/:type", get(download_pdf))
            .with_state(self.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        info!("🚀 Web server running at http://localhost:{}", port);
        info!("📋 Endpoints:");
        info!("   GET /                         - Index page with form");
        info!("   POST /submit-playlist         - Submit playlist form");
        info!("   GET /cards/<id>               - Cards page with PDF generation");
        info!("   POST /api/generate/<job_id>   - Start PDF generation");
        info!("   GET /api/jobs/<job_id>        - Get job status");
        info!("   GET /api/download/<id>/<type> - Download PDF");
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[instrument(skip(_server))]
async fn index(
    State(_server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    let template = IndexTemplate {
        title: "Hitster - Generate Cards".to_string(),
    };
    
    let html = template.render().map_err(|e| AppError::Anything(anyhow::anyhow!(e)))?;
    Ok(Html(html))
}

#[instrument(skip(server))]
async fn submit_playlist(
    State(server): State<WebServer>,
    Form(form): Form<PlaylistForm>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received playlist submission: {}", form.playlist_url);
    
    // Use the PlaylistService to handle all business logic
    let playlist_id = match server.playlist_service.process_playlist_submission(&form.playlist_url).await {
        Ok(id) => {
            info!("Successfully processed playlist submission with ID: {}", id);
            id
        }
        Err(e) => {
            error!("Failed to process playlist submission: {}", e);
            // Check if it's a URL parsing error
            if e.to_string().contains("Invalid Spotify playlist") || 
               e.to_string().contains("Empty playlist ID") {
                return Err(AppError::InvalidPlaylistUrl(form.playlist_url.clone()));
            }
            return Err(AppError::Anything(e));
        }
    };
    
    info!("Redirecting to cards page for playlist ID: {}", playlist_id);
    Ok(Redirect::to(&format!("/cards/{}", playlist_id)))
}

#[instrument(skip(server), fields(playlist_id))]
async fn cards_page(
    Path(playlist_id): Path<String>,
    State(server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    let playlist = server.playlist_service.get_playlist_by_id(&playlist_id).await?
        .ok_or_else(|| AppError::Anything(anyhow::anyhow!("Playlist not found")))?;
    
    // Check if there's already a job, create one if needed
    let job = server.job_service.get_or_create_job_for_playlist(&playlist_id).await?;
    
    let html = build_cards_html_content(playlist, Some(job))?;
    
    info!("Served cards page for playlist: {}", playlist_id);
    Ok(Html(html))
}

fn build_cards_html_content(
    playlist: crate::application::models::Playlist,
    job: Option<crate::infrastructure::Job>,
) -> Result<String, AppError> {
    let total_cards = playlist.tracks.len();
    let card_templates = create_card_templates_from_domain(playlist.tracks.clone())?;
    
    let (job_id, playlist_id_str, has_completed_job) = if let Some(ref job) = job {
        (
            job.id.clone(),
            job.playlist_id.clone(),
            job.status == crate::infrastructure::JobStatus::Completed,
        )
    } else {
        ("".to_string(), "".to_string(), false)
    };
    
    let template = CardsTemplate {
        title: playlist.name,
        total_cards,
        cards: card_templates,
        job,
        job_id,
        playlist_id: playlist_id_str,
        has_completed_job,
    };
    
    let html = template.render().map_err(|e| AppError::Anything(anyhow::anyhow!(e)))?;
    Ok(html)
}

fn create_card_templates_from_domain(tracks: Vec<crate::application::models::Track>) -> Result<Vec<CardTemplate>, AppError> {
    let mut all_cards = Vec::new();
    
    for track in tracks {
        all_cards.push(CardTemplate {
            title: html_escape::encode_text(&track.title).to_string(),
            artist: html_escape::encode_text(&track.artist).to_string(),
            year: track.year,
            qr_code: qr_code::generate_qr_data_url(&track.spotify_url)?,
        });
    }
    
    Ok(all_cards)
}

#[instrument(skip(server))]
async fn start_generation(
    Path(job_id): Path<String>,
    State(server): State<WebServer>,
    Json(request): Json<GenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Create a new job for this generation request
    let job = server.job_service.create_job(&request.playlist_id).await?;
    
    info!("Started generation job {} for playlist {}", job.id, request.playlist_id);
    Ok(Json(JobResponse { job_id: job.id }))
}

#[instrument(skip(server))]
async fn get_job_status(
    Path(job_id): Path<String>,
    State(server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    let job = server.job_service.get_job(&job_id).await?
        .ok_or_else(|| AppError::Anything(anyhow::anyhow!("Job not found")))?;
    
    let response = JobStatusResponse {
        id: job.id,
        status: format!("{:?}", job.status),
        front_pdf_path: job.front_pdf_path,
        back_pdf_path: job.back_pdf_path,
    };
    
    Ok(Json(response))
}

#[instrument(skip(server))]
async fn download_pdf(
    Path((playlist_id, pdf_type)): Path<(String, String)>,
    State(server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    // Get the latest completed job for this playlist
    let job = server.job_service.get_latest_job_for_playlist(&playlist_id).await?
        .ok_or_else(|| AppError::Anything(anyhow::anyhow!("No job found for playlist")))?;
    
    if job.status != crate::infrastructure::JobStatus::Completed {
        return Err(AppError::Anything(anyhow::anyhow!("PDF generation not completed")));
    }
    
    let file_path = match pdf_type.as_str() {
        "front" => job.front_pdf_path.ok_or_else(|| AppError::Anything(anyhow::anyhow!("Front PDF not available")))?,
        "back" => job.back_pdf_path.ok_or_else(|| AppError::Anything(anyhow::anyhow!("Back PDF not available")))?,
        _ => return Err(AppError::Anything(anyhow::anyhow!("Invalid PDF type"))),
    };
    
    // Read the file and return it
    let file_content = tokio::fs::read(&file_path).await?;
    
    let filename = match pdf_type.as_str() {
        "front" => format!("playlist_{}_front.pdf", playlist_id),
        "back" => format!("playlist_{}_back.pdf", playlist_id),
        _ => unreachable!(),
    };
    
    info!("Downloading {} PDF for playlist {}", pdf_type, playlist_id);
    
    let mut response = file_content.into_response();
    response.headers_mut().insert("Content-Type", "application/pdf".parse().unwrap());
    response.headers_mut().insert("Content-Disposition", format!("attachment; filename=\"{}\"", filename).parse().unwrap());
    
    Ok(response)
}