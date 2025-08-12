use std::{borrow::Cow, fmt::Display};

use serde::Serialize;
use sqlx::error::DatabaseError;


#[derive(Debug)]
pub enum DbError {
    Violation(ViolationKind),
    NotFound,
    Unprocessable(Cow<'static, str>),
}

#[derive(Debug)]
pub enum ViolationKind {
    Unique(Box<dyn DatabaseError>),
    Foreign(Box<dyn DatabaseError>),
    Check(Box<dyn DatabaseError>),
    NotNull(Box<dyn DatabaseError>),
    Other(Box<dyn DatabaseError>),
}

#[derive(Debug, Serialize)]
pub enum AuthError {
    InvalidToken,
    Expired,
}

impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        error_handling(value)
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

impl core::error::Error for DbError {}

pub fn error_handling(e: sqlx::Error) -> DbError {
    use Cow::*;
    use DbError::*;
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

fn this_should_be_processible(e: Option<Box<dyn DatabaseError>>) -> DbError {
    use ViolationKind::*;
    use sqlx::error::ErrorKind;
    if let Some(e) = e {
        match e.kind() {
            ErrorKind::UniqueViolation => DbError::Violation(Unique(e)),
            ErrorKind::ForeignKeyViolation => DbError::Violation(Foreign(e)),
            ErrorKind::NotNullViolation => DbError::Violation(NotNull(e)),
            ErrorKind::CheckViolation => DbError::Violation(Check(e)),
            ErrorKind::Other => DbError::Violation(Other(e)),
            _ => unreachable!(),
        }
    } else {
        DbError::NotFound
    }
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

impl core::error::Error for ViolationKind {}

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

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.into_kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
            _ => AuthError::InvalidToken,
        }
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => f.write_str("jwt error: this is invalid"),
            AuthError::Expired => f.write_str("jwt error: this token has been expired"),
        }
    }
}