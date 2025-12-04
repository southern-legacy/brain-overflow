use crab_vault::logger::{json::JsonLogger, pretty::PrettyLogger};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app_config;

pub fn init() {
    let logger_config = app_config::logger();
    let logger = tracing_subscriber::registry().with(
        PrettyLogger::new(logger_config.level())
            .with_ansi(logger_config.with_ansi())
            .with_file(logger_config.with_file())
            .with_target(logger_config.with_target())
            .with_thread(logger_config.with_thread()),
    );

    if logger_config.dump_path().is_some() {
        let json = JsonLogger::new(
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
