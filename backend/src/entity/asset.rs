#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query, query_as};
use uuid::Uuid;

use crate::error::db::DbResult;

#[derive(Deserialize, Serialize, Debug)]
pub struct Asset {
    /// 这个资源的 id，使用 UUID
    pub id: Uuid,

    /// 最新版本的 URI 路径
    pub newest_key: String,

    /// 所有历史版本的 URI 路径
    pub history: Vec<String>,

    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct AssetHandle {
    id: Uuid,
    allow_deleted: bool,
}

impl Asset {
    pub fn deleted(&self) -> bool {
        matches!(self.deleted_at, Some(deleted) if deleted < chrono::Utc::now())
    }
}

impl AssetHandle {
    pub fn generate() -> Self {
        Self {
            id: Uuid::now_v7(),
            allow_deleted: false,
        }
    }

    pub const fn new_with_id(id: Uuid) -> Self {
        Self {
            id,
            allow_deleted: false,
        }
    }

    pub const fn allow_deleted(mut self) -> Self {
        self.allow_deleted = true;
        self
    }

    pub async fn get_by_id(&self, db: &PgPool) -> DbResult<Option<Asset>> {
        Ok(if self.allow_deleted {
            query_as!(Asset, r#"SELECT * FROM "asset" WHERE "id" = $1;"#, self.id)
                .fetch_optional(db)
                .await?
        } else {
            query_as!(
                Asset,
                r#"SELECT * FROM "asset" WHERE "id" = $1 AND "deleted_at" IS NULL;"#,
                self.id
            )
            .fetch_optional(db)
            .await?
        })
    }

    pub async fn insert_to_db(&self, db: &PgPool, newest_key: String) -> DbResult<()> {
        query!(
            r#"INSERT INTO "asset" (id, newest_key, history) VALUES ($1, $2, ARRAY[$2]);"#,
            self.id,
            newest_key
        )
        .fetch_one(db)
        .await?;

        Ok(())
    }

    pub async fn update_to(&self, db: &PgPool, newest_key: String) -> DbResult<Option<()>> {
        let query = query!(
            r#"
UPDATE "asset"
SET
    "history" = array_append("history", "newest_key"),
    "newest_key" = $1
WHERE
    "id" = $2 AND "deleted_at" IS NULL"#,
            newest_key,
            self.id
        )
        .fetch_optional(db)
        .await?;

        Ok(query.map(|_| ()))
    }

    pub async fn logically_delete(&self, db: &PgPool) -> DbResult<Option<()>> {
        let query = query!(
            r#"UPDATE "asset" SET "deleted_at" = now() WHERE "id" = $1;"#,
            self.id
        )
        .fetch_optional(db)
        .await?;

        Ok(query.map(|_| ()))
    }

    pub async fn hard_delete(&self, db: &PgPool) -> DbResult<Option<()>> {
        let query = query!(
            r#"DELETE FROM "asset" WHERE "id" = $1;"#,
            self.id
        )
        .fetch_optional(db)
        .await?;

        Ok(query.map(|_| ()))
    }
}
