use axum::{debug_handler, extract::State, http::StatusCode, response::{IntoResponse, Response}, Extension};

use crate::{
    entity::usr::user_profiles::UsrProfile, http::api::usr::UsrIdent, server::ServerState
};

#[debug_handler]
#[tracing::instrument(name = "[usr/bio]", skip(state))]
pub(super) async fn bio_get(
    State(state): State<ServerState>,
    Extension(ident): Extension<UsrIdent>,
) -> Response {
    let res = UsrProfile::fetch_all_fields_by_id(state.db(), ident.id).await;
    match res {
        Ok(profile) => (StatusCode::OK, axum::Json(profile)).into_response(),
        Err(e) => if e.is_not_found() {
            StatusCode::UNAUTHORIZED.into_response()
        } else {
            Response::from(e)
        },
    }
}
