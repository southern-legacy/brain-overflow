use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{entity::usr::usr_info::UsrInfo, http::api::ApiResult, server::ServerState};

#[debug_handler]
#[tracing::instrument(name = "[usr/info]", skip(state))]
pub async fn info(State(state): State<ServerState>, Path(id): Path<i64>) -> ApiResult {
    let res = UsrInfo::find_by_id(state.db(), id).await?;

    match res {
        Some(res) => Ok(axum::Json(res).into_response()),
        None => Err((StatusCode::NOT_FOUND).into_response()),
    }
}
