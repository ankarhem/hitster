use crate::infrastructure::Database;
use crate::application::models::{JobId, JobType, Job};
use crate::domain::errors::{DomainError, DomainResult};
use tracing::info;
use std::sync::Arc;

/// Application service for job management business logic
#[derive(Clone)]
pub struct JobService {
    database: Arc<Database>,
}

impl JobService {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
        }
    }

    /// Create a new job
    pub async fn create_job(&self, job_type: JobType) -> DomainResult<JobId> {
        info!("Creating job: {:?}", job_type);
        
        let playlist_id = match job_type {
            JobType::GeneratePlaylistPdf { id } => id,
        };
        
        let db_job = self.database.create_job(playlist_id.as_str()).await?;
        Ok(JobId::from_string(db_job.id)?)
    }

    /// Get a job by ID
    pub async fn get_job(&self, id: &JobId) -> DomainResult<Job> {
        info!("Getting job: {}", id);
        
        let db_job = self.database.get_job_by_id(id.as_str()).await?
            .ok_or_else(|| DomainError::JobNotFound(id.as_str().to_string()))?;
        
        let job = Job {
            id: JobId::from_string(db_job.id)?,
            playlist_id: db_job.playlist_id,
            status: db_job.status,
            created_at: db_job.created_at,
            completed_at: db_job.completed_at,
            front_pdf_path: db_job.front_pdf_path,
            back_pdf_path: db_job.back_pdf_path,
        };
        
        Ok(job)
    }
}