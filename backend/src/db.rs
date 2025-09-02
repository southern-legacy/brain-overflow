use crate::app_config;
use crate::error::cli::CliError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, query};
use std::time::Duration;
use tracing::info_span;

pub async fn init() -> PgPool {
    let span = info_span!("Setting up database connection...");
    let _ = span.enter();

    let db_config = app_config::database();
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config.usr(),
        db_config.passwd(),
        db_config.host(),
        db_config.port(),
        db_config.db()
    );

    let conn_opts = PgPoolOptions::new()
        .min_connections(db_config.min_conn())
        .max_connections(db_config.max_conn())
        .idle_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(20));

    tracing::info!("Connecting to database: {url}");
    let conn = conn_opts
        .connect(&url)
        .await
        .map_err(|e| CliError::from(e).add_source("while setting up database connection".into()).exit_now())
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
