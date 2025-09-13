use crate::domain::SpotifyIdParserError;
use crate::web::templates::ErrorTemplate;
use askama::Template;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

/// Error type for handling errors in view rendering
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum TemplateError {
    /// Not found
    NotFound(String),

    /// Template rendering error: {0}
    RenderError(#[from] askama::Error),
    ///  error: {0}
    ApplicationError(#[from] anyhow::Error),
    /// Custom error: {0}
    GenerateQrCode(#[from] qrcode::types::QrError),
}

impl IntoResponse for TemplateError {
    fn into_response(self) -> Response {
        let mut details = "Something went wrong. Please try again later.".to_string();

        let status = match self {
            TemplateError::NotFound(message) => {
                tracing::info!("Not Found: {}", message);
                details = message;
                StatusCode::NOT_FOUND
            }
            TemplateError::RenderError(err) => {
                tracing::error!("Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            TemplateError::ApplicationError(err) => {
                tracing::error!("Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            TemplateError::GenerateQrCode(err) => {
                tracing::error!("Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        // Try to render the error template
        let template = ErrorTemplate {
            details,
            status_code: status,
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
    /// Unmapped error: {0}
    Internal(#[from] anyhow::Error),
    /// ValidationError: {0}
    ValidationError(String),
    ///  Resource not found
    NotFound,
}

impl From<SpotifyIdParserError> for ApiError {
    fn from(err: SpotifyIdParserError) -> Self {
        ApiError::ValidationError(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::Internal(_) => {
                tracing::error!("{}", self);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::ValidationError(_) => {
                tracing::info!("{}", self);
                StatusCode::BAD_REQUEST
            }
            ApiError::NotFound => {
                tracing::info!("{}", self);
                StatusCode::NOT_FOUND
            }
        };

        (status, self.to_string()).into_response()
    }
}
