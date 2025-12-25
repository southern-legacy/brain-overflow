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
    error::db::DbError,
    http::api::{ApiResult, user::UserIdent},
    server::ServerState,
};

pub fn build_router() -> Router<ServerState> {
    Router::new()
        .route("/asset/{id}", routing::get(safe))
        .route("/asset/{id}", routing::head(safe))
        .route("/asset/{id}", routing::put(start_upload))
}

#[debug_handler]
async fn safe(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    _user_ident: Option<Extension<UserIdent>>,
) -> ApiResult {
    let asset = AssetHandle::from(id)
        .get(state.database.as_ref())
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

#[debug_handler]
async fn start_upload(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    Extension(_user_ident): Extension<UserIdent>,
) -> ApiResult {
    let encoding_config = &state.config.crab_vault.encoder_config;

    let asset = {
        let mut tx = state.database.begin().await.map_err(DbError::from)?;

        let asset = AssetHandle::from(id)
            .get(tx.as_mut())
            .await?
            .ok_or(StatusCode::NOT_FOUND.into_response())?;

        tx.commit().await.map_err(DbError::from)?;
        asset
    };

    let token = Jwt::new(
        &encoding_config.issue_as,
        &encoding_config.audience,
        Permission::new()
            .permit_method(vec![HttpMethod::Put])
            .permit_resource_pattern(asset.newest_key)
            .restrict_maximum_size_option(None)
            .permit_content_type(vec!["*".to_string()])
    );

    Ok(encoding_config.encoder.encode_randomly(&token)?.into_response())
}
