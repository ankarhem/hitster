use crate::application::playlist_service::IPlaylistService;
use crate::web::error::TemplateError;
use crate::web::server::Services;
use crate::web::templates::playlist::TrackVM;
use crate::web::templates::{IndexTemplate, PlaylistTemplate};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};
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
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let template = PlaylistTemplate {
        title: playlist.name.clone(),
        total_tracks,
        tracks,
        job_id: "not implemented".to_string(),
        playlist_id: playlist_id.to_string(),
        has_completed_job: false,
    };

    Ok(Html(template.render()?))
}
