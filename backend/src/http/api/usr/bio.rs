use axum::{
    Extension, debug_handler,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use crab_vault::auth::{HttpMethod, Jwt, Permission, error::AuthError};

use crate::{
    app_config, entity::usr::user_profiles::UsrProfile, http::{ENCODER_TO_CRAB_VAULT, api::usr::UsrIdent},
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
pub(super) async fn safe_bio_operation(
    method: axum::http::Method,
    request: Request,
) -> Result<String, AuthError> {
    let method = <axum::http::Method as Into<HttpMethod>>::into(method);

    if !method.safe() {
        return Err(AuthError::InsufficientPermissions);
    }

    let permission = Permission::new_minimum()
        .permit_method(vec![method])
        .permit_resource_pattern(request.uri().path())
        .restrict_maximum_size(0)
        .permit_content_type(vec![
            "image/jpeg".into(),
            "image/png".into(),
            "image/webp".into(),
        ]);

    let config = app_config::auth().encoder_config_to_crab_vault();
    let jwt = Jwt::new(config.issue_as(), config.audience(), permission)
        .expires_in(config.expire_in())
        .not_valid_in(config.not_valid_in());

    ENCODER_TO_CRAB_VAULT.encode_randomly(&jwt)
}
