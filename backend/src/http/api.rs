pub mod article;
pub mod asset;
pub mod user;
pub mod webhook;

use auth::Jwt;
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;

use crate::{
    http::{
        api::user::{AccessToken, RefreshToken},
        middleware::auth::AuthLayer,
    },
    server::ServerState,
};

type ApiResult = Result<Response, Response>;

pub fn build_router(state: ServerState) -> Router {
    let strict_layer = AuthLayer::new(state.clone(), |_, _, token: Jwt<RefreshToken>| {
        Box::pin(async move { Ok(token.load) })
    });

    let auth_layer = AuthLayer::new(state.clone(), |state, _, token: Jwt<AccessToken>| {
        Box::pin(async move {
            match state.redis().sismember("brain-overflow:banned-jti", token.jti).await {
                Ok(false) => Ok(token.load),
                Ok(true) => Err(StatusCode::FORBIDDEN.into_response()),
                _ => Err(StatusCode::UNAUTHORIZED.into_response()),
            }
        })
    });

    Router::new()
        .merge(user::build_router(state.clone(), auth_layer.clone(), strict_layer))
        .merge(article::build_router(state.clone(), auth_layer.clone()))
        .merge(asset::build_router(state.clone(), auth_layer))
        .merge(webhook::build_router(state))
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
        .fallback(|| async {
            (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"code": "noSuchApi"})),
            )
        })
}
