use displaydoc::Display;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(
    Debug, Display, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct JobId(Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<JobId> for Uuid {
    fn from(value: JobId) -> Self {
        value.0
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
        let uuid = Uuid::parse_str(s)?;
        Ok(Self(uuid))
    }
}

impl From<Uuid> for JobId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum JobStatus {
    /// pending
    Pending,
    /// processing
    Processing,
    /// completed
    Completed,
    /// failed
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobKind {
    GeneratePdfs,
    RefetchPlaylist,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub kind: JobKind,
    pub payload: serde_json::Value,
}

impl Job {
    pub fn new(kind: JobKind, payload: serde_json::Value) -> Self {
        Self {
            id: JobId::new(),
            status: JobStatus::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
            kind,
            payload,
        }
    }
}

pub trait BackgroundTask: Serialize + for<'de> Deserialize<'de> {
    type State;

    fn kind(&self) -> String;

    fn run(&self, state: &Self::State) -> impl Future<Output = anyhow::Result<()>> + Send;
}
