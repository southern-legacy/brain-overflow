use crate::{app_config::AppConfig, cli::Cli, database, http, logger};
use ::http::StatusCode;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client as S3Client, config::Credentials};
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

/// 本身就是一个引用类型，可以廉价的 clone
#[derive(Clone)]
pub struct ServerState {
    pub database: PgPool,
    pub config: Arc<AppConfig>,
    pub s3_client: S3Client,
}

pub async fn start(cli: &Cli) {
    let config_path = cli.config_path.clone().unwrap_or("./brain-overflow.toml".to_string());

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

    // 数据库连接
    let database = database::init(&config.database).await;

    // S3 客户端
    let s3_client = S3Client::from_conf({
        let s3_config = &config.s3;
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest()).region(aws_sdk_s3::config::Region::new(s3_config.region.clone()));

        // 设置 credential
        if let Some(access_key_id) = &s3_config.access_key_id
            && let Some(secret_access_key) = &s3_config.secret_access_key
        {
            let credentials = Credentials::new(
                access_key_id,
                secret_access_key,
                None, // session token
                None, // expiration
                "static",
            );
            config_loader = config_loader.credentials_provider(credentials);
        } else {
            tracing::warn!("No aws s3 credential provided, cannot generate presign url")
        }

        // Configure endpoint if provided
        if let Some(endpoint) = &s3_config.endpoint {
            config_loader = config_loader.endpoint_url(endpoint);
        }

        aws_sdk_s3::config::Builder::from(&config_loader.load().await)
            .force_path_style(s3_config.force_path_style)
            .build()
    });

    let (ipv6, port) = (config.server.ipv6, config.server.port);

    // 路由和通用中间件
    let router = {
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

        let timeout_layer = TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(120));

        let body_limit_layer = DefaultBodyLimit::max((1024 * 1024 * 16) as usize); // 16 MB 的最大报文大小

        let cors_layer = CorsLayer::new()
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_origin(cors::Any)
            .allow_credentials(false)
            .expose_headers(cors::Any)
            .max_age(Duration::from_secs(3600 * 24));

        let path_normalize_layer = NormalizePathLayer::trim_trailing_slash();

        http::build_router(ServerState { database, config, s3_client })
            .layer(timeout_layer)
            .layer(body_limit_layer)
            .layer(tracing_layer)
            .layer(cors_layer)
            .layer(path_normalize_layer)
    };

    let listener = if ipv6 {
        TcpListener::bind((Ipv6Addr::UNSPECIFIED, port)).await.unwrap()
    } else {
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await.unwrap()
    };

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap()
}
