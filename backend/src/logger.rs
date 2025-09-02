use crate::app_config;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod json;
mod pretty;

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default, ValueEnum)]
pub enum LogLevel {
    #[default]
    #[serde(alias = "trace", alias = "TRACE")]
    Trace,
    #[serde(alias = "debug", alias = "DEBUG")]
    Debug,
    #[serde(alias = "info", alias = "INFO")]
    Info,
    #[serde(alias = "warn", alias = "WARN")]
    Warn,
    #[serde(alias = "error", alias = "ERROR")]
    Error,
}

impl From<tracing::Level> for LogLevel {
    #[inline(always)]
    fn from(value: tracing::Level) -> Self {
        match value {
            tracing::Level::TRACE => Self::Trace,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::INFO => Self::Info,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::ERROR => Self::Error,
        }
    }
}

pub fn init() {
    let logger_config = app_config::logger();
    let logger = tracing_subscriber::registry().with(
        pretty::PrettyLogger::new(logger_config.level())
            .with_ansi(logger_config.with_ansi())
            .with_file(logger_config.with_file())
            .with_target(logger_config.with_target())
            .with_thread(logger_config.with_thread()),
    );

    if logger_config.dump_path().is_some() {
        let json = json::JsonLogger::new(
            logger_config.dump_path().unwrap(),
            logger_config.dump_level().unwrap(),
        );

        match json {
            Ok(json) => {
                logger
                    .with(
                        json.with_file(logger_config.with_file())
                            .with_target(logger_config.with_target())
                            .with_thread(logger_config.with_thread()),
                    )
                    .init();
            }
            Err(e) => {
                logger.init();
                tracing::error!("Cannot open the logger file! Details: {}", e);
            }
        }
    } else {
        logger.init();
    }
}
