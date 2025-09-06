pub mod auth;
pub mod db;
pub mod logger;
pub mod server;

use crate::app_config::auth::AuthConfig;
use crate::app_config::logger::LoggerConfig;
use crate::cli::Cli;
use crate::error::cli::CliError;

use self::db::DatabaseConfig;
use self::server::ServerConfig;
use clap::Parser;
use config::Config;
use serde::Deserialize;
use std::sync::LazyLock;

static CONFIG: LazyLock<AppConfig> = LazyLock::new(AppConfig::load);

#[derive(Deserialize)]
struct AppConfig {
    #[serde(default)]
    server: ServerConfig, // server 配置字段

    #[serde(default)]
    logger: LoggerConfig, // logger 字段

    database: DatabaseConfig, // db 配置字段

    auth: AuthConfig,

    crab_vault_location: String
}

impl AppConfig {
    fn load() -> Self {
        let cli_conf = Cli::parse();
        let mut file_conf: AppConfig = Config::builder()
            .add_source(
                config::File::with_name("br-ovfl.toml")
                    .required(true)
                    .format(config::FileFormat::Toml),
            )
            .build()
            .map_err(|e| {
                CliError::from(e)
                    .add_source("while reading the configuration file or deserializing it".into())
                    .exit_now()
            })
            .unwrap()
            .try_deserialize()
            .map_err(|e| {
                CliError::from(e)
                    .add_source(
                        "while converting the configuration file into expected structure".into()
                    )
                    .exit_now()
            })
            .unwrap();

        if let Some(port) = cli_conf.port {
            file_conf.server.port = port
        }

        file_conf
    }
}

pub fn server() -> &'static ServerConfig {
    &CONFIG.server
}

pub fn database() -> &'static DatabaseConfig {
    &CONFIG.database
}

pub fn logger() -> &'static LoggerConfig {
    &CONFIG.logger
}

pub fn auth() -> &'static AuthConfig {
    &CONFIG.auth
}

pub fn crab_vault_location() -> &'static str {
    &CONFIG.crab_vault_location
}
