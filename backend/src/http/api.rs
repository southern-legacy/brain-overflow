pub mod user;

use axum::{Router, http::StatusCode, response::Response};

use crate::{
    http::services::crab_vault::{TokenIssueService, TokenIssueServiceInner},
    server::ServerState,
};

type ApiResult = Result<Response, Response>;

pub fn build_router() -> Router<ServerState> {
    let inner = TokenIssueServiceInner::default()
        .allowed_content_types(vec![])
        .allowed_methods(&[])
        .max_size_option(None);

    let service = TokenIssueService::new(inner);

    Router::new()
        .nest("/user", user::build_router())
        .route_service("/user/bio/{*}", service)
        .fallback(|| async { StatusCode::NOT_FOUND })
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
}
