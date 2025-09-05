use sqlx::{Pool, Sqlite};
use crate::application::IJobRepository;
use crate::domain::{Job, JobId, JobType};

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

impl IJobRepository for JobsRepository {
    async fn create(&self, job_type: &JobType) -> anyhow::Result<Job>{
        todo!()
    }

    async fn get(&self, job_id: &JobId) -> anyhow::Result<Option<Job>> {
        todo!()
    }
}