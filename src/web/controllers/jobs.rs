use axum::{
    extract::{Path, State},
    response::Json,
};
use crate::domain::{Job, JobId};
use crate::web::error::ApiError;
use crate::web::server::Services;

pub async fn get_job<JobsService, PlaylistService>(
    State(services): State<Services<JobsService, PlaylistService>>,
    Path(job_id): Path<String>,
) -> Result<Json<Job>, ApiError>
where 
    JobsService: crate::application::job_service::IJobService,
    PlaylistService: crate::application::playlist_service::IPlaylistService,
{
    let job_id = job_id.parse::<JobId>()?; 
    match services.job_service.get(&job_id).await? {
        Some(job) => Ok(Json(job)),
        None => todo!(),
    }
}