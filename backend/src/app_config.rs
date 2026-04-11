pub mod auth;
pub mod db;
pub mod logger;
pub mod redis;
pub mod s3;
pub mod server;
pub mod util;

use crate::{
    app_config::{
        auth::{AuthConfig, StaticAuthConfig}, db::DatabaseConfig, logger::{LoggerConfig, StaticLoggerConfig}, redis::RedisConfig, s3::{S3Config, StaticS3Config}, server::{ServerConfig, StaticServerConfig}
    },
    cli::Cli,
    error::fatal::{FatalError, FatalResult, MultiFatalError},
};

use self::db::StaticDatabaseConfig;
use config::Config;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
struct StaticAppConfig {
    #[serde(default)]
    server: StaticServerConfig, // server 配置字段

    #[serde(default)]
    logger: StaticLoggerConfig, // logger 字段

    database: StaticDatabaseConfig, // db 配置字段

    auth: StaticAuthConfig,
    redis: RedisConfig,

    #[serde(default)]
    s3: StaticS3Config,
}

pub struct AppConfig {
    pub server: ServerConfig,
    pub logger: LoggerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub redis: RedisConfig,
    pub s3: S3Config,
}

/// [`ConfigItem`] 表示一个配置项，实现了这个 trait 的结构就是一个配置项
///
/// 一个配置项必须能够转化为某一个 `Self::RuntimeConfig`，这些能够直接在 runtime 获取
///
/// 类似于某一个 cache 之类的概念
///
/// 在这个转换过程中，可能会出现不同的、大量的错误，我们使用 [`MultiFatalError`](crate::error::fatal::MultiFatalError) 表示
pub trait ConfigItem
where
    Self: for<'de> Deserialize<'de> + Clone + Sized + Default,
{
    type RuntimeConfig;
    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig>;
}

impl ConfigItem for StaticAppConfig {
    type RuntimeConfig = AppConfig;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        let Self {
            server,
            logger,
            database,
            redis,
            auth,
            s3,
        } = self;

        let (database_res, auth_res, s3_res) = (database.into_runtime(), auth.into_runtime(), s3.into_runtime());

        let mut errors = MultiFatalError::new();

        // 这里这么写好看一些
        #[allow(clippy::unnecessary_unwrap)]
        if database_res.is_ok() && auth_res.is_ok() && s3_res.is_ok() {
            // unwrap safety: 全部在上面进行了 is_ok 检查
            Ok(AppConfig {
                server,
                logger,
                redis,
                database: database_res.unwrap(),
                auth: auth_res.unwrap(),
                s3: s3_res.unwrap(),
            })
        } else {
            let _ = database_res.map_err(|mut e| errors.append(&mut e));
            let _ = auth_res.map_err(|mut e| errors.append(&mut e));
            let _ = s3_res.map_err(|mut e| errors.append(&mut e));

            errors.exit_now()
        }
    }
}

impl AppConfig {
    pub fn load(path: &str) -> Self {
        let static_config: StaticAppConfig = Config::builder()
            .add_source(config::File::with_name(path).required(true).format(config::FileFormat::Toml))
            .build()
            .unwrap_or_else(|e| FatalError::from(e).when("while reading the configuration file".into()).exit_now())
            .try_deserialize()
            .unwrap_or_else(|e| FatalError::from(e).when("while deserializing the configuration file".into()).exit_now());

        static_config.into_runtime().map_err(|e| e.exit_now()).unwrap()
    }

    pub fn merge_cli(mut self, cli: &Cli) -> Self {
        self.server.port = cli.port;
        self
    }
}
