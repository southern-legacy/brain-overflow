use crate::error::db::DbResult;
use redis::{FromRedisValue, ToRedisArgs, ToSingleRedisArg};
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,

    #[serde(skip)]
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertParam {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
}

impl ToRedisArgs for InsertParam {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(&serde_json::to_vec(self).expect("This can never fail"));
    }
}

impl ToSingleRedisArg for InsertParam {}

impl FromRedisValue for InsertParam {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        let json_str = String::from_redis_value(v)?;
        serde_json::from_str(&json_str).map_err(|e| redis::ParsingError::from(e.to_string()))
    }
}

impl UserInfo {
    pub async fn fetch_all_fields_by_id<'c, E>(db: E, id: Uuid) -> DbResult<Self>
    where
        E: PgExecutor<'c>,
    {
        let statement = sqlx::query_as!(Self, r#"SELECT * FROM "user"."user_info" "U" WHERE "U"."id" = $1"#, id);
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn fetch_all_fields_by_email<'c, E>(db: E, email: &str) -> DbResult<Self>
    where
        E: PgExecutor<'c>,
    {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "user"."user_info" "U" WHERE "U"."email" = $1"#,
            email
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn fetch_all_fields_by_phone<'c, E>(db: E, phone: &str) -> DbResult<Self>
    where
        E: PgExecutor<'c>,
    {
        let statement = sqlx::query_as!(
            Self,
            r#"SELECT * FROM "user"."user_info" "U" WHERE "U"."phone" = $1"#,
            phone
        );
        Ok(statement.fetch_one(db).await?)
    }

    /// 创建用户然后返回新建用户的 id
    pub async fn insert_and_return_id<'a, 'c, E>(
        db: E,
        InsertParam {
            id,
            name,
            email,
            phone,
            password,
        }: &InsertParam,
    ) -> DbResult<Uuid>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query!(
            r#"
                INSERT INTO "user"."user_info" (id, name, email, phone, password_hash)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING "id";
            "#,
            id,
            name,
            email.as_ref(),
            phone.as_ref(),
            password
        )
        .fetch_one(db)
        .await?;

        Ok(res.id)
    }

    /// 按照提供的 id 删除一个用户的信息，这也会删除用户的 profile
    ///
    /// 返回值：`Uuid` 标识删除的用户的 id
    pub async fn delete_by_id<'c, E>(db: E, id: Uuid) -> DbResult<Uuid>
    where
        E: PgExecutor<'c>,
    {
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

    pub async fn update_name<'c, E>(db: E, id: Uuid, new_name: &str) -> DbResult<UserInfo>
    where
        E: PgExecutor<'c>,
    {
        let statement = sqlx::query_as!(
            UserInfo,
            r#"
                UPDATE "user"."user_info"
                SET "name" = $1
                WHERE "id" = $2
                RETURNING *;
            "#,
            new_name,
            id
        );
        Ok(statement.fetch_one(db).await?)
    }

    pub async fn update_email<'c, E>(db: E, id: Uuid, new_email: &str) -> DbResult<UserInfo>
    where
        E: PgExecutor<'c>,
    {
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

    pub async fn update_phone<'c, E>(db: E, id: Uuid, new_phone: &str) -> DbResult<UserInfo>
    where
        E: PgExecutor<'c>,
    {
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

    pub async fn update_password_hash<'c, E>(db: E, id: Uuid, new_password_hash: &str) -> DbResult<UserInfo>
    where
        E: PgExecutor<'c>,
    {
        let statement = sqlx::query_as!(
            UserInfo,
            r#"
                UPDATE "user"."user_info"
                SET "password_hash" = $1
                WHERE "id" = $2
                RETURNING *;
            "#,
            new_password_hash,
            id,
        );
        Ok(statement.fetch_one(db).await?)
    }
}
