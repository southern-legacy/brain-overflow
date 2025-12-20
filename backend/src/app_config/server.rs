use serde::{Deserialize, Serialize};

use crate::{app_config::ConfigItem, error::fatal::FatalResult};

pub type ServerConfig = StaticServerConfig;

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(deny_unknown_fields, default)]
pub struct StaticServerConfig {
    pub(super) port: u16,
    pub(super) ipv6: bool,
}

impl ServerConfig {
    #[inline]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[inline]
    pub fn ipv6(&self) -> bool {
        self.ipv6
    }
}

impl Default for ServerConfig {
    #[inline]
    fn default() -> Self {
        Self {
            port: 32767,
            ipv6: false,
        }
    }
}

impl ConfigItem for ServerConfig {
    type RuntimeConfig = ServerConfig;

    fn into_runtime(self) -> FatalResult<Self> {
        Ok(self)
    }
}
