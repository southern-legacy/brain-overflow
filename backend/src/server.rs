use crate::{app_config, http, logger};
use ::http::StatusCode;
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
}

impl ServerState {
    pub fn db(&self) -> &PgPool {
        &self.db
    }
}

pub async fn start() {
    let logo = r"⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣴⡶⠿⢶⣦⡀⠀⢰⣶⠀⣶⡆⢰⣶⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣶⠿⠟⠁⣴⣶⣦⠈⢿⡆⢸⣿⠀⣿⣇⢸⣿⡀⢀⣴⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⠟⢠⣶⣶⣤⣿⣿⣿⠁⢸⡟⠛⠛⠛⠛⠛⠛⠛⠛⠻⣿⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⠟⢡⡄⠸⣿⣿⣿⡛⠛⠻⠆⢸⡿⠟⠛⠛⠛⠛⠛⠟⣿⡆⢹⡿⠿⠿⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⠀⣿⣿⣶⣄⠙⣿⣿⣿⣷⡀⢸⡇⠀⠀⠀⠀⠀⠀⠀⣿⡇⢸⣷⣶⣶⣶⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣾⠟⢀⡈⢻⣿⣿⣿⣿⣿⣋⠉⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⣿⡇⢸⣧⣤⣤⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⠃⣰⣿⣧⣈⠛⠛⣿⣿⣿⣿⣿⠄⢸⡇⠀⠀⠀⠀⠀⠀⠀⣿⡇⢸⣏⣉⣉⣉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⡄⢻⣿⣿⣿⣿⣿⣿⣿⣤⣈⠹⠆⢸⡇⠀⠀⠀⠀⠀⠀⠀⣿⡇⢸⡟⠛⠛⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⣷⣤⡉⠉⣡⣼⣿⠻⣿⣿⣿⡆⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⣿⡇⢸⡿⠿⠿⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⠀⣾⣿⣿⣿⡄⠹⢿⣿⣿⠄⢸⣷⣤⣤⣤⣤⣤⣤⣤⣿⠇⣸⣷⣶⣶⣶⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢿⣦⡘⠻⠿⠛⢁⣶⣤⣤⣿⠀⢸⣧⣤⣤⣤⣤⣤⣤⣤⣤⣴⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠻⠷⣶⣦⠈⢿⣿⣿⡿⠀⣼⡏⢸⣿⠀⣿⡏⢸⣿⠁⠈⠻⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⢷⣦⣤⣥⣤⣾⠟⠀⠸⠿⠀⠿⠇⠸⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢸⣿⣿⠿⠿⣿⣶⠀⢸⣿⣿⠿⢿⣿⣦⠀⠀⠀⣼⣿⣿⣇⠀⠀⠀⣿⣿⠀⢸⣿⣷⡄⠀⢸⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢸⣿⣿⣤⣤⣿⡿⠀⢸⣿⣇⣀⣀⣿⣿⠀⠀⢰⣿⡏⢻⣿⡄⠀⠀⣿⣿⠀⢸⣿⣿⣿⣆⢸⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢸⣿⣿⠉⠉⢻⣿⡆⢸⣿⡿⠻⣿⣿⠁⠀⢀⣿⣿⣤⣼⣿⣿⡀⠀⣿⣿⠀⢸⣿⡇⠹⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠸⣿⣿⣶⣾⣿⠿⠃⢸⣿⡇⠀⠙⣿⣷⠀⣼⣿⠏⠉⠉⠹⣿⣧⠀⣿⣿⠀⢸⣿⠇⠀⠙⣿⣿⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣀⣀⣀⡀⢀⡀⠀⣀⡀⣀⣀⣀⠀⣀⣀⣀⠀⣀⣀⣀⡀⣀⠀⠀⢀⣀⣀⣀⢀⣀⠀⣀⡀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⢠⣿⠉⠙⣿⠘⣷⢀⣿⠀⣿⣭⣭⠀⣿⣉⣽⠇⣿⣭⣭⠀⣿⠀⠀⣾⡏⠉⣿⡆⣿⣠⣿⣇⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠻⠦⠾⠟⠀⠹⠿⠃⠀⠿⠤⠤⠀⠿⠈⠿⠄⠿⠇⠀⠀⠿⠶⠦⠹⠷⡴⠿⠁⠸⠿⠁⠿⠟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀";
    println!("{logo}");

    logger::init();

    match reqwest::get(app_config::crab_vault_location().to_string() + "/health").await {
        Ok(val) => {
            info!(
                "crab vault instance `{}` returned `{}` as response",
                &(app_config::crab_vault_location().to_string() + "/health"),
                val.status()
            );
        }
        Err(e) => {
            error!(
                "Cannot establish a connection to the crab vault instance `{}`, which means this instance might not be valid, details: {e}",
                &(app_config::crab_vault_location().to_string() + "/health")
            );
        }
    };

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

    let timeout_layer =
        TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(120));

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

    let listener = if app_config::server().ipv6() {
        TcpListener::bind((Ipv6Addr::UNSPECIFIED, app_config::server().port()))
            .await
            .unwrap()
    } else {
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, app_config::server().port()))
            .await
            .unwrap()
    };

    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        router.with_state(ServerState { db: Arc::new(conn) }),
    )
    .await
    .unwrap()
}
