use crab_vault_auth::JwtConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::{error::cli::MultiCliError, http::auth::{IssueConfig, JwtConfigBuilder}};

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    #[serde(default, rename = "jwt_config")]
    jwt_config_builder: JwtConfigBuilder,

    #[serde(skip)]
    jwt_config_cache: JwtConfigCache,
}

#[derive(Default)]
pub struct JwtConfigCache(OnceCell<JwtConfig>);

impl AuthConfig {
    pub fn jwt_config(&self) -> impl Future<Output = &JwtConfig> {
        self.jwt_config_cache.0.get_or_init(async || {
            crate::app_config::auth()
                .jwt_config_builder
                .clone()
                .build()
                .map_err(MultiCliError::exit_now)
                .unwrap()
        })
    }

    #[inline]
    pub fn iss_config(&self) -> &IssueConfig {
        self.jwt_config_builder.iss_config()
    }
}
