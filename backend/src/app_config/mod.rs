mod db;
mod server;

use self::db::DatabaseConfig;
use self::server::ServerConfig;
use config::Config;
use serde::Deserialize;
use std::sync::LazyLock;

static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::load());

#[derive(Deserialize)]
pub struct AppConfig {
    server: ServerConfig, // server 配置字段

    #[serde(rename = "database")]
    db: DatabaseConfig, // db 配置字段
}

impl AppConfig {
    pub fn load() -> Self {
        let configuration: AppConfig = Config::builder()
            .add_source(
                // 从配置文件读取信息
                config::File::with_name("br-ovfl.toml")
                    .required(true)
                    .format(config::FileFormat::Toml),
            )
            .build() // 启动 I/O 读取配置文件（可能出错）
            .map_err(|e| {
                println!("读取配置时出错: {e}");
            })
            .unwrap()
            .try_deserialize() // 解析配置文件（可能出错）
            .map_err(|e| {
                println!("无法反序列化配置文件: {e}");
            })
            .unwrap();

        let server_config = &configuration.server;
        if !server_config.ipv6_enabled() ^ server_config.ipv4_enabled() {
            panic!("无法同时支持 IPv4 和 IPv6 监听.")
        }
        configuration
    }
}

pub fn get_server() -> &'static ServerConfig {
    &CONFIG.server
}

pub fn get_database() -> &'static DatabaseConfig {
    &CONFIG.db
}
