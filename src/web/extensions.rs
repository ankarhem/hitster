pub trait HtmxExtension {
    fn is_htmx_request(&self) -> bool;
}

impl HtmxExtension for axum::http::HeaderMap {
    fn is_htmx_request(&self) -> bool {
        self.get("hx-request").map(|v| v == "true").unwrap_or(false)
    }
}
