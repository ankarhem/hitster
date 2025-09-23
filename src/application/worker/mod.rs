mod tasks;

pub use tasks::*;

use crate::application::interfaces::IJobsRepository;
use crate::domain::job::Job;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

pub trait IWorkerTask: Serialize + for<'de> Deserialize<'de> + Send + 'static {
    type State: Clone + Send + Sync;
    type Output: Serialize + for<'de> Deserialize<'de> + Send + Sync;

    fn run(&self, state: &Self::State)
    -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}
pub trait IWorker: Send + Sync {
    type Task: IWorkerTask;
    fn enqueue(&self, task: Self::Task) -> impl Future<Output = Result<Job, anyhow::Error>> + Send;
}

pub struct Worker<JR: IJobsRepository, WT: IWorkerTask> {
    jobs_repository: Arc<JR>,
    task_sender: UnboundedSender<(Job, WT)>,
}

impl<JR: IJobsRepository, WT: IWorkerTask> IWorker for Worker<JR, WT> {
    type Task = WT;

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

impl<JR: IJobsRepository, WT: IWorkerTask> Worker<JR, WT> {
    pub fn new(jobs_repository: Arc<JR>, state: Arc<WT::State>) -> Self {
        let (task_sender, mut task_receiver) = mpsc::unbounded_channel::<(Job, WT)>();

        let _state = state.clone();
        let _jobs_repository = jobs_repository.clone();
        tokio::spawn(async move {
            let state = _state;
            let jobs_repository = _jobs_repository;

            while let Some((mut job, task)) = task_receiver.recv().await {
                job.status = crate::domain::JobStatus::Processing;
                if let Err(e) = jobs_repository.update(job.clone()).await {
                    error!("Failed to update job status to processing: {:?}", e);
                    continue;
                }

                // Run the task
                let started_at = chrono::Utc::now();
                let result = task.run(&state).await;
                let ended_at = chrono::Utc::now();
                let diff = ended_at - started_at;
                info!("Task finished after {} ms", diff.num_milliseconds());

                match result {
                    Ok(output) => {
                        job.status = crate::domain::JobStatus::Completed;
                        job.completed_at = Some(chrono::Utc::now());
                        match serde_json::to_value(output) {
                            Ok(output_value) => job.result = Some(output_value),
                            Err(e) => {
                                error!("Failed to serialize task output: {:?}", e);
                                job.status = crate::domain::JobStatus::Failed;
                            }
                        }
                        if let Err(e) = jobs_repository.update(job.clone()).await {
                            error!("Failed to update completed job: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Task failed to complete: {:?}", e);
                        job.status = crate::domain::JobStatus::Failed;
                        job.completed_at = Some(chrono::Utc::now());
                        if let Err(e) = jobs_repository.update(job.clone()).await {
                            error!("Failed to update failed job: {:?}", e);
                        }
                    }
                }
            }
        });

        Self {
            jobs_repository,
            task_sender,
        }
    }
}
