use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Executor, Postgres, query};
use uuid::Uuid;

use crate::{entity::asset::AssetHandle, error::db::DbResult};

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
    pub async fn fetch_all_fields_by_id<'c, E>(db: E, id: Uuid) -> DbResult<Self>
    where
        E: Executor<'c, Database = Postgres>,
    {
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

    pub async fn insert<'c, E>(id: Uuid, db: E) -> DbResult<()>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let _res = query!(
            r#"
                INSERT INTO "user"."user_profile"(user_id, updated_at)
                VALUES ($1, $2);
            "#, id, Utc::now()
        )
        .execute(db).await?;

        Ok(())
    }

    pub async fn write_back<'c, E>(self, db: E) -> DbResult<Option<()>>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = query!(
            r#"
                UPDATE "user"."user_profile"
                SET "biography" = $2, "avatar" = $3, "banner" = $4, "contact_me" = $5, "updated_at" = $6
                WHERE "user"."user_profile"."user_id" = $1;
            "#, self.user_id, self.biography.map(|v| v.id), self.avatar.map(|v| v.id), self.banner.map(|v| v.id), self.contact_me, Utc::now()
        )
        .execute(db).await?;

        match res.rows_affected() {
            0 => Ok(None),
            _ => Ok(Some(()))
        }
    }
}
