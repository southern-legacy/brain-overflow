use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub(super) port: Option<u16>,
    #[serde(default = "LoggerConfig::default")]
    pub(super) log: LoggerConfig,
    pub(super) ipv4_enabled: Option<bool>,
    pub(super) ipv6_enabled: Option<bool>,
    pub(super) secret_key: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggerConfig {
    pub(super) log_level: Option<String>,
    pub(super) dump_path: Option<String>,
    pub(super) with_ansi: Option<bool>,
    pub(super) with_file: Option<bool>,
    pub(super) with_target: Option<bool>,
    pub(super) with_thread: Option<bool>,
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(80)
    }

    pub fn log(&self) -> &LoggerConfig {
        &self.log
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

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_level: Some("trace".to_string()),
            dump_path: None,
            with_ansi: Some(true),
            with_file: Some(true),
            with_target: Some(true),
            with_thread: Some(true),
        }
    }
}

impl LoggerConfig {
    pub fn level(&self) -> &str {
        self.log_level.as_deref().unwrap_or("trace")
    }

    pub fn dump_path(&self) -> Option<&str> {
        match &self.dump_path {
            Some(val) => Some(val),
            None => None,
        }
    }

    pub fn with_ansi(&self) -> bool {
        self.with_ansi.unwrap_or(true)
    }

    pub fn with_file(&self) -> bool {
        self.with_file.unwrap_or(true)
    }

    pub fn with_target(&self) -> bool {
        self.with_target.unwrap_or(true)
    }

    pub fn with_thread(&self) -> bool {
        self.with_thread.unwrap_or(true)
    }
}
