use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transcoding_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TranscodingJob {
    pub transcoding_job_id: Uuid,
    pub media_id: Uuid,
    pub status: JobStatus,
    pub ffmpeg_command: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transcoding_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTranscodingJob {
    pub media_id: Uuid,
    pub ffmpeg_command: String,
    pub status: JobStatus,
}

#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::JobStatus"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Deleted,
}