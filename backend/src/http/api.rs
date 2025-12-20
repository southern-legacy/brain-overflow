pub mod asset;
pub mod user;

use axum::{Router, http::StatusCode, response::Response};

use crate::{app_config::AppConfig, server::ServerState};

type ApiResult = Result<Response, Response>;

pub fn build_router(config: &AppConfig) -> Router<ServerState> {
    user::build_router(config)
        .merge(asset::build_router())
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
        .fallback(|| async { StatusCode::NOT_FOUND })
}
