use axum::{
    Extension, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use crab_vault_auth::{Jwt, Permission, error::AuthError};

use crate::{
    app_config, entity::usr::user_profiles::UsrProfile, http::api::usr::UsrIdent,
    server::ServerState,
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
        Err(e) => {
            if e.is_not_found() {
                StatusCode::UNAUTHORIZED.into_response()
            } else {
                Response::from(e)
            }
        }
    }
}

#[debug_handler]
#[tracing::instrument(name = "[usr/bio/issue token]")]
pub(super) async fn bio_operation() -> Result<String, AuthError> {
    Jwt::encode(
        &Jwt::new(Permission::new_root()),
        app_config::auth().jwt_config().await,
    )
}
