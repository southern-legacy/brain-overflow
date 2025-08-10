pub mod usr;

use axum::{Router, response::Response, routing};

use crate::server::ServerState;

type ApiResult = Result<Response, Response>;

pub fn build_router() -> Router<ServerState> {
    Router::new()
        .nest("/usr", usr::build_router())
        .route("/test", routing::get(|| async { "Hello world!" }))
}
