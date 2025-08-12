use chrono::{DateTime, Local};
use serde::Serialize;
use sqlx::{query_as, PgPool};

use crate::db::SqlxError;

#[allow(dead_code)]
#[derive(Serialize)]
pub struct UsrProfile {
    pub usr_id: i64,
    pub biography: String,
    pub avatar: String,
    pub background: String,
    pub contact_me: serde_json::Value,
    pub updated_at: DateTime<Local>,
}

impl UsrProfile {
    pub async fn fetch_all_fields_by_id(db: &PgPool, id: i64) -> Result<Self, SqlxError> {
        let statement = query_as!(
            UsrProfile,
            r#"SELECT * FROM "usr"."usr_profile" "UP" WHERE "UP"."usr_id" = $1;"#,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }
}
