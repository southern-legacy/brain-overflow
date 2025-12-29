use axum::{
    Extension, Router, debug_handler,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use crab_vault::auth::{HttpMethod, Jwt, Permission};
use http::{StatusCode, header};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entity::asset::{AssetHandle, AssetStatus},
    error::db::DbError,
    http::api::{ApiResult, user::UserIdent},
    server::ServerState,
};

pub fn build_router() -> Router<ServerState> {
    Router::new()
        .route("/asset/{id}", routing::get(safe))
        .route("/asset/{id}", routing::head(safe))
        .route("/asset/{id}", routing::delete(delete))
        .route("/asset/{id}", routing::put(start_upload))
        .route("/asset/{id}/end", routing::any(end_upload))
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

    if asset.status != AssetStatus::Available {
        // 如果这个 asset 不可用，那么就会返回这个状态码
        return Err(StatusCode::IM_A_TEAPOT.into_response());
    }

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

    let token = Jwt::new(
        &encoding_config.issue_as,
        &encoding_config.audience,
        Permission::new()
            .permit_method(vec![HttpMethod::Put])
            .permit_resource_pattern(&asset.newest_key)
            .restrict_maximum_size_option(None)
            .permit_content_type(vec!["*".to_string()]),
    );

    Ok((
        StatusCode::OK,
        // asset 结构体中的 newest_key 字段，就是客户端需要访问的地方（带域名）
        [(header::LOCATION, asset.newest_key)],
        json!({
            "token": encoding_config.encoder.encode_randomly(&token)?
        })
        .to_string(),
    )
        .into_response())
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
        let mut transaction = state.database.begin().await.map_err(DbError::from)?;

        AssetHandle::from(id)
            .logically_delete(transaction.as_mut())
            .await?;
    }

    todo!()
}
