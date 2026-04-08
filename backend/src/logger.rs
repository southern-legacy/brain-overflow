use crate::app_config::logger::LoggerConfig;

pub fn init(logger_config: &LoggerConfig) {
    tracing_subscriber::fmt()
        .with_ansi(logger_config.with_ansi())
        .with_file(logger_config.with_file())
        .with_target(logger_config.with_target())
        .with_thread_ids(logger_config.with_thread())
        .with_thread_names(logger_config.with_thread())
        .with_max_level(logger_config.level())
        .init();
}
