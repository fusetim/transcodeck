use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::media)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Media {
    pub media_id: Uuid,
    pub basename: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::media)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMedia {
    pub basename: Option<String>,
}

