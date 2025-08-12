use std::{borrow::Cow, error::Error, fmt::Display};

use axum::{http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use sqlx::error::DatabaseError;

#[allow(dead_code)]
pub trait CustomError<Origin>
where Self: From<Origin> + Error, Response: From<Self> { }

#[derive(Debug)]
pub enum DbError {
    Violation(ViolationKind),
    NotFound,
    Unprocessable(Cow<'static, str>),
}

impl CustomError<sqlx::error::Error> for DbError { }
impl CustomError<jsonwebtoken::errors::Error> for AuthError { }

#[derive(Debug)]
pub enum ViolationKind {
    Unique(Box<dyn DatabaseError>),
    Foreign(Box<dyn DatabaseError>),
    Check(Box<dyn DatabaseError>),
    NotNull(Box<dyn DatabaseError>),
    Other(Box<dyn DatabaseError>),
}

#[derive(Debug)]
pub enum AuthError {
    TokenInvalid,
    TokenExpired,
}

//////////////////////////////////////
impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        use Cow::*;
        use DbError::*;
        use sqlx::Error::*;
        match value {
            Configuration(e) | Encode(e) | Decode(e) | AnyDriverError(e) | Tls(e) => {
                Unprocessable(Owned(e.to_string()))
            }
            Io(e) => Unprocessable(Owned(e.to_string())),
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

            RowNotFound => DbError::NotFound,
            Database(e) => {
                use ViolationKind::*;
                use sqlx::error::ErrorKind;
                match e.kind() {
                    ErrorKind::UniqueViolation => DbError::Violation(Unique(e)),
                    ErrorKind::ForeignKeyViolation => DbError::Violation(Foreign(e)),
                    ErrorKind::NotNullViolation => DbError::Violation(NotNull(e)),
                    ErrorKind::CheckViolation => DbError::Violation(Check(e)),
                    ErrorKind::Other => DbError::Violation(Other(e)),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Violation(e) => {
                use ViolationKind::*;
                match e {
                    Unique(e) => f.write_fmt(format_args!("unique key violation: {e}")),
                    Foreign(e) => f.write_fmt(format_args!("foreign key violation: {e}")),
                    Check(e) => f.write_fmt(format_args!("check violation: {e}")),
                    NotNull(e) => f.write_fmt(format_args!("not null violation: {e}")),
                    Other(e) => f.write_fmt(format_args!("other database error: {e}")),
                }
            }
            DbError::NotFound => f.write_str("row not found"),
            DbError::Unprocessable(e) => f.write_str(e),
        }
    }
}

impl From<DbError> for Response {
    fn from(value: DbError) -> Self {
        use DbError::*;
        match value {
            Unprocessable(e) => {
                tracing::error!("Error occurs while manipulating database! Details: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Violation(e) => {
                use ViolationKind::*;
                use tracing::warn;
                match &e {
                    Unique(error) => warn!("Unique key violation! Details: {}", error.message()),
                    Foreign(error) => warn!("Foreign key violation! Details: {}", error.message()),
                    Check(error) => warn!("Check key violation! Details: {}", error.message()),
                    NotNull(error) => warn!("Not null key violation! Details: {}", error.message()),
                    Other(error) =>  warn!("Other violation! Details: {}", error.message()),
                };
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(e)).into_response()
            }
            NotFound => StatusCode::NOT_FOUND.into_response()
        }
    }
}

impl Error for DbError { }

///////////////////////////////////////
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

///////////////////////////////////////
impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.into_kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        }
    }
}

impl From<AuthError> for Response {
    fn from(val: AuthError) -> axum::response::Response {
        match val {
            AuthError::TokenInvalid => (StatusCode::UNAUTHORIZED, r#"{"code","token_invalid"}"#).into_response(),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, r#"{"code","token_expired"}"#).into_response(),
        }
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::TokenInvalid => f.write_str("jwt error: this is invalid"),
            AuthError::TokenExpired => f.write_str("jwt error: this token has been expired"),
        }
    }
}

impl Error for AuthError { }