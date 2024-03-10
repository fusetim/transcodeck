// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "fragment_job_status"))]
    pub struct FragmentJobStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;
}

diesel::table! {
    fragment (fragment_id) {
        fragment_id -> Uuid,
        media_id -> Uuid,
        filename -> Text,
        fragment_number -> Nullable<Int4>,
        encryption_key -> Nullable<Text>,
        retrieval_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    media (media_id) {
        media_id -> Uuid,
        basename -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FragmentJobStatus;

    transcoding_fragment_job (transcoding_fragment_job_id) {
        transcoding_fragment_job_id -> Uuid,
        transcoding_job_id -> Uuid,
        fragment_id -> Uuid,
        status -> FragmentJobStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    transcoding_job (transcoding_job_id) {
        transcoding_job_id -> Uuid,
        media_id -> Uuid,
        status -> JobStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(fragment -> media (media_id));
diesel::joinable!(transcoding_fragment_job -> fragment (fragment_id));
diesel::joinable!(transcoding_fragment_job -> transcoding_job (transcoding_job_id));
diesel::joinable!(transcoding_job -> media (media_id));

diesel::allow_tables_to_appear_in_same_query!(
    fragment,
    media,
    transcoding_fragment_job,
    transcoding_job,
);
