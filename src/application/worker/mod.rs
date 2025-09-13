mod tasks;

pub use tasks::*;

use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedSender};
use tracing::error;
use crate::application::interfaces::{IJobsRepository};
use crate::domain::job::{Job};

#[trait_variant::make(IWorkerTask: Send)]
pub trait _IWorkerTask: Serialize + for<'de> Deserialize<'de> {
    type State: Clone + Send + Sync;
    type Output: Serialize + for<'de> Deserialize<'de> + Send + Sync;

    async fn run(&self, state: &Self::State) -> anyhow::Result<Self::Output>;
}
#[trait_variant::make(IWorker: Send)]
pub trait _IWorker: Send + Sync {
    type Task: IWorkerTask;
    async fn enqueue(&self, task: Self::Task) -> Result<Job, anyhow::Error>;
}

pub struct Worker<JobsRepository, WorkerTask>
where
    JobsRepository: IJobsRepository,
    WorkerTask: IWorkerTask,
{
    jobs_repository: Arc<JobsRepository>,
    // state: Arc<WorkerTask::State>,
    task_sender: UnboundedSender<(Job, WorkerTask)>,
}

impl<JobsRepository, WorkerTask> IWorker for Worker<JobsRepository, WorkerTask>
where
    JobsRepository: IJobsRepository + 'static,
    WorkerTask: IWorkerTask + 'static,
{
    type Task = WorkerTask;

    async fn enqueue(&self, task: Self::Task) -> Result<Job, anyhow::Error> {
        let payload = serde_json::to_value(&task)?;
        let job = Job::new(payload);
        let job = self.jobs_repository.create(job).await?;

        if let Err(e) = self.task_sender.send((job.clone(), task)) {
            return Err(anyhow::anyhow!("Failed to send task to worker {e}"));
        }

        Ok(job)
    }
}

impl<JobsRepository, WorkerTask> Worker<JobsRepository, WorkerTask>
where
    JobsRepository: IJobsRepository + 'static,
    WorkerTask: IWorkerTask + 'static,
{
    pub fn new(
        jobs_repository: Arc<JobsRepository>,
        state: Arc<WorkerTask::State>,
    ) -> Self {
        let (task_sender, mut task_receiver) = mpsc::unbounded_channel::<(Job, WorkerTask)>();

        let _state = state.clone();
        let _jobs_repository = jobs_repository.clone();
        tokio::spawn(async move {
            let state = _state;
            let jobs_repository = _jobs_repository;

            while let Some((mut job, task)) = task_receiver.recv().await {
                job.status = crate::domain::JobStatus::Processing;
                jobs_repository.update(job.clone()).await.unwrap();

                let result = task.run(&state).await;
                match result {
                    Ok(output) => {
                        job.status = crate::domain::JobStatus::Completed;
                        job.completed_at = Some(chrono::Utc::now());
                        job.result = Some(serde_json::to_value(output).unwrap());
                        jobs_repository.update(job).await.unwrap();
                    }
                    Err(e) => {
                        error!("Task failed to complete: {:?}", e);
                        job.status = crate::domain::JobStatus::Failed;
                        job.completed_at = Some(chrono::Utc::now());
                        jobs_repository.update(job).await.unwrap();
                    }
                }
            }
        });

        Self {
            jobs_repository,
            // state,
            task_sender
        }
    }
}