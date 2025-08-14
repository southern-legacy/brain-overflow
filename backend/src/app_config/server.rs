use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub(super) port: Option<u16>,
    pub(super) log_level: Option<String>,
    pub(super) ipv4_enabled: Option<bool>,
    pub(super) ipv6_enabled: Option<bool>,
    pub(super) secret_key: Option<String>,
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(80)
    }

    pub fn log_level(&self) -> &str {
        self.log_level.as_deref().unwrap_or("info")
    }

    pub fn ipv4_enabled(&self) -> bool {
        self.ipv4_enabled.unwrap_or(true)
    }

    pub fn ipv6_enabled(&self) -> bool {
        self.ipv6_enabled.unwrap_or(false)
    }

    pub fn secret_key(&self) -> &str {
        // 默认值为 "Brain Overflow, designed neither for merchant, nor for fortune, but for cultivating skills, by Southern Lagecy, under MIT lisence."
        self.secret_key
            .as_deref()
            .unwrap_or("QnJhaW4gT3ZlcmZsb3csIGRlc2lnbmVkIG5laXRoZXIgZm9yIG1lcmNoYW50LCBub3IgZm9yIGZvcnR1bmUsIGJ1dCBmb3IgY3VsdGl2YXRpbmcgc2tpbGxzLCBieSBTb3V0aGVybiBMYWdlY3ksIHVuZGVyIE1JVCBsaXNlbmNlLg==")
    }
}
