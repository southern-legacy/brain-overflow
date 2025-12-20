use axum::{
    Extension, Router, debug_handler,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use crab_vault::auth::{HttpMethod, Jwt, Permission};
use http::StatusCode;
use uuid::Uuid;

use crate::{
    entity::asset::AssetHandle,
    http::api::{ApiResult, user::UserIdent},
    server::ServerState,
};

pub fn build_router() -> Router<ServerState> {
    Router::new().route("/asset/{id}", routing::get(get))
}

#[debug_handler]
async fn get(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    _user_ident: Option<Extension<UserIdent>>,
) -> ApiResult {
    let asset = AssetHandle::from(id)
        .get(&state.database)
        .await?
        .ok_or(StatusCode::NOT_FOUND.into_response())?;

    let permmision = Permission::new_minimum()
        .permit_method(vec![HttpMethod::Safe])
        .permit_content_type(vec!["*".to_string()])
        .permit_resource_pattern(asset.newest_key)
        .restrict_maximum_size_option(None);

    let encoder_config = &state.config.crab_vault.encoder_config;
    let token = encoder_config.encoder.encode_randomly(&Jwt::new(
        &encoder_config.issue_as,
        &encoder_config.audience,
        permmision,
    ))?;
    
    Ok(token.into_response())
}
