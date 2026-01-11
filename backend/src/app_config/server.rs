use serde::{Deserialize, Serialize};

use crate::{app_config::ConfigItem, error::fatal::FatalResult};

pub type ServerConfig = StaticServerConfig;

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StaticServerConfig {
    pub port: u16,
    pub ipv6: bool,
    pub location: String
}

impl Default for ServerConfig {
    #[inline]
    fn default() -> Self {
        Self {
            port: 10086,
            ipv6: false,
            location: "http://localhost:10086".to_string()
        }
    }
}

impl ConfigItem for ServerConfig {
    type RuntimeConfig = ServerConfig;

    fn into_runtime(self) -> FatalResult<Self> {
        Ok(self)
    }
}
