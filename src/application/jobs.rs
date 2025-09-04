use crate::infrastructure::{Database, Job, JobStatus};
use anyhow::Result;
use tokio::sync::mpsc;
use std::sync::Arc;
use tracing::{info, error};

#[derive(Clone)]
pub struct JobService {
    database: Arc<Database>,
    job_sender: mpsc::UnboundedSender<JobMessage>,
}

#[derive(Debug, Clone)]
pub struct JobMessage {
    pub job_id: String,
    pub playlist_id: String,
}

impl JobService {
    pub fn new(database: Arc<Database>) -> Self {
        let (job_sender, mut job_receiver) = mpsc::unbounded_channel();
        
        let service = Self {
            database,
            job_sender,
        };
        
        // Start background worker
        let db_clone = service.database.clone();
        tokio::spawn(async move {
            while let Some(message) = job_receiver.recv().await {
                let job_id = message.job_id.clone();
                if let Err(e) = Self::process_job(db_clone.clone(), message).await {
                    error!("Failed to process job {}: {}", job_id, e);
                }
            }
        });
        
        service
    }
    
    pub async fn create_job(&self, playlist_id: &str) -> Result<Job> {
        let job = self.database.create_job(playlist_id).await?;
        let job_id = job.id.clone();
        info!("Created job {} for playlist {}", job.id, playlist_id);
        
        // Send job to worker
        self.job_sender.send(JobMessage {
            job_id,
            playlist_id: playlist_id.to_string(),
        })?;
        
        Ok(job)
    }
    
    pub async fn get_job(&self, job_id: &str) -> Result<Option<Job>> {
        self.database.get_job_by_id(job_id).await
    }
    
    async fn process_job(database: Arc<Database>, message: JobMessage) -> Result<()> {
        let job_id = message.job_id.clone();
        let playlist_id = message.playlist_id.clone();
        
        // Update job status to processing
        database.update_job_status(&job_id, JobStatus::Processing).await?;
        info!("Processing job {} for playlist {}", job_id, playlist_id);
        
        // Get playlist and tracks
        let playlist = database.get_playlist_by_id(&playlist_id).await?
            .ok_or_else(|| anyhow::anyhow!("Playlist not found"))?;
        
        let tracks = database.get_tracks_by_playlist_id(&playlist_id).await?;
        
        // Generate PDFs
        let pdf_result = crate::generate_pdfs(&playlist, &tracks).await;
        
        match pdf_result {
            Ok((front_path, back_path)) => {
                // Update job with PDF paths and mark as completed
                database.update_job_pdf_paths(
                    &job_id,
                    Some(&front_path),
                    Some(&back_path),
                ).await?;
                
                database.update_job_status(&job_id, JobStatus::Completed).await?;
                info!("Completed job {} for playlist {}", job_id, playlist_id);
            }
            Err(e) => {
                // Mark job as failed
                database.update_job_status(&job_id, JobStatus::Failed).await?;
                error!("Failed to generate PDFs for job {}: {}", job_id, e);
            }
        }
        
        Ok(())
    }
}