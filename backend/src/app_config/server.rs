use crab_vault_auth::JwtConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{
    error::cli::MultiCliError,
    http::auth::{IssueConfig, JwtConfigBuilder},
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerConfig {
    pub(super) port: u16,
    pub(super) ipv6_enabled: bool,
    pub(super) auth: AuthConfig,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    /// jwt 鉴权相关设置
    #[serde(default, rename = "jwt_config")]
    jwt_config_builder: JwtConfigBuilder,

    #[serde(skip)]
    jwt_config_cache: JwtConfigCache,
}

#[derive(Default)]
pub struct JwtConfigCache {
    jwt_config: OnceCell<JwtConfig>,
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn ipv6_enabled(&self) -> bool {
        self.ipv6_enabled
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 32767,
            ipv6_enabled: false,
            auth: AuthConfig::default(),
        }
    }
}

impl AuthConfig {
    pub fn jwt_config(&self) -> impl Future<Output = &JwtConfig> {
        self.jwt_config_cache.get_inner()
    }

    pub fn iss_config(&self) -> &IssueConfig {
        self.jwt_config_builder.iss_config()
    }
}

impl JwtConfigCache {
    pub async fn get_inner(&self) -> &JwtConfig {
        self.jwt_config
            .get_or_init(async || {
                crate::app_config::server()
                    .auth()
                    .jwt_config_builder
                    .clone()
                    .build()
                    .map_err(MultiCliError::exit_now)
                    .unwrap()
            })
            .await
    }
}
