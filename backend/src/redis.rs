use redis::{AsyncConnectionConfig, aio::MultiplexedConnection};

use crate::app_config::AppConfig;

pub async fn init(config: &AppConfig) -> MultiplexedConnection {
    let async_conn = AsyncConnectionConfig::new();

    redis::Client::open(config.redis.url.as_str())
        .unwrap()
        .get_multiplexed_async_connection_with_config(&async_conn)
        .await
        .unwrap()
}

pub const ARTICLE_CACHE_TTL: u64 = 30 * 60;
