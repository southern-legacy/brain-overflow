pub mod article;
pub mod asset;
pub mod user;
pub mod webhook;

use axum::{Router, http::StatusCode, response::Response};

use crate::server::ServerState;

type ApiResult = Result<Response, Response>;

pub fn build_router(state: ServerState) -> Router {
    user::build_router(state.clone())
        .merge(article::build_router(state.clone()))
        .merge(asset::build_router(state.clone()))
        .merge(webhook::build_router(state.clone()))
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
        .fallback(|| async { StatusCode::NOT_FOUND })
}
