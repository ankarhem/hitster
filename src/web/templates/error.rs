use axum::http::StatusCode;

/// Template context for error pages
#[derive(askama::Template, Debug)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    /// Error message to display
    pub details: String,
    /// HTTP status code
    pub status_code: StatusCode,
}
