#[derive(Debug)]
pub struct TrackVM {
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub qr_code: String,
}

/// Template context for the cards page
#[derive(askama::Template, Debug)]
#[template(path = "playlist.html")]
pub struct PlaylistTemplate {
    /// Page title
    pub title: String,
    pub total_tracks: usize,
    /// List of tracks to display
    pub tracks: Vec<TrackVM>,
    /// Helper fields for template
    pub job_id: String,
    pub playlist_id: String,
    pub has_completed_job: bool,
}
