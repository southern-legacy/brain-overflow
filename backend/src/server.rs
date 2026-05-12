#![forbid(unsafe_code)]

use crate::{app_config::AppConfig, cli::Cli, database, http, logger, redis};
use ::http::StatusCode;
use ::redis::{AsyncCommands, aio::MultiplexedConnection};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client as S3Client, config::Credentials};
use axum::extract::{DefaultBodyLimit, Request};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use lettre::{Message, SmtpTransport, Transport, message::MessageBuilder};
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

/// 相当便宜的克隆，懒加载获取里面的字段
#[derive(Clone)]
pub struct ServerState(Arc<ServerStateImpl>);

/// 本身就是一个引用类型，可以廉价的 clone，但是里面的字段似乎有点多了，大多数时候都不需要全部克隆
struct ServerStateImpl {
    pub config: Arc<AppConfig>,
    pub smtp_transport: SmtpTransport,
    pub database: PgPool,
    pub redis: MultiplexedConnection,
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

    // Redis 连接
    let mut redis = redis::init(&config).await;
    let _: Result<(), _> = redis.set("brain-overflow-running", true).await;

    // smtp 邮箱连接
    let smtp_transport = SmtpTransport::relay(&config.email.smtp_addr)
        .unwrap()
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            config.email.from.email.to_string(),
            config.email.password.clone(),
        ))
        .build();

    // S3 客户端
    let s3_client = S3Client::from_conf({
        let s3_config = &config.s3;
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_sdk_s3::config::Region::new(s3_config.region.clone()));

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
                let mth = req.method().to_string();
                let uri = req.uri().to_string();
                let id = BASE64_STANDARD_NO_PAD.encode(rand::random::<[u8; 8]>());
                tracing::info_span!("[rqst]", id, mth, uri)
            })
            .on_failure(())
            .on_request(DefaultOnRequest::new().level(tracing::Level::DEBUG))
            .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG));

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

        http::build_router(ServerState(Arc::new(ServerStateImpl {
            config,
            database,
            smtp_transport,
            redis,
            s3_client,
        })))
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

impl ServerState {
    #[inline]
    pub fn redis(&self) -> MultiplexedConnection {
        self.0.redis.clone()
    }

    #[inline]
    pub fn database(&self) -> PgPool {
        self.0.database.clone()
    }

    #[inline]
    pub fn smtp_transport(&self) -> SmtpTransport {
        self.0.smtp_transport.clone()
    }

    #[inline]
    pub fn s3_client(&self) -> S3Client {
        self.0.s3_client.clone()
    }

    #[inline]
    pub fn config(&self) -> Arc<AppConfig> {
        self.0.config.clone()
    }

    /// uri 应该以 `/` 开头
    #[inline]
    pub fn prefix_uri(&self, uri: impl std::fmt::Display) -> String {
        format!("{}{}", self.0.config.server.location, uri)
    }

    #[inline]
    pub async fn begin_transaction(&self) -> crate::error::db::DbResult<sqlx::PgTransaction<'static>> {
        use crate::error::db::DbError;
        self.0.database.begin().await.map_err(DbError::from)
    }

    /// fill 在调用时会被填入一个 `from` 被填好的 [`MessageBuilder`]
    pub fn email_to(&self, fill: impl FnOnce(MessageBuilder) -> Message + Send + 'static) {
        let mailer = self.smtp_transport();
        let from = self.0.config.email.from.clone();
        tokio::spawn(async move { mailer.send(&fill(Message::builder().from(from))) });
    }
}
