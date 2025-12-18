use chrono::{DateTime, Local};
use serde::Serialize;
use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{entity::asset::AssetHandle, error::db::DbResult};

#[allow(dead_code)]
#[derive(Serialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub biography: AssetHandle,
    pub avatar: AssetHandle,
    pub banner: AssetHandle,
    pub contact_me: serde_json::Value,
    pub updated_at: DateTime<Local>,
}

impl UserProfile {
    pub async fn fetch_all_fields_by_id(db: &PgPool, id: Uuid) -> DbResult<Self> {
        let statement = query_as!(
            UserProfile,
            r#"SELECT * FROM "user"."user_profile" WHERE "user"."user_profile"."user_id" = $1;"#,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }
}
