use crate::domain::{Job, JobId, JobType};

#[trait_variant::make(IJobRepository: Send)]
pub trait _IJobRepository: Send + Sync {
    async fn create(&self, job_type: &JobType) -> anyhow::Result<Job>;
    async fn get(&self, job_id: &JobId) -> anyhow::Result<Option<Job>>;
}