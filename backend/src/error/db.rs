use std::{borrow::Cow, error::Error, fmt::Display};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::error::DatabaseError;

use crate::error::CustomError;

#[derive(Debug, Serialize)]
pub struct DbError {
    #[serde(rename = "code")]
    kind: DbErrorKind,
}

impl CustomError for DbError {
    type Kind = DbErrorKind;

    #[inline(always)]
    fn kind(&self) -> &Self::Kind {
        &self.kind
    }

    #[inline(always)]
    fn new(kind: Self::Kind) -> Self {
        Self { kind }
    }
}

impl DbError {
    pub fn is_not_found(&self) -> bool {
        match self.kind() {
            DbErrorKind::NotFound => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum DbErrorKind {
    Unique(Box<dyn DatabaseError>),
    Foreign(Box<dyn DatabaseError>),
    Check(Box<dyn DatabaseError>),
    NotNull(Box<dyn DatabaseError>),
    Other(Box<dyn DatabaseError>),
    NotFound,
    Unprocessable(Cow<'static, str>),
}

impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        use Cow::*;
        use DbErrorKind::*;
        use sqlx::Error::*;
        match value {
            Configuration(e) | Encode(e) | Decode(e) | AnyDriverError(e) | Tls(e) => {
                DbError::new(Unprocessable(Owned(e.to_string())))
            }
            Io(e) => DbError::new(Unprocessable(Owned(e.to_string()))),
            InvalidArgument(e) => DbError::new(Unprocessable(Owned(e))),
            TypeNotFound { type_name } => DbError::new(Unprocessable(Owned(format!(
                "Type name {type_name} not found!"
            )))),
            ColumnDecode { index, source } => DbError::new(Unprocessable(Owned(format!(
                "Cloumn Decode Error{index}, {source}"
            )))),
            Protocol(e) => DbError::new(Unprocessable(Owned(e))),
            ColumnNotFound(_error) => todo!(),
            ColumnIndexOutOfBounds { index, len } => DbError::new(Unprocessable(Owned(format!(
                "Column Index Out of Bounds! index: {index}, len: {len}"
            )))),
            PoolTimedOut => DbError::new(Unprocessable(Borrowed(
                "Pool Time Out, which shouldn't have been",
            ))),
            PoolClosed => {
                DbError::new(Unprocessable(Borrowed("Pool Closed, which shouldn't have been")))
            }
            WorkerCrashed => DbError::new(Unprocessable(Borrowed(
                "Worker Crashed, which shouldn't have been",
            ))),
            InvalidSavePointStatement => DbError::new(Unprocessable(Borrowed(
                "Invalid Save Point Statement (Trigger)",
            ))),
            Migrate(e) => DbError::new(Unprocessable(Owned(format!("{e}")))),
            BeginFailed => DbError::new(Unprocessable(Borrowed("Begin Failed!"))),

            RowNotFound => DbError::new(NotFound),
            Database(e) => {
                use sqlx::error::ErrorKind;
                match e.kind() {
                    ErrorKind::UniqueViolation => DbError::new(Unique(e)),
                    ErrorKind::ForeignKeyViolation => DbError::new(Foreign(e)),
                    ErrorKind::NotNullViolation => DbError::new(NotNull(e)),
                    ErrorKind::CheckViolation => DbError::new(Check(e)),
                    ErrorKind::Other => DbError::new(Other(e)),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            DbErrorKind::Unique(e) => f.write_fmt(format_args!("unique key violation: {e}")),
            DbErrorKind::Foreign(e) => f.write_fmt(format_args!("foreign key violation: {e}")),
            DbErrorKind::Check(e) => f.write_fmt(format_args!("check violation: {e}")),
            DbErrorKind::NotNull(e) => f.write_fmt(format_args!("not null violation: {e}")),
            DbErrorKind::Other(e) => f.write_fmt(format_args!("other database error: {e}")),
            DbErrorKind::NotFound => f.write_str("row not found"),
            DbErrorKind::Unprocessable(e) => f.write_str(e),
        }
    }
}

impl From<DbError> for Response {
    fn from(value: DbError) -> Self {
        use DbErrorKind::*;
        use tracing::{warn, error};
        match value.kind() {
            Unprocessable(e) => {
                error!("Error occurs while manipulating database! Details: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Unique(error) => {
                warn!("Unique key violation! Details: {}", error.message());
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(value)).into_response()
            },
            Foreign(error) => {
                warn!("Foreign key violation! Details: {}", error.message());
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(value)).into_response()
            },
            Check(error) => {
                warn!("Check key violation! Details: {}", error.message());
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(value)).into_response()
            },
            NotNull(error) => {
                warn!("Not null key violation! Details: {}", error.message());
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(value)).into_response()
            },
            Other(error) => {
                warn!("Other violation! Details: {}", error.message());
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(value)).into_response()
            }
            NotFound => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl IntoResponse for DbError {
    #[inline(always)]
    fn into_response(self) -> Response {
        Response::from(self)
    }
}

impl Error for DbError {}

impl Display for DbErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DbErrorKind::*;
        match self {
            Check(e) | Other(e) | Unique(e) | Foreign(e) | NotNull(e) => f.write_str(e.message()),
            Unprocessable(e) => f.write_str(e),
            NotFound => f.write_str("specified row not found")
        }
    }
}

impl Serialize for DbErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use DbErrorKind::*;
        let (idx, val) = match self {
            Unique(_) => (0, "unique"),
            Foreign(_) => (1, "foreign"),
            Check(_) => (2, "check"),
            NotNull(_) => (3, "notNull"),
            Other(_) => (4, "other"),
            NotFound => (5, "notFound"),
            Unprocessable(_) => (6, "unprocessable")
        };
        serializer.serialize_unit_variant("violationKind", idx, val)
    }
}
