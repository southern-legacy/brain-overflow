use crab_vault_logger::{json::JsonLogger, pretty::PrettyLogger};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app_config::logger::LoggerConfig;

pub fn init(logger_config: &LoggerConfig) {
    let logger = tracing_subscriber::registry().with(
        PrettyLogger::new(logger_config.level())
            .with_ansi(logger_config.with_ansi())
            .with_file(logger_config.with_file())
            .with_target(logger_config.with_target())
            .with_thread(logger_config.with_thread()),
    );

    if logger_config.dump_path().is_some() {
        let json = JsonLogger::new(
            logger_config.dump_path().expect("no panic"),
            logger_config.dump_level().expect("no panic"),
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
