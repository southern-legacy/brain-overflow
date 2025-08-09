pub mod usr;

use axum::{routing, Router};

use crate::server::ServerState;

pub fn build_router() -> Router<ServerState> {
    Router::new().nest("/usr", usr::build_router())
    .route("/test", routing::get(|| async { "Hello world!" }))
}