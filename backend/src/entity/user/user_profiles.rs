use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, query};
use uuid::Uuid;

use crate::{entity::asset::AssetHandle, error::db::DbResult};

#[allow(dead_code)]
#[derive(Serialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub biography: Option<AssetHandle>,
    pub avatar: Option<AssetHandle>,
    pub banner: Option<AssetHandle>,
    pub contact_me: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

impl UserProfile {
    pub async fn fetch_all_fields_by_id(db: &PgPool, id: Uuid) -> DbResult<Self> {
        let v = query!(
            r#"SELECT * FROM "user"."user_profile" WHERE "user"."user_profile"."user_id" = $1;"#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(UserProfile {
            user_id: v.user_id,
            biography: v.biography.map(AssetHandle::from),
            avatar: v.avatar.map(AssetHandle::from),
            banner: v.banner.map(AssetHandle::from),
            contact_me: v.contact_me,
            updated_at: v.updated_at,
        })
    }
}
