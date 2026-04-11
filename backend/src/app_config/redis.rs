use clap::error::ErrorKind;
use serde::Deserialize;

use crate::{
    app_config::ConfigItem,
    error::fatal::{FatalError, FatalResult, MultiFatalError},
};

#[derive(Clone, Default, Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

impl ConfigItem for RedisConfig {
    type RuntimeConfig = RedisConfig;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        if self.url.is_empty() {
            let mut error = MultiFatalError::new();
            error.push(FatalError::new(
                ErrorKind::Io,
                "You must specify a URL pointing to redis".into(),
                Some("".into()),
            ));
            Err(error)
        } else {
            Ok(self)
        }
    }
}
