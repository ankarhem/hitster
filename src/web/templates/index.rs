/// Template context for the index page
#[derive(askama::Template, Debug)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Page title
    pub title: String,
}