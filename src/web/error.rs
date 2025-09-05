use crate::web::templates::ErrorTemplate;
use askama::Template;
use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode, HeaderValue};

/// Application error type for handling all web-related errors
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum AppError {
    /// Any error: {0}
    Anything(#[from] anyhow::Error),
    /// IO error: {0}
    Io(#[from] std::io::Error),
    /// Invalid Spotify playlist URL: {0}
    InvalidPlaylistUrl(String),
    /// Spotify API error: {0}
    SpotifyApiError(String),
    /// Database error: {0}
    DatabaseError(String),
    /// Playlist not found: {0}
    PlaylistNotFound(String),
    /// Validation error: {0}
    ValidationError(String),
    /// Template error: {0}
    Template(#[from] askama::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, user_friendly_message) = match &self {
            AppError::Anything(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error occurred", "An unexpected error occurred. Please try again later.".to_string())
            },
            AppError::Io(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "An I/O error occurred", "A file system error occurred. Please try again.".to_string())
            },
            AppError::InvalidPlaylistUrl(url) => {
                (StatusCode::BAD_REQUEST, "Invalid Spotify playlist URL", format!("The URL '{}' doesn't appear to be a valid Spotify playlist URL. Please check the URL and try again.", url))
            },
            AppError::SpotifyApiError(details) => {
                (StatusCode::BAD_GATEWAY, "Spotify API error", format!("Unable to fetch playlist from Spotify: {}. Please check the playlist ID and ensure it's public.", details))
            },
            AppError::DatabaseError(_details) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error", "A database error occurred while processing your request. Please try again.".to_string())
            },
            AppError::PlaylistNotFound(id) => {
                (StatusCode::NOT_FOUND, "Playlist not found", format!("The requested playlist with ID '{}' was not found.", id))
            },
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "Validation error", format!("Validation error: {}", msg))
            },
            AppError::Template(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error", "A template error occurred while rendering the page.".to_string())
            },
        };
        
        tracing::error!("Error: {} - Details: {}", self, error_message);

        // Try to render the error template
        let template = ErrorTemplate {
            error_message: user_friendly_message.clone(),
            status_code: status.as_u16(),
        };

        match template.render() {
            Ok(body) => {
                let mut response = (status, axum::response::Html(body)).into_response();
                response.headers_mut().insert(
                    "content-type",
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                response
            },
            Err(err) => {
                tracing::error!("Failed to render error template: {}", err);
                // Fallback to plain text error
                (status, format!("Error: {}", user_friendly_message)).into_response()
            }
        }
    }
}