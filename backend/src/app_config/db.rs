use serde::Deserialize;
use std::cmp::{max, min};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub(super) host: Option<String>,
    pub(super) port: Option<u16>,

    #[serde(rename = "user")]
    pub(super) usr: Option<String>,

    #[serde(rename = "password")]
    pub(super) passwd: Option<String>,

    #[serde(rename = "database")]
    pub(super) db: Option<String>,

    #[serde(rename = "max_connection")]
    pub(super) max_conn: Option<u32>,

    #[serde(rename = "min_connection")]
    pub(super) min_conn: Option<u32>,
}

impl DatabaseConfig {
    #[inline]
    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("localhost")
    }

    #[inline]
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(5432)
    }

    #[inline]
    pub fn usr(&self) -> &str {
        self.usr.as_deref().unwrap_or("postgres")
    }

    #[inline]
    pub fn passwd(&self) -> &str {
        self.passwd.as_deref().expect("passwd unknown")
    }

    #[inline]
    pub fn db(&self) -> &str {
        self.db.as_deref().unwrap_or("postgres")
    }

    #[inline]
    pub fn max_conn(&self) -> u32 {
        self.max_conn
            .unwrap_or(max((num_cpus::get() * 8) as u32, 10))
    }

    #[inline]
    pub fn min_conn(&self) -> u32 {
        self.min_conn
            .unwrap_or(min((num_cpus::get() * 4) as u32, 10))
    }
}
