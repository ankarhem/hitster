use crate::web::templates::ErrorTemplate;
use askama::Template;
use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode, HeaderValue};

/// Application error type for handling all web-related errors
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum AppError {
    /// Any error: {0}
    Anything(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Anything(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error occurred")
            },
        };
        tracing::error!("Error: {}", self);

        // Try to render the error template
        let template = ErrorTemplate {
            error_message: error_message.to_string(),
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
                (status, format!("Error: {}", error_message)).into_response()
            }
        }
    }
}