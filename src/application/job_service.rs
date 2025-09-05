use crate::application::{IJobRepository};
use crate::domain::{JobId, JobType, Job};

#[trait_variant::make(IJobService: Send)]
pub trait _IJobService: Send + Sync {
    async fn create(&self, job_type: &JobType) -> anyhow::Result<Job>;
    async fn get(&self, id: &JobId) -> anyhow::Result<Option<Job>>;
}

#[derive(Clone)]
pub struct JobService<JobRepository>
where JobRepository: IJobRepository
{
    repository: JobRepository
}

impl<JobRepository> JobService<JobRepository>
where JobRepository: IJobRepository
{
    pub fn new(repository: JobRepository) -> Self {
        Self { repository }
    }
}

impl<JobRepository> IJobService for JobService<JobRepository>
where JobRepository: IJobRepository
{
    async fn create(&self, job_type: &JobType) -> anyhow::Result<Job> {
        match &job_type {
            JobType::GeneratePlaylistPdf { id } => {
                self.repository.create(&job_type).await
            }
        }
    }

    async fn get(&self, id: &JobId) -> anyhow::Result<Option<Job>> {
        self.repository.get(id).await
    }
}