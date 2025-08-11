use crate::{app_config, http, logger};
use axum::extract::{DefaultBodyLimit, Request};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use sqlx::PgPool;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};
use tokio::net::TcpListener;
use tower_http::{
    cors::{self, CorsLayer},
    normalize_path::NormalizePathLayer,
    timeout::TimeoutLayer,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::info;

#[derive(Clone, Debug)]
pub struct ServerState {
    db: PgPool,
}

impl ServerState {
    pub fn db(&self) -> &PgPool {
        &self.db
    }
}

pub async fn start() {
    logger::init();
    let conn = crate::db::init();

    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request| {
            let method = req.method().to_string();
            let uri = req.uri().to_string();
            let request_id = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4()); // 使用 base64 编码的 uuid 作为请求 req_id
            tracing::info_span!("", request_id, uri, method)
        })
        .on_failure(())
        .on_request(())
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));

    let timeout_layer = TimeoutLayer::new(Duration::from_secs(120));

    let body_limit_layer = DefaultBodyLimit::max((1024 * 1024 * 16) as usize); // 16 MB 的最大报文大小

    let cors_layer = CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_headers(cors::Any)
        .allow_origin(cors::Any)
        .allow_credentials(false)
        .max_age(Duration::from_secs(3600 * 24));

    let path_normalize_layer = NormalizePathLayer::trim_trailing_slash();

    let router = http::build_router();
    let router = router
        .layer(timeout_layer)
        .layer(body_limit_layer)
        .layer(tracing_layer)
        .layer(path_normalize_layer)
        .layer(cors_layer);

    if app_config::get_server().ipv4_enabled() {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, app_config::get_server().port()))
            .await
            .unwrap();
        info!("Listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, router.with_state(ServerState { db: conn.await }))
            .await
            .unwrap()
    } else if app_config::get_server().ipv6_enabled() {
        let listener = TcpListener::bind((Ipv6Addr::UNSPECIFIED, app_config::get_server().port()))
            .await
            .unwrap();
        info!("Listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, router.with_state(ServerState { db: conn.await }))
            .await
            .unwrap()
    }
}
