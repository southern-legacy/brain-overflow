pub mod article;
pub mod asset;
pub mod user;
pub mod webhook;

use auth::Jwt;
use axum::{Router, http::StatusCode, response::Response};

use crate::{http::{api::user::UserIdent, middleware::auth::AuthLayer}, server::ServerState};

type ApiResult = Result<Response, Response>;

pub fn build_router(state: ServerState) -> Router {
    let auth_layer = AuthLayer::new(state.clone(), |_, _, token: Jwt<UserIdent>| {
        Box::pin(async move { Ok(token.load) })
    });

    user::build_router(state.clone())
        .merge(article::build_router(state.clone(), auth_layer.clone()))
        .merge(asset::build_router(state.clone(), auth_layer))
        .merge(webhook::build_router(state.clone()))
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
        .fallback(|| async { StatusCode::NOT_FOUND })
}
