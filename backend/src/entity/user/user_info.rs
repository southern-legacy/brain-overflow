use serde::Serialize;
use sqlx::PgPool;

use crate::error::db::DbResult;

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,

    #[serde(skip)]
    pub passwd_hash: String,
}

pub struct InsertParam<'a> {
    pub name: &'a str,
    pub email: Option<&'a String>,
    pub phone: Option<&'a String>,
    pub passwd: &'a str,
}

impl UserInfo {
    pub async fn fetch_all_fields_by_id(db: &PgPool, id: i64) -> DbResult<Self> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "user"."user_info" "U" WHERE "U"."id" = $1"#,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn fetch_all_fields_by_email(db: &PgPool, email: &str) -> DbResult<Self> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "user"."user_info" "U" WHERE "U"."email" = $1"#,
            email
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn fetch_all_fields_by_phone(db: &PgPool, phone: &str) -> DbResult<Self> {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "user"."user_info" "U" WHERE "U"."phone" = $1"#,
            phone
        );
        Ok(statement.fetch_one(db).await?)
    }

    /// 创建用户然后返回新建用户的 id
    pub async fn insert_and_return_id<'a>(
        db: &PgPool,
        InsertParam {
            name,
            email,
            phone,
            passwd,
        }: InsertParam<'a>,
    ) -> DbResult<i64> {
        let statement = sqlx::query!(
            r#"
INSERT INTO "user"."user_info" (name, email, phone, passwd_hash)
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
INSERT INTO "user"."user_profile" ("user_id")
VALUES ($1)
RETURNING "user_id";
            "#,
            res.id
        )
        .fetch_one(db)
        .await?;

        Ok(res.id)
    }

    /// 按照提供的 id 删除一个用户的信息，这也会删除用户的 profile
    ///
    /// 返回值：`i64` 标识删除的用户的 id
    pub async fn delete_by_id(db: &PgPool, id: i64) -> DbResult<i64> {
        let statement = sqlx::query!(
            r#"
DELETE FROM "user"."user_info"
WHERE "id" = $1
RETURNING "id";
        "#,
            id
        );
        Ok(statement.fetch_one(db).await?.id)
    }

    pub async fn update_email(db: &PgPool, id: i64, new_email: &str) -> DbResult<UserInfo> {
        let statement = sqlx::query_as!(
            UserInfo,
            r#"
UPDATE "user"."user_info"
SET "email" = $1
WHERE "id" = $2
RETURNING *;
            "#,
            new_email,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn update_phone(db: &PgPool, id: i64, new_phone: &str) -> DbResult<UserInfo> {
        let statement = sqlx::query_as!(
            UserInfo,
            r#"
UPDATE "user"."user_info"
SET "phone" = $1
WHERE "id" = $2
RETURNING *;
            "#,
            new_phone,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn update_passwd_hash(
        db: &PgPool,
        id: i64,
        new_passwd_hash: &str,
    ) -> DbResult<UserInfo> {
        let statement = sqlx::query_as!(
            UserInfo,
            r#"
UPDATE "user"."user_info"
SET "passwd_hash" = $1
WHERE "id" = $2
RETURNING *;
            "#,
            new_passwd_hash,
            id,
        );
        Ok(statement.fetch_one(db).await?)
    }
}
