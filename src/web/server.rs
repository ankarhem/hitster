use axum::{
    extract::{Path, State, Form},
    response::{Html, IntoResponse, Redirect, Json},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use crate::application::{HitsterService, JobService};
use crate::infrastructure::Database;
use crate::web::{templates::{CardsTemplate, CardTemplate, IndexTemplate}, qr_code, AppError};
use askama::Template;
use tracing::{info, error, instrument};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct WebServer {
    hitster_service: HitsterService,
    database: Database,
    job_service: JobService,
}

#[derive(Deserialize, Debug)]
pub struct PlaylistForm {
    playlist_url: String,
}

#[derive(Serialize)]
struct JobResponse {
    job_id: i64,
}

#[derive(Serialize)]
struct JobStatusResponse {
    id: i64,
    status: String,
    front_pdf_path: Option<String>,
    back_pdf_path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerateRequest {
    playlist_id: i64,
}

impl WebServer {
    #[instrument(skip(hitster_service, database, job_service))]
    pub fn new(hitster_service: HitsterService, database: Database, job_service: JobService) -> Self {
        Self { hitster_service, database, job_service }
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
    
    // Extract playlist ID from URL
    let playlist_id = match extract_playlist_id(&form.playlist_url) {
        Ok(id) => {
            info!("Extracted playlist ID: {}", id);
            id
        }
        Err(e) => {
            error!("Failed to extract playlist ID from URL {}: {}", form.playlist_url, e);
            return Err(e);
        }
    };
    
    // Check if playlist already exists in database
    let existing_playlist = match server.database.get_playlist_by_spotify_id(&playlist_id).await {
        Ok(playlist) => {
            info!("Found existing playlist in database: {:?}", playlist);
            playlist
        }
        Err(e) => {
            error!("Failed to check existing playlist: {}", e);
            return Err(AppError::DatabaseError(e.to_string()));
        }
    };
    
    let playlist_id_num = if let Some(playlist) = existing_playlist {
        info!("Using existing playlist with ID: {}", playlist.id);
        playlist.id
    } else {
        info!("Fetching new playlist from Spotify API: {}", playlist_id);
        
        // Get playlist info from Spotify
        let spotify_playlist = match server.hitster_service.get_playlist_by_id(&playlist_id).await {
            Ok(playlist) => {
                info!("Successfully fetched playlist from Spotify: {} with {} tracks", playlist.name, playlist.tracks.len());
                playlist
            }
            Err(e) => {
                error!("Failed to fetch playlist from Spotify API: {}", e);
                return Err(AppError::SpotifyApiError(e.to_string()));
            }
        };
        
        // Create playlist in database
        let db_playlist = match server.database.create_playlist(&playlist_id, &spotify_playlist.name).await {
            Ok(playlist) => {
                info!("Created playlist in database with ID: {}", playlist.id);
                playlist
            }
            Err(e) => {
                error!("Failed to create playlist in database: {}", e);
                return Err(AppError::DatabaseError(e.to_string()));
            }
        };
        
        // Store tracks in database
        let tracks: Vec<_> = spotify_playlist.tracks.into_iter().enumerate().map(|(i, track)| {
            crate::infrastructure::NewTrack {
                playlist_id: db_playlist.id,
                title: track.title,
                artist: track.artist,
                year: track.year,
                spotify_url: track.spotify_url,
                position: i as i32,
            }
        }).collect();
        
        info!("Storing {} tracks in database", tracks.len());
        match server.database.create_tracks(&tracks).await {
            Ok(()) => {
                info!("Successfully stored tracks in database");
            }
            Err(e) => {
                error!("Failed to store tracks in database: {}", e);
                return Err(AppError::DatabaseError(e.to_string()));
            }
        };
        
        db_playlist.id
    };
    
    info!("Redirecting to cards page for playlist ID: {}", playlist_id_num);
    Ok(Redirect::to(&format!("/cards/{}", playlist_id_num)))
}

#[instrument(skip(server), fields(playlist_id))]
async fn cards_page(
    Path(playlist_id): Path<i64>,
    State(server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    let playlist = server.database.get_playlist_by_id(playlist_id).await?
        .ok_or_else(|| AppError::Anything(anyhow::anyhow!("Playlist not found")))?;
    
    let tracks = server.database.get_tracks_by_playlist_id(playlist_id).await?;
    
    // Check if there's already a job, create one if needed
    let job = match server.database.get_latest_job_for_playlist(playlist_id).await? {
        Some(existing_job) => existing_job,
        None => {
            // Create a pending job
            server.database.create_job(playlist_id).await?
        }
    };
    
    let html = build_cards_html_content(playlist, tracks, Some(job))?;
    
    info!("Served cards page for playlist: {}", playlist_id);
    Ok(Html(html))
}

fn build_cards_html_content(
    playlist: crate::infrastructure::Playlist,
    tracks: Vec<crate::infrastructure::Track>,
    job: Option<crate::infrastructure::Job>,
) -> Result<String, AppError> {
    let total_cards = tracks.len();
    let card_templates = create_card_templates_from_db(tracks)?;
    
    let (job_id, playlist_id, has_completed_job) = if let Some(ref job) = job {
        (
            job.id,
            job.playlist_id,
            job.status == crate::infrastructure::JobStatus::Completed,
        )
    } else {
        (0, 0, false)
    };
    
    let template = CardsTemplate {
        title: playlist.name,
        total_cards,
        cards: card_templates,
        job,
        job_id,
        playlist_id,
        has_completed_job,
    };
    
    let html = template.render().map_err(|e| AppError::Anything(anyhow::anyhow!(e)))?;
    Ok(html)
}

fn create_card_templates_from_db(tracks: Vec<crate::infrastructure::Track>) -> Result<Vec<CardTemplate>, AppError> {
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

pub fn extract_playlist_id(url: &str) -> Result<String, AppError> {
    // Handle different URL formats:
    // https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M
    // https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=xyz
    // spotify:playlist:37i9dQZF1DXcBWIGoYBM5M
    // 37i9dQZF1DXcBWIGoYBM5M
    
    if url.contains("open.spotify.com/playlist/") {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(last_part) = parts.last() {
            // Remove query parameters if present
            let clean_id = last_part.split('?').next().unwrap_or(last_part);
            if !clean_id.is_empty() {
                return Ok(clean_id.to_string());
            }
        }
    } else if url.contains("spotify:playlist:") {
        let parts: Vec<&str> = url.split(':').collect();
        if let Some(id) = parts.last() {
            // Remove query parameters if present
            let clean_id = id.split('?').next().unwrap_or(id);
            if !clean_id.is_empty() {
                return Ok(clean_id.to_string());
            }
        }
    } else if !url.contains('/') && !url.contains(':') {
        // Assume it's a raw ID, but still clean query parameters
        let clean_id = url.split('?').next().unwrap_or(url);
        if !clean_id.is_empty() {
            return Ok(clean_id.to_string());
        }
    }
    
    Err(AppError::InvalidPlaylistUrl(url.to_string()))
}

#[instrument(skip(server))]
async fn start_generation(
    Path(job_id): Path<i64>,
    State(server): State<WebServer>,
    Json(request): Json<GenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Create a new job for this generation request
    let job = server.job_service.create_job(request.playlist_id).await?;
    
    info!("Started generation job {} for playlist {}", job.id, request.playlist_id);
    Ok(Json(JobResponse { job_id: job.id }))
}

#[instrument(skip(server))]
async fn get_job_status(
    Path(job_id): Path<i64>,
    State(server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    let job = server.job_service.get_job(job_id).await?
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
    Path((playlist_id, pdf_type)): Path<(i64, String)>,
    State(server): State<WebServer>,
) -> Result<impl IntoResponse, AppError> {
    // Get the latest completed job for this playlist
    let job = server.database.get_latest_job_for_playlist(playlist_id).await?
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