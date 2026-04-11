use axum::{
    Extension, Json, Router, debug_handler,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing,
};
use http::{StatusCode, header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entity::{article::Article, page::PageOption},
    error::db::DbError,
    http::{
        api::{ApiResult, user::UserIdent},
        middleware::auth::{AuthLayer, Judgement},
    },
    server::ServerState,
};

pub(super) fn build_router(state: ServerState, auth_layer: AuthLayer<UserIdent, impl Judgement<UserIdent>>) -> Router {
    Router::new()
        .route("/article", routing::post(post_article))
        .layer(auth_layer)
        .route("/user/{user_id}/article", routing::get(get_article_of_someone))
        .route("/article/{title}", routing::get(get_article_by_title))
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
    Extension(ident): Extension<UserIdent>,
    Json(PostArticle { title, tags }): Json<PostArticle>,
) -> ApiResult {
    let new_article = {
        let mut transaction = state.begin_transaction().await?;
        let res = Article::insert(&mut transaction, title, ident.id, &tags).await?;
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
async fn get_article_of_someone(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    Query(opts): Query<PageOption>,
) -> ApiResult {
    let res = Article::by_author(&state.database, id, opts).await?;
    Ok((StatusCode::OK, axum::Json(res)).into_response())
}

#[debug_handler]
async fn get_article_by_title(
    State(state): State<ServerState>,
    Path(title): Path<String>,
    Query(opts): Query<PageOption>,
) -> ApiResult {
    let res = Article::by_title(&state.database, title, opts).await?;
    Ok((StatusCode::OK, axum::Json(res)).into_response())
}
