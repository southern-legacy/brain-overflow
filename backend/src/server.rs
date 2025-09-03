use crate::{
    app_config::{self, server::AuthConfig},
    http, logger,
};
use axum::extract::{DefaultBodyLimit, Request};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use sqlx::PgPool;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
    time::Duration,
};
use tokio::net::TcpListener;
use tower_http::{
    cors::{self, CorsLayer},
    normalize_path::NormalizePathLayer,
    timeout::TimeoutLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info};

#[derive(Clone)]
pub struct ServerState {
    db: Arc<PgPool>,
    auth: &'static AuthConfig,
}

impl ServerState {
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    pub fn auth_config(&self) -> &AuthConfig {
        &self.auth
    }
}

pub async fn start() {
    let logo = tokio::fs::read_to_string("logo").await;
    let conn = crate::db::init().await;

    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request| {
            let method = req.method().to_string();
            let uri = req.uri().to_string();
            let req_id = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4()); // 使用 base64 编码的 uuid 作为请求 req_id
            tracing::info_span!("[request id]", req_id, method, uri)
        })
        .on_failure(())
        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
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
        .layer(cors_layer)
        .layer(path_normalize_layer);

    let listener = if app_config::server().ipv6_enabled() {
        TcpListener::bind((Ipv6Addr::UNSPECIFIED, app_config::server().port()))
            .await
            .unwrap()
    } else {
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, app_config::server().port()))
            .await
            .unwrap()
    };

    let error = match logo {
        Ok(val) => Ok(println!("{val}")),
        Err(e) => Err(e),
    };

    logger::init();

    if let Err(e) = error {
        error!("cannot load logo file because {e}");
    }

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        router.with_state(ServerState {
            db: Arc::new(conn),
            auth: app_config::server().auth(),
        }),
    )
    .await
    .unwrap()
}
