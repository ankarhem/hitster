use crate::application::models::JobId;
use crate::infrastructure::JobStatus;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Job model with domain types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub playlist_id: String,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub front_pdf_path: Option<String>,
    pub back_pdf_path: Option<String>,
}

impl Job {
    /// Get a display name for the job
    pub fn display_name(&self) -> String {
        format!("Job {} for playlist {}", self.id, self.playlist_id)
    }
    
    /// Check if the job is completed
    pub fn is_completed(&self) -> bool {
        self.status == JobStatus::Completed
    }
    
    /// Check if the job is processing
    pub fn is_processing(&self) -> bool {
        self.status == JobStatus::Processing
    }
    
    /// Check if the job is failed
    pub fn is_failed(&self) -> bool {
        self.status == JobStatus::Failed
    }
    
    /// Check if the job is pending
    pub fn is_pending(&self) -> bool {
        self.status == JobStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::models::JobId;

    #[test]
    fn test_job_creation() {
        let job_id = JobId::new();
        let job = Job {
            id: job_id.clone(),
            playlist_id: "test_playlist".to_string(),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            front_pdf_path: None,
            back_pdf_path: None,
        };
        
        assert_eq!(job.id, job_id);
        assert_eq!(job.playlist_id, "test_playlist");
        assert!(job.is_pending());
        assert!(!job.is_completed());
        assert!(!job.is_processing());
        assert!(!job.is_failed());
    }

    #[test]
    fn test_job_display_name() {
        let job_id = JobId::new();
        let job = Job {
            id: job_id.clone(),
            playlist_id: "test_playlist".to_string(),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            front_pdf_path: None,
            back_pdf_path: None,
        };
        
        let display_name = job.display_name();
        assert!(display_name.contains(&job_id.to_string()));
        assert!(display_name.contains("test_playlist"));
    }
}