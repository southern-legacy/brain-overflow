pub mod usr;

use axum::{Router, http::StatusCode, response::Response};
use regex::Regex;

use crate::{
    http::services::crab_vault::{TokenIssueService, TokenIssueServiceInner},
    server::ServerState,
};

type ApiResult = Result<Response, Response>;

pub fn build_router() -> Router<ServerState> {
    let inner = TokenIssueServiceInner::default()
        .regex("")
        .allowed_content_types(vec![])
        .allowed_methods(&[])
        .map_fn(|_: Regex, _: &'_ str| Ok("".into()))
        .max_size_option(None);

    let service = TokenIssueService::new(inner);

    Router::new()
        .nest("/usr", usr::build_router())
        .route_service("/usr/bio/{*}", service)
        .fallback(|| async { StatusCode::NOT_FOUND })
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
}
 