use crate::domain::{Job, JobId};
use std::future::Future;

pub trait IJobsRepository: Clone + Send + Sync + 'static {
    fn create(&self, job: Job) -> impl Future<Output = anyhow::Result<Job>> + Send;
    fn get(&self, id: &JobId) -> impl Future<Output = anyhow::Result<Option<Job>>> + Send;
    fn update(&self, job: Job) -> impl Future<Output = anyhow::Result<Job>> + Send;
    fn get_by_playlist_id(
        &self,
        playlist_id: &crate::domain::PlaylistId,
    ) -> impl Future<Output = anyhow::Result<Vec<Job>>> + Send;
}
