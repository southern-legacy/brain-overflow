use axum::{debug_handler, routing, Router};

pub fn build_router() -> Router {
    let router = Router::new();
    router
        .route("/login", routing::post(login))
        .route("/signup", routing::post(signup))
        .route("/delete_account", routing::post(delete_account))
}

#[debug_handler]
async fn login() {
    todo!()
}

#[debug_handler]
async fn signup() {
    todo!()
}

#[debug_handler]
async fn delete_account() {
    todo!()
}