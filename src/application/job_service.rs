use crate::application::{IJobsRepository};
use crate::domain::{JobId, JobType, Job};

#[trait_variant::make(IJobService: Send)]
pub trait _IJobService: Send + Sync {
    async fn create(&self, job_type: &JobType) -> anyhow::Result<Job>;
    async fn get(&self, id: &JobId) -> anyhow::Result<Option<Job>>;
}

#[derive(Clone)]
pub struct JobsService<JobsRepository>
where JobsRepository: IJobsRepository
{
    repository: JobsRepository
}

impl<JobsRepository> JobsService<JobsRepository>
where JobsRepository: IJobsRepository
{
    pub fn new(repository: JobsRepository) -> Self {
        Self { repository }
    }
}

impl<JobRepository> IJobService for JobsService<JobRepository>
where JobRepository: IJobsRepository
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