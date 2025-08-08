use std::cmp::{max, min};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DatabaseConfig {
    host: Option<String>,
    port: Option<u16>,
    usr: Option<String>,
    passwd: Option<String>,

    #[serde(rename = "database")]
    db: Option<String>,

    #[serde(rename = "max_connection")]
    max_conn: Option<u32>,

    #[serde(rename = "min_connection")]
    min_conn: Option<u32>,

    log_sql: Option<bool>,
}

impl DatabaseConfig {
    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("localhost")
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(5432)
    }

    pub fn usr(&self) -> &str {
        self.usr.as_deref().unwrap_or("postgres")
    }

    pub fn passwd(&self) -> &str {
        self.passwd.as_deref().expect("passwd unknown")
    }

    pub fn db(&self) -> &str {
        self.db.as_deref().unwrap_or("postgres")
    }

    pub fn max_conn(&self) -> u32 {
        self.max_conn.unwrap_or(max((num_cpus::get() * 8) as u32, 10))
    }
    
    pub fn min_conn(&self) -> u32 {
        self.min_conn.unwrap_or(min((num_cpus::get() * 4) as u32, 10))
    }

    pub fn log_sql(&self) -> bool {
        self.log_sql.unwrap_or(false)
    }
}
