#[derive(Debug)]
pub struct TrackVM {
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub qr_code: String,
    pub album_cover_url: Option<String>,
}

impl TrackVM {
    pub fn has_album_cover(&self) -> bool {
        self.album_cover_url.is_some()
    }
    
    pub fn album_cover_url_or_empty(&self) -> &str {
        self.album_cover_url.as_deref().unwrap_or("")
    }
}

#[derive(Debug)]
pub enum JobKind {
    GeneratePdf,
    RefetchPlaylist,
}

#[derive(Debug)]
pub struct JobVM {
    pub id: String,
    pub is_in_progress: bool,
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
    pub playlist_id: String,
    pub latest_job: Option<JobVM>,
    pub has_generated_pdfs: bool,
}

impl PlaylistTemplate {
    pub fn enable_download_buttons(&self) -> bool {
        match &self.latest_job {
            Some(job) => !job.is_in_progress && self.has_generated_pdfs,
            None => false,
        }
    }

    pub fn has_job_in_progress(&self) -> bool {
        match &self.latest_job {
            Some(job) => job.is_in_progress,
            None => false,
        }
    }
}
