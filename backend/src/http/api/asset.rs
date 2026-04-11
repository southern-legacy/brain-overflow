use auth::Jwt;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
    Extension, Router, debug_handler,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use http::{Method, StatusCode};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    entity::asset::{AssetHandle, AssetStatus},
    error::db::DbError,
    http::{
        api::{ApiResult, user::UserIdent},
        middleware::auth::{AuthLayer, Validator},
    },
    server::ServerState,
};

pub fn build_router<F>(state: ServerState, auth_layer: AuthLayer<UserIdent, F>) -> Router
where
    F: Validator<UserIdent>,
{
    Router::new()
        // .route("/asset/{id}", routing::delete(delete))
        .route("/asset/{id}", routing::put(start_upload))
        .route_layer(auth_layer)
        .route("/asset/{id}", routing::get(safe))
        .route("/asset/{id}", routing::head(safe))
        .with_state(state)
}

#[debug_handler]
async fn safe(State(state): State<ServerState>, method: http::Method, Path(id): Path<Uuid>, _user_ident: Option<Extension<UserIdent>>) -> ApiResult {
    let asset = AssetHandle::from(id)
        .get(&state.database)
        .await?
        .ok_or(StatusCode::NOT_FOUND.into_response())?;

    if asset.status != AssetStatus::Available {
        return Err(StatusCode::NOT_FOUND.into_response());
    }

    let (client, bucket, url_ttl) = (&state.s3_client, &state.config.s3.bucket, state.config.s3.url_ttl);
    let presigned_request = match method {
        Method::GET => client
            .get_object()
            .bucket(bucket)
            .key(asset.id.to_string())
            .presigned(PresigningConfig::expires_in(Duration::from_secs(url_ttl)).unwrap())
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "failed to generate presigned URL");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?,
        Method::HEAD => client
            .head_object()
            .bucket(bucket)
            .key(asset.id.to_string())
            .presigned(PresigningConfig::expires_in(Duration::from_secs(url_ttl)).unwrap())
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "failed to generate presigned URL");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?,
        _ => unreachable!(),
    };

    let response = json!({ "url": presigned_request.uri() });

    Ok((StatusCode::OK, axum::Json(response)).into_response())
}

#[debug_handler]
async fn start_upload(State(state): State<ServerState>, Path(id): Path<Uuid>, Extension(user_ident): Extension<UserIdent>) -> ApiResult {
    let (bucket, url_ttl) = (&state.config.s3.bucket, state.config.s3.url_ttl);

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

    if asset.owner != user_ident.id {
        return Err((StatusCode::FORBIDDEN, axum::Json(json!({"code":"sdva"}))).into_response());
    }

    let presigned_request = state
        .s3_client
        .put_object()
        .bucket(bucket)
        .key(id.to_string())
        .presigned(PresigningConfig::expires_in(Duration::from_secs(url_ttl)).unwrap())
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to generate presigned URL");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let response = json!({
        "upload_url": presigned_request.uri().to_string(),
        "asset_id": asset.id,
        "method": "PUT",
        "expires_in": 900,
        "bucket": bucket,
    });

    Ok((StatusCode::OK, axum::Json(response)).into_response())
}

#[debug_handler]
#[allow(unused)]
async fn delete(State(state): State<ServerState>, Path(id): Path<Uuid>, Extension(_user_ident): Extension<UserIdent>) -> ApiResult {
    {
        let _owner = AssetHandle::from(id).logically_delete(&state.database).await?;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
