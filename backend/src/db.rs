use crate::app_config;
use sea_orm::{ConnectOptions, ConnectionTrait, DbBackend, Statement};
use std::time::Duration;
use tracing::info_span;
use tracing::log::info;

pub async fn init() -> sea_orm::DbConn {
    let span = info_span!("Setting up database connection...");
    let _ = span.enter();

    let db_config = app_config::get_database();
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config.usr(),
        db_config.passwd(),
        db_config.host(),
        db_config.port(),
        db_config.db()
    );
    tracing::info!("Connecting to database: {url}");

    let mut conn_opts = ConnectOptions::new(url);

    conn_opts
        .min_connections(db_config.min_conn())
        .max_connections(db_config.max_conn())
        .sqlx_logging(db_config.log_sql())
        .connect_timeout(Duration::from_secs(20))
        .acquire_timeout(Duration::from_secs(20))
        .idle_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(300));

    let conn = sea_orm::Database::connect(conn_opts).await.unwrap();

    let version = conn
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            String::from("SELECT version()"),
        ))
        .await
        .unwrap()
        .ok_or("Cannot get the version of database".to_string());

    match version {
        Ok(version) => info!(
            "Database version: {}",
            version.try_get_by_index::<String>(0).unwrap()
        ),
        Err(e) => panic!("{}", e),
    }

    return conn;
}

// pub fn error_handling(e: DbErr) -> Cow<'static, str> {
//     match e {
//         DbErr::Exec(RuntimeErr::SqlxError(error)) => match error {
//             sqlx::Error::Database(e) => {
//                 assert_eq!(e.code().unwrap(), "23000");
//             }
//             _ => tracing::error!("Unexpected sqlx::Error kind"),
//         },
//         _ => tracing::error!("Database error! details: {e}"),
//     }
//     todo!()
// }
