use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse, Extension};

use crate::{
    entity::usr::user_profiles::UsrProfile, http::api::{usr::UsrIdent, ApiResult}, server::ServerState
};

#[debug_handler]
#[tracing::instrument(name = "[usr/bio]", skip(state))]
pub(super) async fn bio_get(
    state: State<ServerState>,
    Extension(ident): Extension<UsrIdent>,
) -> ApiResult {
    let profile = UsrProfile::fetch_all_fields_by_id(state.db(), ident.id).await?;
    Ok((StatusCode::OK, axum::Json(profile)).into_response())
}
