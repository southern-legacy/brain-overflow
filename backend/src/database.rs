use crate::app_config::db::DatabaseConfig;
use crate::error::fatal::FatalError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, query};
use std::time::Duration;
use tracing::info_span;

pub async fn init(database_config: &DatabaseConfig) -> PgPool {
    let span = info_span!("Setting up database connection...");
    let _ = span.enter();

    let url = &database_config.url;

    let conn_opts = PgPoolOptions::new()
        .min_connections(database_config.max_connection)
        .max_connections(database_config.min_connection)
        .idle_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(20));

    tracing::info!("Connecting to database: {url}");
    let conn = conn_opts
        .connect(url)
        .await
        .map_err(|e| {
            FatalError::from(e)
                .when("while setting up database connection".into())
                .exit_now()
        })
        .unwrap();
    tracing::info!("Connection set up successfully!");

    let version = query!(r#"SELECT version()"#)
        .fetch_one(&conn)
        .await
        .map(|val| val.version);

    match version {
        Ok(Some(version)) => tracing::info!("Database version: {}", version),
        Ok(None) => tracing::warn!("Database returned nothing after query its version"),
        Err(e) => panic!("{}", e),
    }

    conn
}
