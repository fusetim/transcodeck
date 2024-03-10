use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::fragment)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub fragment_id: Uuid,
    pub media_id: Uuid,
    pub filename: String,
    pub fragment_number: Option<i32>,
    pub encryption_key: Option<String>,
    pub retrieval_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::fragment)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFragment {
    pub media_id: Uuid,
    pub filename: String,
    pub fragment_number: Option<i32>,
    pub encryption_key: Option<String>,
    pub retrieval_url: Option<String>,
}
