pub mod auth;
pub mod crab_vault;
pub mod db;
pub mod logger;
pub mod server;
pub mod utils;

use crate::app_config::auth::{StaticAuthConfig, AuthConfig};
use crate::app_config::crab_vault::{StaticCrabVaultConfig, CrabVaultConfig};
use crate::app_config::db::DatabaseConfig;
use crate::app_config::logger::{StaticLoggerConfig, LoggerConfig};
use crate::app_config::server::{ServerConfig, StaticServerConfig};
use crate::cli::Cli;
use crate::error::fatal::{FatalError, FatalResult, MultiFatalError};

use self::db::StaticDatabaseConfig;
use clap::Parser;
use config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
struct StaticAppConfig {
    #[serde(default)]
    server: StaticServerConfig, // server 配置字段

    #[serde(default)]
    logger: StaticLoggerConfig, // logger 字段

    database: StaticDatabaseConfig, // db 配置字段

    auth: StaticAuthConfig,

    crab_vault: StaticCrabVaultConfig,
}

pub struct AppConfig {
    pub server: ServerConfig,
    pub logger: LoggerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub crab_vault: CrabVaultConfig,
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

impl AppConfig {
    pub fn load(file_name: &str) -> Self {
        let cli_conf = Cli::parse();

        let StaticAppConfig {
            mut server,
            logger,
            database,
            auth,
            crab_vault,
        } = Config::builder()
            .add_source(
                config::File::with_name(file_name)
                    .required(true)
                    .format(config::FileFormat::Toml),
            )
            .build()
            .and_then(|v| v.try_deserialize())
            .unwrap_or_else(|e| {
                FatalError::from(e)
                    .when("while reading the configuration file or deserializing it".into())
                    .exit_now()
            });

        if let Some(port) = cli_conf.port {
            server.port = port;
        }

        let (database_res, auth_res, crab_vault_res) = (
            database.into_runtime(),
            auth.into_runtime(),
            crab_vault.into_runtime(),
        );

        let mut errors = MultiFatalError::new();

        // 这里这么写好看一些
        #[allow(clippy::unnecessary_unwrap)]
        if database_res.is_ok() && auth_res.is_ok() && crab_vault_res.is_ok() {
            // unwrap safety: 全部在上面进行了 is_ok 检查
            AppConfig {
                server,
                logger,
                database: database_res.unwrap(),
                auth: auth_res.unwrap(),
                crab_vault: crab_vault_res.unwrap(),
            }
        } else {
            let _ = database_res.map_err(|mut e| errors.append(&mut e));
            let _ = auth_res.map_err(|mut e| errors.append(&mut e));
            let _ = crab_vault_res.map_err(|mut e| errors.append(&mut e));

            errors.exit_now()
        }
    }
}
