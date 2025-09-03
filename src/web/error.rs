use crate::web::templates::ErrorTemplate;
use askama::Template;
use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode, HeaderValue};

/// Application error type for handling all web-related errors
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum AppError {
    /// Template rendering error: {0}
    Render(#[from] askama::Error),
    /// Service error: {0}
    Service(#[from] anyhow::Error),
    /// QR code generation error: {0}
    QrCode(String),
    /// Playlist not found: {0}
    NotFound(String),
    /// Internal server error
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Render(err) => {
                tracing::error!("Template rendering error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render template")
            },
            AppError::Service(err) => {
                tracing::error!("Service error: {}", err);
                // Check if it's a "not found" type error
                if err.to_string().contains("not found") || err.to_string().contains("404") {
                    (StatusCode::NOT_FOUND, "Playlist not found")
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Service error")
                }
            },
            AppError::QrCode(err) => {
                tracing::error!("QR code generation error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate QR code")
            },
            AppError::NotFound(message) => {
                tracing::warn!("Not found error: {}", message);
                (StatusCode::NOT_FOUND, message.as_str())
            },
            AppError::Internal(message) => {
                tracing::error!("Internal error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, message.as_str())
            },
        };

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

/// Helper function to convert anyhow::Error to AppError with additional context
pub fn anyhow_to_app_error(err: anyhow::Error, context: &str) -> AppError {
    AppError::Service(anyhow::anyhow!("{}: {}", context, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let render_error = AppError::Render(askama::Error::from(std::fmt::Error));
        assert!(render_error.to_string().contains("Template rendering error"));

        let service_error = AppError::Service(anyhow::anyhow!("test error"));
        assert!(service_error.to_string().contains("Service error"));

        let not_found_error = AppError::NotFound("test".to_string());
        assert!(not_found_error.to_string().contains("not found"));
    }

    #[test]
    fn test_anyhow_to_app_error() {
        let original_err = anyhow::anyhow!("network failed");
        let app_err = anyhow_to_app_error(original_err, "Failed to fetch playlist");
        
        match app_err {
            AppError::Service(err) => {
                assert!(err.to_string().contains("Failed to fetch playlist"));
                assert!(err.to_string().contains("network failed"));
            },
            _ => panic!("Expected Service error"),
        }
    }
}