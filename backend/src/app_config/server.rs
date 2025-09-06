use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerConfig {
    pub(super) port: u16,
    pub(super) ipv6_enabled: bool,
}

impl ServerConfig {
    #[inline]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[inline]
    pub fn ipv6_enabled(&self) -> bool {
        self.ipv6_enabled
    }
}

impl Default for ServerConfig {
    #[inline]
    fn default() -> Self {
        Self {
            port: 32767,
            ipv6_enabled: false,
        }
    }
}
