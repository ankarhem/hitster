use crate::application::job_service::IJobService;
use crate::application::playlist_service::IPlaylistService;
use crate::domain::{JobType, PlaylistId};
use crate::web::AppError;
use crate::web::server::Services;
use crate::web::templates::{CardTemplate, IndexTemplate, PlaylistTemplate};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};
use std::str::FromStr;

pub async fn index() -> Result<Html<String>, AppError> {
    let template = IndexTemplate {
        title: "Welcome to Playlist Card Generator".to_string(),
    };
    Ok(Html(template.render()?))
}

pub async fn view_playlist<JobsService, PlaylistService>(
    State(server): State<Services<JobsService, PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Html<String>, AppError>
where
    JobsService: IJobService,
    PlaylistService: IPlaylistService,
{
    let playlist_id = PlaylistId::from_str(&playlist_id)?;
    let playlist = match server.playlist_service.get_playlist(&playlist_id).await? {
        None => todo!(),
        Some(p) => p,
    };

    let job = server
        .job_service
        .create(&JobType::GeneratePlaylistPdf {
            id: playlist_id.clone(),
        })
        .await?;

    let cards: Vec<CardTemplate> = playlist
        .tracks
        .iter()
        .map(|track| CardTemplate {
            title: track.title.clone(),
            artist: track.artist.clone(),
            year: track.year.clone(),
            qr_code: format!("QR code for {}", track.title), // Placeholder
        })
        .collect();

    let template = PlaylistTemplate {
        title: playlist.name.clone(),
        total_cards: playlist.tracks.len(),
        cards,
        job_id: job.id.to_string(),
        playlist_id: playlist_id.to_string(),
        has_completed_job: false,
    };

    Ok(Html(template.render()?))
}
