pub mod usr;

use axum::{
    Router,
    http::StatusCode,
    response::Response,
};

use crate::server::ServerState;

type ApiResult = Result<Response, Response>;

pub fn build_router() -> Router<ServerState> {
    Router::new()
        .nest("/usr", usr::build_router())
        .fallback(|| async { StatusCode::NOT_FOUND })
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
}
