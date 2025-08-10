use crate::app_config;
use sqlx::error::DatabaseError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{query, PgPool};
use std::borrow::Cow;
use std::fmt::Display;
use std::time::Duration;
use tracing::info_span;

pub async fn init() -> PgPool {
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
    
    let conn_opts = PgPoolOptions::new()
        .min_connections(db_config.min_conn())
        .max_connections(db_config.max_conn())
        .idle_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(20));

    tracing::info!("Connecting to database: {url}");
    let conn = conn_opts.connect(&url).await.unwrap();
    tracing::info!("Connection set up successfully!");

    let version = query!(r#"SELECT version()"#)
        .fetch_one(&conn)
        .await
        .map(|val|  val.version);

    match version {
        Ok(Some(version)) => tracing::info!("Database version: {}", version),
        Ok(None) => tracing::warn!("Database returned nothing after query its version"),
        Err(e) => panic!("{}", e),
    }

    return conn;
}

#[derive(Debug)]
pub enum SqlxError {
    Processible(Violation),
    Unprocessible(Cow<'static, str>),
}

impl From<sqlx::Error> for SqlxError {
    fn from(value: sqlx::Error) -> Self {
        error_handling(value)
    }
}

#[derive(Debug)]
pub enum Violation {
    Unique(Box<dyn DatabaseError>),
    Foreign(Box<dyn DatabaseError>),
    Check(Box<dyn DatabaseError>),
    NotNull(Box<dyn DatabaseError>),
    Other(Box<dyn DatabaseError>),
}

impl Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Violation::*;
        match self {
            Unique(error) => f.write_str(error.message()),
            Foreign(error) => f.write_str(error.message()),
            Check(error) => f.write_str(error.message()),
            NotNull(error) => f.write_str(error.message()),
            Other(error) => f.write_str(error.message()),
        }
    }
}

impl Display for SqlxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqlxError::Processible(e) => {
                use Violation::*;
                match e {
                    Unique(e) => f.write_fmt(format_args!("unique key violation: {e}")),
                    Foreign(e) => f.write_fmt(format_args!("foreign key violation: {e}")),
                    Check(e) => f.write_fmt(format_args!("check violation: {e}")),
                    NotNull(e) => f.write_fmt(format_args!("not null violation: {e}")),
                    Other(e) => f.write_fmt(format_args!("other database error: {e}"))
                }
            },
            SqlxError::Unprocessible(e) => f.write_str(e),
        }
    }
}

impl core::error::Error for SqlxError {}

pub fn error_handling(e: sqlx::Error) -> SqlxError {
    use sqlx::Error::*;
    use SqlxError::*;
    use Cow::*;
    match e {
        Configuration(e) |
        Encode(e) |
        Decode(e) |
        AnyDriverError(e) |
        Tls(e) => Unprocessible(Owned(e.to_string())),
        Io(e) => Unprocessible(Owned(e.to_string())),

        // 所有的其他类型
        InvalidArgument(e) => Unprocessible(Owned(e)),
        TypeNotFound { type_name } => Unprocessible(Owned(format!("Type name {type_name} not found!"))),
        ColumnDecode { index, source } => Unprocessible(Owned(format!("Cloumn Decode Error{index}, {source}"))),
        Protocol(e) => Unprocessible(Owned(e)),
        ColumnNotFound(_error) => todo!(),
        ColumnIndexOutOfBounds { index, len } => Unprocessible(Owned(format!("Column Index Out of Bounds! index: {index}, len: {len}"))),
        PoolTimedOut => Unprocessible(Borrowed("Pool Time Out, which should've been")),
        PoolClosed => Unprocessible(Borrowed("Pool Closed, which should've been")),
        RowNotFound => Unprocessible(Borrowed("Row Not Found, which should've been")),
        WorkerCrashed => Unprocessible(Borrowed("Worker Crashed, which should've been")),
        InvalidSavePointStatement => Unprocessible(Borrowed("Invalid Save Point Statement (Trigger)")),
        Migrate(e) => Unprocessible(Owned(format!("{e}"))),
        BeginFailed => Unprocessible(Borrowed("Begin Failed!")),

        Database(e) => this_should_be_processible(e),
        _ => todo!()
    };
    todo!()
}

fn this_should_be_processible(e: Box<dyn DatabaseError>) -> SqlxError {
    use sqlx::error::ErrorKind;
    use SqlxError::*;
    use Violation::*;
    match e.kind() {
        ErrorKind::UniqueViolation => Processible(Unique(e)),
        ErrorKind::ForeignKeyViolation => Processible(Foreign(e)),
        ErrorKind::NotNullViolation => Processible(NotNull(e)),
        ErrorKind::CheckViolation => Processible(Check(e)),
        ErrorKind::Other => Processible(Other(e)),
        _ => unreachable!(),
    }
}