use axum::{debug_handler, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use sea_orm::EntityTrait;

use crate::{entity::usr::prelude::UsrInfo, http::api::ApiResult, server::ServerState};

#[debug_handler]
#[tracing::instrument(name = "[usr/info]", skip(state))]
pub async fn info(
    State(state): State<ServerState>,
    Path(id): Path<i64>
) -> ApiResult {
    let model = UsrInfo::find_by_id(id).one(state.db()).await;

    match model {
        Ok(Some(model)) => {
            Ok(axum::Json(model).into_response())
        },
        Ok(None) => {
            Err((StatusCode::NOT_FOUND).into_response())
        },
        Err(_e) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR).into_response())
        }
    }
}