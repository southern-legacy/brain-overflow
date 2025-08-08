mod usr;

use axum::Router;

use crate::server::ServerState;

pub fn build_router() -> Router<ServerState> {
    Router::new().nest("/usr", usr::build_router())
}