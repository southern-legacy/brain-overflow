mod login;
mod signup;
mod delete_account;

use std::sync::LazyLock;

use crate::http::middelware::auth::AUTH_LAYER;
use crate::server::ServerState;
use axum::{Router, routing};

const ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(|| argon2::Config::default());

pub(super) fn build_router() -> Router<ServerState> {
    let router = Router::new();
    router
        .route("/delete_account", routing::post(delete_account::delete_account))
        .route_layer(&*AUTH_LAYER)
        .route("/login", routing::post(login::login))
        .route("/signup", routing::post(signup::signup))
}
