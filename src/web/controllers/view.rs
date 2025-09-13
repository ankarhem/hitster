use crate::application::playlist_service::IPlaylistService;
use crate::web::error::TemplateError;
use crate::web::server::Services;
use crate::web::templates::playlist::{JobVM, TrackVM};
use crate::web::templates::{IndexTemplate, PlaylistTemplate};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};
use crate::application::worker::GeneratePlaylistPdfsResult;
use crate::domain::PlaylistId;

pub async fn index() -> Result<Html<String>, TemplateError> {
    let template = IndexTemplate {
        title: "Welcome to Playlist Card Generator".to_string(),
    };
    Ok(Html(template.render()?))
}

pub async fn view_playlist<PlaylistService>(
    State(server): State<Services<PlaylistService>>,
    Path(playlist_id): Path<String>,
) -> Result<Html<String>, TemplateError>
where
    PlaylistService: IPlaylistService,
{
    let playlist_id: PlaylistId = playlist_id.parse()?;
    let playlist = match server.playlist_service.get_playlist(&playlist_id).await? {
        None => Err(TemplateError::NotFound(format!("Playlist with id {} not found", playlist_id)))?,
        Some(p) => p,
    };

    let total_tracks = playlist.tracks.len();
    let tracks: Vec<TrackVM> = playlist
        .tracks
        .iter()
        .take(20)
        .map(|track| -> Result<TrackVM, TemplateError> {
            let code = qrcode::QrCode::new(&track.spotify_url)?;
            let svg = code
                .render::<qrcode::render::svg::Color>()
                .min_dimensions(0, 200)
                .max_dimensions(200, 200)
                .build();
            let svg = svg.replace(
                r#"crispEdges""#,
                r#"crispEdges" style="height: 100%; width: 100%""#,
            );

            Ok(TrackVM {
                title: track.title.clone(),
                artist: track.artist.clone(),
                year: track.year,
                qr_code: svg,
                album_cover_url: track.album_cover_url.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let latest_job = server.playlist_service.get_latest_job(&playlist_id).await?;
    let latest_job = latest_job.map(|job| JobVM {
        id: job.id.to_string(),
        is_in_progress: job.status != crate::domain::JobStatus::Completed,
    });

    let has_pdfs = server.playlist_service.get_playlist_pdfs(&playlist_id).await.ok().is_some();
    let template = PlaylistTemplate {
        title: playlist.name.clone(),
        total_tracks,
        tracks,
        playlist_id: playlist_id.to_string(),
        latest_job,
        has_generated_pdfs: has_pdfs,
    };

    Ok(Html(template.render()?))
}
