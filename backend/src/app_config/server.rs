use std::collections::HashSet;

use crab_vault_auth::HttpMethod;
use glob::Pattern;
use serde::{Deserialize, Serialize};

use crate::http::auth::{JwtConfigBuilder, PathRule};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerConfig {
    pub(super) port: u16,
    pub(super) ipv4_enabled: bool,
    pub(super) ipv6_enabled: bool,
    pub(super) auth: AuthConfig,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    /// 这里使用 Vec
    ///
    /// 在编译规则时保证如果同一个路径下有多种公开方式时，采取最后指定的公开请求方法而非并集
    #[serde(default)]
    pub(super) path_rules: Vec<PathRule>,

    /// jwt 鉴权相关设置
    #[serde(default)]
    pub(super) jwt_config: JwtConfigBuilder,
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn ipv4_enabled(&self) -> bool {
        self.ipv4_enabled
    }

    pub fn ipv6_enabled(&self) -> bool {
        self.ipv6_enabled
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

impl AuthConfig {
    pub fn jwt_config_builder(&self) -> &JwtConfigBuilder {
        &self.jwt_config
    }

    /// 这个操作相当昂贵，应当尽少使用
    pub fn get_compiled_path_rules(&self) -> Vec<(Pattern, HashSet<HttpMethod>)> {
        self.path_rules
            .iter()
            .cloned()
            .filter_map(|rule| rule.compile())
            .collect()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 32767,
            ipv4_enabled: true,
            ipv6_enabled: false,
            auth: AuthConfig::default(),
        }
    }
}
