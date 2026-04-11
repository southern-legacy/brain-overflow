use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

use crate::{app_config::ConfigItem, error::fatal::FatalResult};

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StaticDatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connection: u32,
    pub min_connection: u32,
}

pub type DatabaseConfig = StaticDatabaseConfig;

impl ConfigItem for StaticDatabaseConfig {
    type RuntimeConfig = StaticDatabaseConfig;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        Ok(self)
    }
}

impl Default for StaticDatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 5432,
            username: "postgres".into(),
            password: "password unknown".into(),
            database: "postgres".into(),
            max_connection: max((num_cpus::get() * 8) as u32, 10),
            min_connection: min((num_cpus::get() * 4) as u32, 10),
        }
    }
}
