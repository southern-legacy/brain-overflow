use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    entity::usr::user_profiles::UsrProfile,
    http::{
        api::ApiResult,
        extractor::Path
    },
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[usr/info]", skip(state))]
pub async fn info(State(state): State<ServerState>, Path(id): Path<i64>) -> ApiResult {
    let res = UsrProfile::fetch_all_fields_by_id(state.db(), id).await?;

    Ok((StatusCode::OK, axum::Json(res)).into_response())
}
