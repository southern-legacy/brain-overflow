use axum::{
    Extension, Json, Router, debug_handler,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing,
};
use http::{StatusCode, header};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entity::{article::Article, page::PageOption}, error::db::DbError, http::{
        api::{ApiResult, user::AccessToken},
        middleware::auth::{AuthLayer, Judgement},
    }, redis::ARTICLE_CACHE_TTL, server::ServerState
};

pub(super) fn build_router(
    state: ServerState,
    auth_layer: AuthLayer<AccessToken, impl Judgement<AccessToken>>,
) -> Router {
    Router::new()
        .route("/article", routing::post(post_article))
        .layer(auth_layer)
        .route("/user/{user_id}/article", routing::get(get_article_of_someone))
        .route("/article/{id}", routing::get(get_article_by_id))
        .route("/article", routing::get(get_article_by_title))
        .with_state(state)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PostArticle {
    title: String,
    tags: Vec<String>,
}

#[debug_handler]
async fn post_article(
    State(state): State<ServerState>,
    Extension(id): Extension<AccessToken>,
    Json(PostArticle { title, tags }): Json<PostArticle>,
) -> ApiResult {
    let new_article = {
        let mut transaction = state.begin_transaction().await?;
        let res = Article::insert(&mut transaction, title, id.into(), &tags).await?;
        transaction.commit().await.map_err(DbError::from)?;
        res.id
    };
    Ok((
        StatusCode::OK,
        [(header::LOCATION, state.prefix_uri(format!("/asset/{new_article}")))],
        axum::Json(json!({"id": new_article})),
    )
        .into_response())
}

#[debug_handler]
async fn get_article_by_id(State(state): State<ServerState>, Path(id): Path<Uuid>) -> ApiResult {
    let mut redis = state.redis();
    let res = match redis.get::<_, Option<String>>(id).await {
        Ok(Some(v)) => {
            // let _ = redis.incr::<_, _, ()>("get_article_by_id:cache:hits", 1).await;
            v
        }
        Ok(None) | Err(_) => {
            let res = Article::by_id(&state.database(), id)
                .await?
                .as_ref()
                .map(serde_json::to_string)
                .ok_or(StatusCode::NOT_FOUND.into_response())?
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

            let _ = redis
                .set_ex::<_, _, ()>(id, &res, ARTICLE_CACHE_TTL)
                .await
                .map_err(|e| tracing::warn!("error communicating with redis {e}"));

            res
        }
    };
    // let _ = redis.incr::<_, _, ()>("get_article_by_id:cache:total", 1).await;

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], res).into_response())
}

#[debug_handler]
async fn get_article_of_someone(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    Query(opts): Query<PageOption>,
) -> ApiResult {
    let res = Article::by_author(&state.database(), id, opts).await?;
    Ok((StatusCode::OK, axum::Json(res)).into_response())
}

#[derive(Deserialize, Debug)]
struct GetByTitle {
    title: String,

    #[serde(flatten, default)]
    page_opts: PageOption,
}

#[debug_handler]
async fn get_article_by_title(State(state): State<ServerState>, Query(opts): Query<GetByTitle>) -> ApiResult {
    let res = Article::by_title(&state.database(), opts.title, opts.page_opts).await?;
    Ok((StatusCode::OK, axum::Json(res)).into_response())
}
