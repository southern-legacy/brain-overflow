mod usr;

use axum::Router;

pub fn build_router() -> Router {
    Router::new()
        .nest("/usr", usr::build_router())
}