use crate::infrastructure::Database;
use crate::application::models::{JobId, JobType, Pdf};
use crate::domain::errors::{DomainError, DomainResult};
use std::sync::Arc;
use tracing::{info, error, instrument};

/// Infrastructure service for processing background jobs
/// 
/// This service handles the actual execution of jobs in the background
#[derive(Clone)]
pub struct JobProcessor {
    database: Arc<Database>,
}

impl JobProcessor {
    /// Create a new JobProcessor
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
    
    /// Process a job based on its type
    /// 
    /// This method executes the actual work for a job and updates its status
    #[instrument(skip(self), fields(job_id, job_type))]
    pub async fn process_job(&self, job_id: &JobId, job_type: &JobType) -> DomainResult<()> {
        info!("Processing job {} of type {:?}", job_id, job_type);
        
        // Update job status to processing
        self.update_job_status(job_id, crate::infrastructure::JobStatus::Processing).await?;
        
        let result = match job_type {
            JobType::GeneratePlaylistPdf { id } => {
                self.process_pdf_generation_job(job_id, id).await
            }
        };
        
        match result {
            Ok(_) => {
                info!("Successfully completed job {}", job_id);
                self.update_job_status(job_id, crate::infrastructure::JobStatus::Completed).await?;
                Ok(())
            }
            Err(e) => {
                error!("Failed to process job {}: {}", job_id, e);
                self.update_job_status(job_id, crate::infrastructure::JobStatus::Failed).await?;
                Err(e)
            }
        }
    }
    
    /// Process a PDF generation job
    /// 
    /// This method handles the specific logic for generating PDFs for a playlist
    #[instrument(skip(self), fields(job_id, playlist_id))]
    async fn process_pdf_generation_job(&self, job_id: &JobId, playlist_id: &crate::application::models::PlaylistId) -> DomainResult<()> {
        info!("Generating PDFs for playlist {}", playlist_id);
        
        // Get playlist from database
        let playlist = self.database.get_playlist_by_id(playlist_id.as_str()).await
            .map_err(|e| DomainError::PdfGenerationFailed(e.to_string()))?
            .ok_or_else(|| DomainError::PlaylistNotFound(playlist_id.as_str().to_string()))?;
        
        // Get tracks for the playlist
        let tracks = self.database.get_tracks_by_playlist_id(playlist_id.as_str()).await
            .map_err(|e| DomainError::PdfGenerationFailed(e.to_string()))?;
        
        if tracks.is_empty() {
            return Err(DomainError::BusinessRuleViolation(
                "Cannot generate PDFs for empty playlist".to_string()
            ));
        }
        
        // Generate PDFs using the existing PDF generation logic
        let (front_path, back_path) = crate::generate_pdfs(&playlist, &tracks).await
            .map_err(|e| DomainError::PdfGenerationFailed(e.to_string()))?;
        
        // Update job with PDF paths
        self.update_job_pdf_paths(job_id, Some(&front_path), Some(&back_path)).await?;
        
        info!("Successfully generated PDFs for job {}: front={}, back={}", 
              job_id, front_path, back_path);
        
        Ok(())
    }
    
    /// Update job status in the database
    async fn update_job_status(&self, job_id: &JobId, status: crate::infrastructure::JobStatus) -> DomainResult<()> {
        self.database.update_job_status(job_id.as_str(), status).await
            .map_err(|e| DomainError::JobNotFound(e.to_string()))?;
        Ok(())
    }
    
    /// Update job PDF paths in the database
    async fn update_job_pdf_paths(&self, job_id: &JobId, front_path: Option<&str>, back_path: Option<&str>) -> DomainResult<()> {
        self.database.update_job_pdf_paths(job_id.as_str(), front_path, back_path).await
            .map_err(|e| DomainError::JobNotFound(e.to_string()))?;
        Ok(())
    }
    
    /// Get the generated PDF for a job
    /// 
    /// This method retrieves the PDF file for a completed job
    #[instrument(skip(self), fields(job_id, pdf_side))]
    pub async fn get_job_pdf(&self, job_id: &JobId, pdf_side: crate::application::models::PdfSide) -> DomainResult<Pdf> {
        info!("Getting PDF for job {} and side {:?}", job_id, pdf_side);
        
        // Get job from database
        let db_job = self.database.get_job_by_id(job_id.as_str()).await
            .map_err(|e| DomainError::JobNotFound(e.to_string()))?
            .ok_or_else(|| DomainError::JobNotFound(job_id.as_str().to_string()))?;
        
        // Check if job is completed
        if db_job.status != crate::infrastructure::JobStatus::Completed {
            return Err(DomainError::BusinessRuleViolation(
                format!("Job {} is not completed (status: {:?})", job_id, db_job.status)
            ));
        }
        
        // Get the appropriate PDF path
        let pdf_path = match pdf_side {
            crate::application::models::PdfSide::Front => db_job.front_pdf_path,
            crate::application::models::PdfSide::Back => db_job.back_pdf_path,
        };
        
        let pdf_path = pdf_path.ok_or_else(|| {
            DomainError::PdfNotFound(format!("{:?} PDF for job {}", pdf_side, job_id))
        })?;
        
        // Read the PDF file
        let pdf_data = tokio::fs::read(&pdf_path).await
            .map_err(|e| DomainError::PdfNotFound(e.to_string()))?;
        
        let pdf = Pdf::new(pdf_data);
        info!("Successfully retrieved PDF for job {} and side {:?}, size: {}", 
              job_id, pdf_side, pdf.size_string());
        
        Ok(pdf)
    }
    
    /// Check if a job is completed
    pub async fn is_job_completed(&self, job_id: &JobId) -> DomainResult<bool> {
        let db_job = self.database.get_job_by_id(job_id.as_str()).await
            .map_err(|e| DomainError::JobNotFound(e.to_string()))?
            .ok_or_else(|| DomainError::JobNotFound(job_id.as_str().to_string()))?;
        
        Ok(db_job.status == crate::infrastructure::JobStatus::Completed)
    }
    
    /// Get job status
    pub async fn get_job_status(&self, job_id: &JobId) -> DomainResult<crate::infrastructure::JobStatus> {
        let db_job = self.database.get_job_by_id(job_id.as_str()).await
            .map_err(|e| DomainError::JobNotFound(e.to_string()))?
            .ok_or_else(|| DomainError::JobNotFound(job_id.as_str().to_string()))?;
        
        Ok(db_job.status)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_job_processor_creation() {
        // This would need a mock database for testing
        // For now, just test that the struct can be created
        // let processor = JobProcessor::new(/* database */);
        // assert!(true);
    }
}