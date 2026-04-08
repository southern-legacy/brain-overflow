use axum::{
    Extension, Router, debug_handler,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use auth::Jwt;
use http::StatusCode;
use uuid::Uuid;

use crate::{
    app_config::AppConfig,
    entity::asset::{AssetHandle, AssetStatus},
    error::db::DbError,
    http::{
        api::{ApiResult, user::UserIdent},
        middleware::auth::AuthLayer,
    },
    server::ServerState,
};

pub fn build_router(config: &AppConfig) -> Router<ServerState> {
    let auth_layer = AuthLayer::new(
        config.auth.decoder_config.decoder.clone(),
        |_, token: Jwt<UserIdent>| Box::pin(async move { Ok(token.load) }),
    );
    Router::new()
        .route("/asset/{id}", routing::delete(delete))
        .route("/asset/{id}", routing::put(start_upload))
        .route("/asset/{id}/end", routing::any(end_upload))
        .route_layer(auth_layer)
        .route("/asset/{id}", routing::get(safe))
        .route("/asset/{id}", routing::head(safe))
}

#[debug_handler]
async fn safe(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    _user_ident: Option<Extension<UserIdent>>,
) -> ApiResult {
    todo!()
}

#[debug_handler]
async fn start_upload(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    Extension(_user_ident): Extension<UserIdent>,
) -> ApiResult {
    let encoder_config = &state.config.s3.encoder_config;

    let asset = {
        let mut transaction = state.database.begin().await.map_err(DbError::from)?;

        let mut asset = AssetHandle::from(id)
            .get(transaction.as_mut())
            .await?
            .ok_or(StatusCode::NOT_FOUND.into_response())?;

        asset.status = AssetStatus::Uploading;
        asset.write_back(transaction.as_mut()).await?;

        transaction.commit().await.map_err(DbError::from)?;
        asset
    };

    todo!()
}

#[debug_handler]
async fn end_upload(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    Extension(_user_ident): Extension<UserIdent>,
) -> ApiResult {
    {
        let mut transaction = state.database.begin().await.map_err(DbError::from)?;

        let mut asset = AssetHandle::from(id)
            .get(transaction.as_mut())
            .await?
            .ok_or(StatusCode::NOT_FOUND.into_response())?;

        asset.status = AssetStatus::Available;
        asset.write_back(transaction.as_mut()).await?;

        transaction.commit().await.map_err(DbError::from)?;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[debug_handler]
async fn delete(
    State(state): State<ServerState>,
    Path(id): Path<Uuid>,
    Extension(_user_ident): Extension<UserIdent>,
) -> ApiResult {
    {
        AssetHandle::from(id)
            .logically_delete(state.database.as_ref())
            .await?;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
