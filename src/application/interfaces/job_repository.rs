use crate::domain::{Job, JobId};

#[trait_variant::make(IJobsRepository: Send)]
pub trait _IJobsRepository: Send + Sync + Clone {
    async fn create(&self, job: Job) -> anyhow::Result<Job>;
    async fn get(&self, id: &JobId) -> anyhow::Result<Option<Job>>;
    async fn update(&self, job: Job) -> anyhow::Result<Job>;
    async fn get_by_playlist_id(&self, playlist_id: &crate::domain::PlaylistId) -> anyhow::Result<Vec<Job>>;
}
