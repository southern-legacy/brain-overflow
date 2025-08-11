use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};

use crate::db::SqlxError;

#[derive(FromRow, Serialize)]
pub struct UsrInfo {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,

    #[serde(skip)]
    pub passwd_hash: String,
}

pub struct InsertParam {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub passwd: String,
}

impl UsrInfo {
    pub async fn find_by_id(db: &PgPool, id: i64) -> Result<Option<Self>, SqlxError> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "usr"."usr_info" "U" WHERE "U"."id" = $1"#,
            id
        );
        Ok(statement
            .fetch_optional(db)
            .await?)
    }

    pub async fn find_by_email(db: &PgPool, email: &str) -> Result<Option<Self>, SqlxError> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "usr"."usr_info" "U" WHERE "U"."email" = $1"#,
            email
        );
        Ok(statement
            .fetch_optional(db)
            .await?)
    }

    pub async fn find_by_phone(db: &PgPool, phone: &str) -> Result<Option<Self>, SqlxError> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "usr"."usr_info" "U" WHERE "U"."phone" = $1"#,
            phone
        );
        Ok(statement
            .fetch_optional(db)
            .await?)
    }

    pub async fn insert_and_return_all(
        db: &PgPool,
        InsertParam {
            name,
            email,
            phone,
            passwd,
        }: InsertParam,
    ) -> Result<UsrInfo, SqlxError> {
        let statement = sqlx::query_as!(
            UsrInfo,
            r#"
                INSERT INTO "usr"."usr_info" (name, email, phone, passwd_hash)
                VALUES ($1, $2, $3, $4)
                RETURNING *;
            "#,
            name,
            email,
            phone,
            passwd
        );
        let res = statement.fetch_one(db).await?;

        Ok(res)
    }

    #[allow(dead_code)]
    pub async fn insert_and_return_id(
        db: &PgPool,
        InsertParam {
            name,
            email,
            phone,
            passwd,
        }: InsertParam,
    ) -> Result<i64, SqlxError> {
        let statement = sqlx::query!(
            r#"
                INSERT INTO "usr"."usr_info" (name, email, phone, passwd_hash)
                VALUES ($1, $2, $3, $4)
                RETURNING "id";
            "#,
            name,
            email,
            phone,
            passwd
        );
        let res = statement.fetch_one(db).await?;

        Ok(res.id)
    }

    #[allow(dead_code)]
    pub async fn insert_and_no_return(
        db: &PgPool,
        InsertParam {
            name,
            email,
            phone,
            passwd,
        }: InsertParam,
    ) -> Result<(), SqlxError> {
        let statement = sqlx::query!(
            r#"
                INSERT INTO "usr"."usr_info" (name, email, phone, passwd_hash)
                VALUES ($1, $2, $3, $4);
            "#,
            name,
            email,
            phone,
            passwd
        );
        let _res = statement.fetch_one(db).await?;

        Ok(())
    }

    pub async fn delete_by_id(
        db: &PgPool,
        id: i64
    ) -> Result<usize, SqlxError> {
        let statement = sqlx::query!(
            r#"DELETE FROM "usr"."usr_info" "U" WHERE "U"."id" = $1 RETURNING "U"."id" CASCADE"#,
            id
        );
        let res = statement.fetch_all(db).await?;
        Ok(res.len())
    }
}
