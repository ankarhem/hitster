/// Template context for error pages
#[derive(askama::Template, Debug)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    /// Error message to display
    pub error_message: String,
    /// HTTP status code
    pub status_code: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_template_creation() {
        let template = ErrorTemplate {
            error_message: "Test error".to_string(),
            status_code: 404,
        };

        assert_eq!(template.error_message, "Test error");
        assert_eq!(template.status_code, 404);
    }
}
