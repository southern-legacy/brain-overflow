use crate::app_config;
use serde::Serialize;
use sqlx::error::DatabaseError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, query};
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
        .map(|val| val.version);

    match version {
        Ok(Some(version)) => tracing::info!("Database version: {}", version),
        Ok(None) => tracing::warn!("Database returned nothing after query its version"),
        Err(e) => panic!("{}", e),
    }

    return conn;
}

#[derive(Debug)]
pub enum SqlxError {
    Violation(ViolationKind),
    NotFound,
    Unprocessable(Cow<'static, str>),
}

impl From<sqlx::Error> for SqlxError {
    fn from(value: sqlx::Error) -> Self {
        error_handling(value)
    }
}

#[derive(Debug)]
pub enum ViolationKind {
    Unique(Box<dyn DatabaseError>),
    Foreign(Box<dyn DatabaseError>),
    Check(Box<dyn DatabaseError>),
    NotNull(Box<dyn DatabaseError>),
    Other(Box<dyn DatabaseError>),
}

impl Display for ViolationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ViolationKind::*;
        match self {
            Unique(error) => f.write_str(error.message()),
            Foreign(error) => f.write_str(error.message()),
            Check(error) => f.write_str(error.message()),
            NotNull(error) => f.write_str(error.message()),
            Other(error) => f.write_str(error.message()),
        }
    }
}

impl Serialize for ViolationKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ViolationKind::*;
        let (idx, val) = match self {
            Unique(_) => (0, "unique"),
            Foreign(_) => (1, "foreign"),
            Check(_) => (2, "check"),
            NotNull(_) => (3, "not_null"),
            Other(_) => (4, "other"),
        };
        serializer.serialize_newtype_variant("violation", idx, "code", val)
    }
}

impl Display for SqlxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqlxError::Violation(e) => {
                use ViolationKind::*;
                match e {
                    Unique(e) => f.write_fmt(format_args!("unique key violation: {e}")),
                    Foreign(e) => f.write_fmt(format_args!("foreign key violation: {e}")),
                    Check(e) => f.write_fmt(format_args!("check violation: {e}")),
                    NotNull(e) => f.write_fmt(format_args!("not null violation: {e}")),
                    Other(e) => f.write_fmt(format_args!("other database error: {e}")),
                }
            }
            SqlxError::NotFound => f.write_str("row not found"),
            SqlxError::Unprocessable(e) => f.write_str(e),
        }
    }
}

impl core::error::Error for SqlxError {}

pub fn error_handling(e: sqlx::Error) -> SqlxError {
    use Cow::*;
    use SqlxError::*;
    use sqlx::Error::*;
    match e {
        Configuration(e) | Encode(e) | Decode(e) | AnyDriverError(e) | Tls(e) => {
            Unprocessable(Owned(e.to_string()))
        }
        Io(e) => Unprocessable(Owned(e.to_string())),

        // 所有的其他类型
        InvalidArgument(e) => Unprocessable(Owned(e)),
        TypeNotFound { type_name } => {
            Unprocessable(Owned(format!("Type name {type_name} not found!")))
        }
        ColumnDecode { index, source } => {
            Unprocessable(Owned(format!("Cloumn Decode Error{index}, {source}")))
        }
        Protocol(e) => Unprocessable(Owned(e)),
        ColumnNotFound(_error) => todo!(),
        ColumnIndexOutOfBounds { index, len } => Unprocessable(Owned(format!(
            "Column Index Out of Bounds! index: {index}, len: {len}"
        ))),
        PoolTimedOut => Unprocessable(Borrowed("Pool Time Out, which should've been")),
        PoolClosed => Unprocessable(Borrowed("Pool Closed, which should've been")),
        WorkerCrashed => Unprocessable(Borrowed("Worker Crashed, which should've been")),
        InvalidSavePointStatement => {
            Unprocessable(Borrowed("Invalid Save Point Statement (Trigger)"))
        }
        Migrate(e) => Unprocessable(Owned(format!("{e}"))),
        BeginFailed => Unprocessable(Borrowed("Begin Failed!")),

        RowNotFound => this_should_be_processible(None),
        Database(e) => this_should_be_processible(Some(e)),
        _ => todo!(),
    }
}

fn this_should_be_processible(e: Option<Box<dyn DatabaseError>>) -> SqlxError {
    use ViolationKind::*;
    use sqlx::error::ErrorKind;
    if let Some(e) = e {
        match e.kind() {
            ErrorKind::UniqueViolation => SqlxError::Violation(Unique(e)),
            ErrorKind::ForeignKeyViolation => SqlxError::Violation(Foreign(e)),
            ErrorKind::NotNullViolation => SqlxError::Violation(NotNull(e)),
            ErrorKind::CheckViolation => SqlxError::Violation(Check(e)),
            ErrorKind::Other => SqlxError::Violation(Other(e)),
            _ => unreachable!(),
        }
    } else {
        SqlxError::NotFound
    }
}
