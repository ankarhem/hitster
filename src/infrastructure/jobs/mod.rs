use crate::application::IJobsRepository;
use crate::domain;
use crate::infrastructure::entities::JobEntity;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Clone)]
pub struct JobsRepository {
    pool: Pool<Sqlite>,
}

impl JobsRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

impl IJobsRepository for JobsRepository {
    async fn create(&self, job: domain::Job) -> anyhow::Result<domain::Job> {
        let entity: JobEntity = job.clone().into();

        sqlx::query(
            "INSERT INTO jobs (id, status, created_at, payload, result) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entity.id)
        .bind(entity.status)
        .bind(entity.created_at)
        .bind(entity.payload)
        .bind(entity.result)
        .execute(&self.pool)
        .await?;

        Ok(job)
    }

    async fn get(&self, id: &domain::JobId) -> anyhow::Result<Option<domain::Job>> {
        let id: Uuid = id.clone().into();
        let job_entity = sqlx::query_as::<_, JobEntity>(
            "SELECT id, status, created_at, completed_at, kind, payload FROM jobs WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(job_entity.map(domain::Job::from))
    }

    async fn update(&self, job: domain::Job) -> anyhow::Result<domain::Job> {
        let entity: JobEntity = job.clone().into();

        sqlx::query(
            "UPDATE jobs SET status = ?, completed_at = ?, payload = ?, result = ? WHERE id = ?",
        )
        .bind(entity.status)
        .bind(entity.completed_at)
        .bind(entity.payload)
        .bind(entity.result)
        .bind(entity.id)
        .execute(&self.pool)
        .await?;

        Ok(job)
    }

    async fn get_by_playlist_id(&self, playlist_id: &crate::domain::PlaylistId) -> anyhow::Result<Vec<domain::Job>> {
        let playlist_id_str = playlist_id.to_string();
        
        let job_entities = sqlx::query_as::<_, JobEntity>(
            "SELECT id, status, created_at, completed_at, kind, payload FROM jobs 
             WHERE json_extract(payload, '$.playlist_id') = ? 
             ORDER BY created_at DESC",
        )
        .bind(playlist_id_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(job_entities.into_iter().map(domain::Job::from).collect())
    }
}
