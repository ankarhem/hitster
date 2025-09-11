use crate::web::templates::ErrorTemplate;
use askama::Template;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

/// Error type for handling errors in view rendering
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum TemplateError {
    /// Template rendering error: {0}
    RenderError(#[from] askama::Error),
    ///  error: {0}
    ApplicationError(#[from] anyhow::Error),
}

impl IntoResponse for TemplateError {
    fn into_response(self) -> Response {
        let status = match self {
            TemplateError::RenderError(err) => {
                tracing::error!("Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            TemplateError::ApplicationError(err) => {
                tracing::error!("Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let user_friendly_message = "An unexpected error occurred. Please try again later.";
        // Try to render the error template
        let template = ErrorTemplate {
            error_message: user_friendly_message.to_string(),
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
            }
            Err(err) => {
                let message = format!("Failed to render error template: {}", err);
                tracing::error!(message);
                // Fallback to plain text error
                (status, message).into_response()
            }
        }
    }
}

/// Error type for handling api errors
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum ApiError {
    /// Placeholder error: {0}
    Placeholder(#[from] anyhow::Error),
    /// Spotify ID parsing error: {0}
    SpotifyIdParseError(#[from] crate::domain::spotify_id::SpotifyIdParserError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("API Error: {}", self);

        let status = match &self {
            ApiError::Placeholder(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SpotifyIdParseError(_) => StatusCode::BAD_REQUEST,
        };

        (status, self.to_string()).into_response()
    }
}
