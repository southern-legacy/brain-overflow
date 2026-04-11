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
    http::{api::user::UserIdent, middleware::auth::AuthLayer},
    server::ServerState,
};

type ApiResult = Result<Response, Response>;

pub fn build_router(state: ServerState) -> Router {
    let auth_layer = AuthLayer::new(state.clone(), |state, _, token: Jwt<UserIdent>| {
        Box::pin(async move {
            // 获取二进制形式的 jti 存储，为什么不封 user_id 呢？因为有可能是黑客发起的重放攻击
            // 我不太想错杀
            match state.redis().sismember("brain-overflow:banned-jti", token.jti).await {
                Ok(false) => Ok(token.load),
                Ok(true) => Err(StatusCode::FORBIDDEN.into_response()),
                _ => Err(StatusCode::UNAUTHORIZED.into_response()),
            }
        })
    });

    Router::new()
        .merge(user::build_router(state.clone(), auth_layer.clone()))
        .merge(article::build_router(state.clone(), auth_layer.clone()))
        .merge(asset::build_router(state.clone(), auth_layer))
        .merge(webhook::build_router(state.clone()))
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
        .fallback(|| async { StatusCode::NOT_FOUND })
}
