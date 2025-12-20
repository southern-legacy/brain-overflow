use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use uuid::Uuid;

use crate::{
    entity::user::user_profiles::UserProfile,
    http::{api::ApiResult, extractor::Path},
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[user/info]", skip(state))]
pub async fn info(State(state): State<ServerState>, Path(id): Path<Uuid>) -> ApiResult {
    let res = UserProfile::fetch_all_fields_by_id(&state.database, id).await?;

    Ok((StatusCode::OK, axum::Json(res)).into_response())
}
