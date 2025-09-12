pub struct PdfTrack {
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub qr_code: String,
}

#[derive(askama::Template)]
#[template(path = "pdf.html")]
pub struct PdfTemplate {
    pub title: String,
    pub track_chunks: Vec<Vec<Vec<PdfTrack>>>,
}