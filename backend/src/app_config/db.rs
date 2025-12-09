use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

use crate::{
    app_config::ConfigItem,
    error::fatal::FatalResult,
};

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct DatabaseConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
    max_connection: u32,
    min_connection: u32,
}

pub struct RuntimeDatabaseConfig {
    pub url: String,
    pub max_connection: u32,
    pub min_connection: u32,
}

impl ConfigItem for DatabaseConfig {
    type RuntimeConfig = RuntimeDatabaseConfig;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        let DatabaseConfig {
            host,
            port,
            user,
            password,
            database,
            max_connection,
            min_connection,
        } = self;

        Ok(RuntimeDatabaseConfig {
            url: format!("postgres://{user}:{password}@{host}:{port}/{database}"),
            max_connection,
            min_connection,
        })
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "locahost".into(),
            port: 5432,
            user: "postgres".into(),
            password: "passwd unknown".into(),
            database: "postgres".into(),
            max_connection: max((num_cpus::get() * 8) as u32, 10),
            min_connection: min((num_cpus::get() * 4) as u32, 10),
        }
    }
}
