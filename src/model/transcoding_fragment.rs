use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transcoding_fragment_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TranscodingFragmentJob {
    pub transcoding_fragment_job_id: Uuid,
    pub transcoding_job_id: Uuid,
    pub fragment_id: Uuid,
    pub status: FragmentJobStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transcoding_fragment_job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTranscodingFragmentJob {
    pub transcoding_job_id: Uuid,
    pub fragment_id: Uuid,
    pub status: FragmentJobStatus,
}


#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::FragmentJobStatus"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentJobStatus {
    Pending,
    Queued,
    Reserved,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Deleted,
}