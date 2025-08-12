use serde::Serialize;
use sqlx::PgPool;

use crate::db::SqlxError;

#[derive(Serialize)]
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
    pub async fn fetch_all_fields_by_id(db: &PgPool, id: i64) -> Result<Self, SqlxError> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "usr"."usr_info" "U" WHERE "U"."id" = $1"#,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn fetch_all_fields_by_email(db: &PgPool, email: &str) -> Result<Self, SqlxError> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "usr"."usr_info" "U" WHERE "U"."email" = $1"#,
            email
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn fetch_all_fields_by_phone(db: &PgPool, phone: &str) -> Result<Self, SqlxError> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "usr"."usr_info" "U" WHERE "U"."phone" = $1"#,
            phone
        );
        Ok(statement.fetch_one(db).await?)
    }

    // pub async fn insert_and_return_all(
    //     db: &PgPool,
    //     InsertParam {
    //         name,
    //         email,
    //         phone,
    //         passwd,
    //     }: InsertParam,
    // ) -> Result<UsrInfo, SqlxError> {
    //     let statement = sqlx::query_as!(
    //         UsrInfo,
    //         r#"
    //             INSERT INTO "usr"."usr_info" (name, email, phone, passwd_hash)
    //             VALUES ($1, $2, $3, $4)
    //             RETURNING *;
    //         "#,
    //         name,
    //         email,
    //         phone,
    //         passwd
    //     );
    //     let res = statement.fetch_one(db).await?;
    //     let _profile = sqlx::query!(
    //         r#"
    //             INSERT INTO "usr"."usr_profile" (usr_id)
    //             VALUES ($1);
    //         "#,
    //         res.id
    //     ).fetch_one(db).await?;

    //     Ok(res)
    // }

    /// 创建用户然后返回新建用户的 id
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
        let _profile = sqlx::query!(
            r#"
                INSERT INTO "usr"."usr_profile" ("usr_id")
                VALUES ($1) RETURNING "usr_id";
            "#,
            res.id
        )
        .fetch_one(db)
        .await?;

        Ok(res.id)
    }

    /// 按照提供的 id 删除一个用户的信息，这也会删除用户的 profile
    ///
    /// 返回值：`Ok(i64)` 标识删除的用户的 id
    pub async fn delete_by_id(db: &PgPool, id: i64) -> Result<Option<i64>, SqlxError> {
        let statement = sqlx::query!(
            r#"DELETE FROM "usr"."usr_info" "U" WHERE "U"."id" = $1 RETURNING "U"."id""#,
            id
        );
        match statement.fetch_optional(db).await? {
            Some(rec) => Ok(Some(rec.id)),
            None => Ok(None),
        }
    }
}
