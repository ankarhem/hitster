use sqlx::{Pool, Sqlite};
use crate::application::IJobsRepository;
use crate::domain::{Job, JobId, JobType, JobStatus};
use crate::infrastructure::entities::JobEntity;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct JobsRepository {
    pool: Pool<Sqlite>,
}

impl JobsRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
        }
    }
}

impl IJobsRepository for JobsRepository {
    async fn create(&self, job_type: &JobType) -> anyhow::Result<Job> {
        let job_id = JobId::new();
        let playlist_id = match job_type {
            JobType::GeneratePlaylistPdf { id } => id,
        };
        
        let now = Utc::now();
        
        sqlx::query(
            "INSERT INTO jobs (id, playlist_id, status, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(Uuid::from(job_id.clone()))
        .bind(Uuid::from(playlist_id.clone()))
        .bind("pending")
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(Job {
            id: job_id,
            job_type: job_type.clone(),
            status: JobStatus::Pending,
            created_at: now,
            completed_at: None,
            front_pdf_path: None,
            back_pdf_path: None,
        })
    }

    async fn get(&self, job_id: &JobId) -> anyhow::Result<Option<Job>> {
        let job_entity = sqlx::query_as::<_, JobEntity>(
            "SELECT id, playlist_id, status, front_pdf_path, back_pdf_path, created_at, completed_at FROM jobs WHERE id = ?"
        )
        .bind(Uuid::from(job_id.clone()))
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(job_entity.map(Job::from))
    }
}