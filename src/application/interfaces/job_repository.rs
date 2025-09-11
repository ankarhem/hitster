use crate::domain::{Job, JobId};

#[trait_variant::make(IJobsRepository: Send)]
pub trait _IJobsRepository: Send + Sync + Clone {
    async fn create(&self, job: Job) -> anyhow::Result<Job>;
    async fn get(&self, id: &JobId) -> anyhow::Result<Option<Job>>;
    async fn update(&self, job: Job) -> anyhow::Result<Job>;
}