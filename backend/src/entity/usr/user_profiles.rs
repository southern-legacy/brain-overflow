use std::iter::Map;

use sqlx::{prelude::FromRow, types::{time::OffsetDateTime, Json}};

#[allow(dead_code)]
#[derive(FromRow)]
pub struct UsrProfile {
    pub usr_id: i64,
    pub biography: String,
    pub avator: String,
    pub backgound: String,
    pub contact_me: Json<Map<String, String>>,
    pub updated_at: OffsetDateTime
}