use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init() {
    tracing_subscriber::registry()
        .with(EnvFilter::new(crate::app_config::get_server().log_level()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_names(true)
                .with_thread_ids(false)
                .pretty()
        ).init()
}