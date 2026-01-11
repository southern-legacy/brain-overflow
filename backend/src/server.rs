use crate::{app_config::AppConfig, cli::Cli, database, http, logger};
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
use tracing::info;

#[derive(Clone)]
pub struct ServerState {
    pub database: Arc<PgPool>,
    pub config: Arc<AppConfig>,
}

pub async fn start(cli: &Cli) {
    let config_path = cli.config_path.clone().unwrap_or_else(|| {
        std::env::home_dir()
            .map(|mut v| {
                v.push(".config/brain/brain-overflow.toml");
                v.to_string_lossy().to_string()
            })
            .unwrap_or("./brain-overflow.toml".to_string())
    });

    let config = Arc::new(AppConfig::load(&config_path).merge_cli(cli));

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

    logger::init(&config.logger);

    let database = Arc::new(database::init(&config.database).await);

    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request| {
            let method = req.method().to_string();
            let uri = req.uri().to_string();
            let req_id = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::now_v7()); // 使用 base64 编码的 uuid 作为请求 req_id
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
        .expose_headers(cors::Any)
        .max_age(Duration::from_secs(3600 * 24));

    let path_normalize_layer = NormalizePathLayer::trim_trailing_slash();

    let router = http::build_router(config.as_ref());
    let router = router
        .layer(timeout_layer)
        .layer(body_limit_layer)
        .layer(tracing_layer)
        .layer(cors_layer)
        .layer(path_normalize_layer);

    let listener = if config.server.ipv6 {
        TcpListener::bind((Ipv6Addr::UNSPECIFIED, config.server.port))
            .await
            .unwrap()
    } else {
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.server.port))
            .await
            .unwrap()
    };

    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        router.with_state(ServerState { database, config }),
    )
    .await
    .unwrap()
}
