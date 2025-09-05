use std::str::FromStr;
use uuid::Uuid;

/// Job ID wrapper for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct JobId(String);

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl JobId {
    /// Create a new JobId
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create a JobId from a string
    pub fn from_string(id: String) -> Result<Self, anyhow::Error> {
        if id.trim().is_empty() {
            return Err(anyhow::anyhow!("JobId cannot be empty"));
        }
        
        // Validate that it's a valid UUID format
        Uuid::parse_str(&id)?;
        
        Ok(Self(id))
    }
    
    /// Get the JobId as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the JobId as a String
    pub fn into_string(self) -> String {
        self.0
    }
    
    /// Get the JobId as a UUID
    pub fn as_uuid(&self) -> Result<Uuid, anyhow::Error> {
        Ok(Uuid::parse_str(&self.0)?)
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for JobId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

impl From<Uuid> for JobId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_creation() {
        let job_id = JobId::new();
        assert!(!job_id.as_str().is_empty());
        assert!(job_id.as_uuid().is_ok());
    }

    #[test]
    fn test_job_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let job_id = JobId::from_string(uuid_str.to_string()).unwrap();
        assert_eq!(job_id.as_str(), uuid_str);
    }

    #[test]
    fn test_job_id_from_str() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let job_id: JobId = uuid_str.parse().unwrap();
        assert_eq!(job_id.as_str(), uuid_str);
    }

    #[test]
    fn test_job_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let job_id = JobId::from(uuid);
        assert_eq!(job_id.as_str(), uuid.to_string());
    }

    #[test]
    fn test_job_id_empty_string() {
        let result = JobId::from_string("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_job_id_invalid_uuid() {
        let result = JobId::from_string("invalid-uuid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_job_id_into_string() {
        let job_id = JobId::new();
        let string = job_id.into_string();
        assert!(!string.is_empty());
    }
}