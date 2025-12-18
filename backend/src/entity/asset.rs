#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query, query_as};
use uuid::Uuid;

use crate::error::db::DbResult;

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum OwnerType {
    User,
    Article,
    Question,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum AssetStatus {
    Init,
    Uploading,
    Available,
    Failed,
    Aborted,
    Deleted,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Asset {
    /// 这个资源的 id，使用 UUID
    pub id: Uuid,

    /// 最新版本的 URI 路径
    pub newest_key: String,

    pub status: AssetStatus,

    pub owner: Uuid,

    pub owner_type: OwnerType,

    /// 所有历史版本的 URI 路径
    pub history: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct AssetHandle {
    id: Uuid,
    allow_deleted: bool,
}

impl Asset {
    pub fn deleted(&self) -> bool {
        matches!(self.deleted_at, Some(deleted) if deleted < Utc::now())
    }
}

impl Serialize for AssetHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AssetHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        uuid::serde::simple::deserialize(deserializer).map(AssetHandle::from)
    }
}

impl From<Uuid> for AssetHandle {
    fn from(id: Uuid) -> Self {
        Self::new_with_id(id)
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

    pub async fn get(&self, db: &PgPool) -> DbResult<Option<Asset>> {
        Ok(if self.allow_deleted {
            query_as!(
                Asset, 
                r#"
                    SELECT
                        id, newest_key, owner, history, created_at, updated_at, deleted_at,
                        owner_type as "owner_type: OwnerType",
                        status as "status: AssetStatus"
                    FROM "asset"
                    WHERE "id" = $1;
                "#, 
                self.id
            )
                .fetch_optional(db)
                .await?
        } else {
            query_as!(
                Asset, 
                r#"
                    SELECT
                        id, newest_key, owner, history, created_at, updated_at, deleted_at,
                        owner_type as "owner_type: OwnerType",
                        status as "status: AssetStatus"
                    FROM "asset"
                    WHERE "id" = $1;
                "#,
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
        .execute(db)
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
                    "id" = $2 AND "deleted_at" IS NULL
            "#,
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
        let query = query!(r#"DELETE FROM "asset" WHERE "id" = $1;"#, self.id)
            .fetch_optional(db)
            .await?;

        Ok(query.map(|_| ()))
    }
}
