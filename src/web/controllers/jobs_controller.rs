use axum::{
    extract::{Path, State},
    response::Json,
};
use crate::web::{WebServer, AppError};
use crate::application::models::{JobId, Job};
use std::str::FromStr;

/// Controller for job API endpoints
pub struct JobsController {}

impl JobsController {
    /// GET /api/jobs/<JobId> -> Returns the job status
    pub async fn get_job(
        State(server): State<WebServer>,
        Path(job_id): Path<String>,
    ) -> Result<Json<Job>, AppError> {
        let job_id = JobId::from_str(&job_id)?;
        let job = server.job_service.get_job(&job_id).await?;
        Ok(Json(job))
    }
}