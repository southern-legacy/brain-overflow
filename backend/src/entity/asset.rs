use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres, query, query_as};
use uuid::Uuid;

use crate::error::db::DbResult;

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy, Debug)]
#[sqlx(rename_all = "snake_case", type_name = "asset_status")]
#[derive(PartialEq)]
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
    pub key: String,
    pub status: AssetStatus,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct AssetHandle {
    pub id: Uuid,
    pub allow_deleted: bool,
}

impl Asset {
    #[allow(dead_code)]
    #[inline]
    pub fn deleted(&self) -> bool {
        matches!(self.deleted_at, Some(deleted) if deleted < Utc::now())
    }

    /// 更新 [`Asset`] 记录
    pub async fn write_back<'c, E>(&self, db: E) -> DbResult<Option<()>>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let Self {
            id,
            key,
            status,
            created_at,
            updated_at,
            deleted_at,
        } = self;

        let query = query!(
            r#"
                UPDATE asset
                SET key = $2, status = $3, created_at = $4, updated_at = $5, deleted_at = $6
                WHERE id = $1
                RETURNING id;
            "#,
            id,
            key,
            status as _,
            created_at,
            updated_at,
            deleted_at.as_ref()
        ).fetch_optional(db).await?;

        Ok(query.map(|_| ()))
    }

    /// 将一个新的 [`Asset`] 记录添加到数据库中
    pub async fn insert<'c, E>(&self, db: E) -> DbResult<AssetHandle>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let Self {
            id,
            key,
            status,
            created_at,
            updated_at,
            deleted_at,
        } = self;

        let query = query!(
            r#"
                INSERT INTO asset (id, key, status, created_at, updated_at, deleted_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id;
            "#,
            id,
            key,
            status as _,
            created_at,
            updated_at,
            deleted_at.as_ref()
        ).fetch_one(db).await?;

        Ok(query.id.into())
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

#[allow(dead_code)]
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

    /// 允许过期的 [`Asset`] 被查找到
    pub const fn allow_deleted(mut self) -> Self {
        self.allow_deleted = true;
        self
    }

    pub async fn get<'c, E>(&self, db: E) -> DbResult<Option<Asset>>
    where
        E: Executor<'c, Database = Postgres>,
    {
        Ok(if self.allow_deleted {
            query_as!(
                Asset,
                r#"
                    SELECT
                        id, key, created_at, updated_at, deleted_at,
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
                        id, key, created_at, updated_at, deleted_at,
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

    /// ### **逻辑删除**
    ///
    /// 同所有的方法一样，这个函数的 `db` 也是一个执行器类型，可以是一个
    /// [`sqlx::Transaction`]，也可以是一个
    /// [`sqlx::Pool<Postgres>`]，
    ///
    /// 返回值说明：
    /// - Ok(Some())：确确实实有一个 [`Asset`] 被标记为了删除
    /// - Ok(None)：没找到这个 [`AssetHandle`] 指定的 [`Asset`]
    /// - Err([`DbError`](crate::error::db::DbError))：发生了各种各样的错误
    pub async fn logically_delete<'c, E>(&self, db: E) -> DbResult<Option<()>>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let query = query!(
            r#"UPDATE "asset" SET "deleted_at" = now() WHERE "id" = $1;"#,
            self.id
        )
        .fetch_optional(db)
        .await?;

        Ok(query.map(|_| ()))
    }

    pub async fn hard_delete<'c, E>(&self, db: E) -> DbResult<Option<()>>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let query = query!(r#"DELETE FROM "asset" WHERE "id" = $1;"#, self.id)
            .fetch_optional(db)
            .await?;

        Ok(query.map(|_| ()))
    }
}
