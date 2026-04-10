use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction, query, query_as};
use uuid::Uuid;

use crate::{
    entity::{asset::AssetHandle, page::PageOption},
    error::db::DbResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: AssetHandle,
    pub title: String,
    pub likes: i64,
    pub views: i64,
    pub tags: Vec<String>,
}

impl Article {
    /// 插入一篇文章数据，注意此函数不会自动提交事务
    pub async fn insert<'c>(tx: &mut Transaction<'c, Postgres>, title: String, author: Uuid, tags: &[String]) -> DbResult<AssetHandle> {
        let asset_id = query!("INSERT INTO asset(owner) VALUES ($1) RETURNING id", author)
            .fetch_one(tx.as_mut())
            .await
            .map(|v| v.id)?;

        query!("INSERT INTO article(id, title, tags) VALUES ($1, $2, $3) RETURNING id", asset_id, title, tags)
            .fetch_one(tx.as_mut())
            .await?;

        Ok(asset_id.into())
    }

    /// 精准匹配标题
    pub async fn by_title<'c, E>(db: E, title: String, opt: PageOption) -> DbResult<Vec<Article>>
    where
        E: PgExecutor<'c>,
    {
        Ok(query_as!(
            Article,
            r#"SELECT x.id, x.title, x.likes, x.views, x.tags
            FROM article AS x
                JOIN asset ON asset.id = x.id AND asset.status = 'available' AND asset.deleted_at IS NULL
            WHERE x.title = $1
            ORDER BY asset.created_at DESC
            LIMIT $2 OFFSET $3"#,
            title,
            opt.limit(),
            opt.offset()
        )
        .fetch_all(db)
        .await?)
    }

    /// 获取某一个人的发表的所有文章
    pub async fn by_author<'c, E>(db: E, id: Uuid, opt: PageOption) -> DbResult<Vec<Article>>
    where
        E: PgExecutor<'c>,
    {
        Ok(query_as!(
            Article,
            r#"SELECT x.id, x.title, x.likes, x.views, x.tags
            FROM article AS x
                JOIN asset ON asset.owner = $1 AND asset.status = 'available' AND asset.id = x.id AND asset.deleted_at IS NULL
            ORDER BY asset.created_at DESC
            LIMIT $2 OFFSET $3;"#,
            id,
            opt.limit(),
            opt.offset()
        )
        .fetch_all(db)
        .await?)
    }
}
