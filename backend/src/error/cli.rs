use std::{
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use clap::{CommandFactory, error::ErrorKind};
use crab_vault::auth::error::AuthError;

use crate::cli::Cli;

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub struct CliError {
    kind: ErrorKind,
    general_message: String,
    source: Vec<String>,
}

#[derive(Debug)]
pub struct MultiCliError {
    errors: Vec<CliError>,
}

impl MultiCliError {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn add(&mut self, error: CliError) -> &mut Self {
        self.errors.push(error);
        self
    }

    pub fn exit_now(self) -> ! {
        let mut final_message = "".to_string();
        for e in self.errors {
            final_message.push_str(&format!("\n\n{}", e.into_message()));
        }

        Cli::command().error(ErrorKind::Io, final_message).exit()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.len() == 0
    }
}

impl CliError {
    pub fn new(kind: ErrorKind, general_message: String, source: Option<String>) -> Self {
        Self {
            kind,
            general_message,
            source: match source {
                Some(val) => vec![val],
                None => vec![],
            },
        }
    }

    pub fn exit_now(self) -> ! {
        let (kind, message) = (self.kind, self.into_message());
        Cli::command().error(kind, format!("\n\n{message}")).exit()
    }

    pub fn add_source(mut self, source: String) -> Self {
        self.source.push(source);
        self
    }

    pub fn into_message(self) -> String {
        if self.source.is_empty() {
            format!("    - {}", self.general_message)
        } else {
            let mut message = format!("    - {}", self.general_message);
            for src in self.source.into_iter().rev() {
                message.push_str(&format!("\n    | {src}"))
            }
            message
        }
    }
}

impl From<ParseIntError> for CliError {
    fn from(err: ParseIntError) -> Self {
        Self::new(
            ErrorKind::InvalidValue,
            format!("cannot transfer the value to an i64 value, details: {err}"),
            None,
        )
    }
}

impl From<ParseFloatError> for CliError {
    fn from(err: ParseFloatError) -> Self {
        Self::new(
            ErrorKind::InvalidValue,
            format!("cannot transfer the value to a f64 value, details: {err}"),
            None,
        )
    }
}

impl From<ParseBoolError> for CliError {
    fn from(err: ParseBoolError) -> Self {
        Self::new(
            ErrorKind::InvalidValue,
            format!("cannot transfer the value to a bool value, details: {err}"),
            None,
        )
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::new(
            ErrorKind::Io,
            format!("io error occurred while reading configuration file, details: {err}"),
            None,
        )
    }
}

impl From<base64::DecodeError> for CliError {
    fn from(value: base64::DecodeError) -> Self {
        Self::new(ErrorKind::Io, format!("base64 error: {}", value), None)
    }
}

impl From<AuthError> for CliError {
    fn from(value: AuthError) -> Self {
        use AuthError::*;
        let (general_message, source) = match value {
            MissingAuthHeader => ("missing auth header".into(), None),
            InvalidAuthFormat => ("invalid token format".into(), None),
            InvalidToken => ("token is invalid".into(), None),
            TokenExpired => ("token expired".into(), None),
            TokenNotYetValid => ("token not yet valid".into(), None),
            InvalidSignature => ("token signature is invalid".into(), None),
            InvalidAlgorithm(alg) => (format!("cannot validate token encoded by {:?}", alg), None),
            InvalidIssuer => ("token is issued by untrusted issuer".into(), None),
            InvalidAudience => ("token has invalid audience".into(), None),
            InvalidSubject => ("subject of this token is invalid".into(), None),
            InsufficientPermissions => ("the permission is not sufficient".into(), None),
            InvalidKeyId => (format!("no such key id!"), None),
            MissingClaim(claim) => (format!("claim `{claim}` is absent"), None),
            TokenRevoked => ("this token is revoked by the server".into(), None),
            InvalidUtf8(e) => (
                format!("the token has some invalid utf-8 character, details: {e}"),
                None,
            ),
            InvalidJson(e) => (
                format!("this token cannot be deserialized, details: {e}"),
                None,
            ),
            InvalidBase64(e) => (
                format!("this token is not encoded in standard base64, details: {e}"),
                None,
            ),
            InternalError(e) => (
                format!("something wrong while handling the token, details: {e}"),
                None,
            ),
        };

        Self::new(ErrorKind::Io, general_message, source)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(value: serde_json::Error) -> Self {
        match value.classify() {
            serde_json::error::Category::Io => todo!(),
            serde_json::error::Category::Syntax => todo!(),
            serde_json::error::Category::Data => todo!(),
            serde_json::error::Category::Eof => todo!(),
        }
    }
}

impl From<config::ConfigError> for CliError {
    fn from(value: config::ConfigError) -> Self {
        match value {
            config::ConfigError::Frozen => Self::new(
                ErrorKind::Io,
                format!(
                    "Failed to read configuration file, because configuration file is frozen and no further mutations can be made."
                ),
                None,
            ),
            config::ConfigError::NotFound(e) => Self::new(
                ErrorKind::Io,
                format!(
                    "Failed to read configuration file, because configuration field `{e}` is not found"
                ),
                None,
            ),
            config::ConfigError::PathParse { cause } => Self::new(
                ErrorKind::Io,
                format!("Failed to read configuration file, because `{cause}`"),
                None,
            ),
            config::ConfigError::FileParse { uri, cause } => Self::new(
                ErrorKind::Io,
                format!(
                    "Failed to understand the configuration file `{}`, because `{cause}`",
                    uri.unwrap_or("N/A".into())
                ),
                None,
            ),
            config::ConfigError::Type {
                origin,
                unexpected,
                expected,
                key,
            } => Self::new(
                ErrorKind::Io,
                format!(
                    "Failed to understand configuration file `{}`, should be `{expected}`, but found `{unexpected}` with key {}",
                    origin.unwrap_or("N/A".into()),
                    key.unwrap_or("N/A".into())
                ),
                None,
            ),
            config::ConfigError::At { error, origin, key } => Self::new(
                ErrorKind::Io,
                format!(
                    "Failed to read configuration file, error: {error}, origin: `{}`, key: `{}` ",
                    origin.unwrap_or("N/A".into()),
                    key.unwrap_or("N/A".into())
                ),
                None,
            ),
            config::ConfigError::Message(e) => Self::new(
                ErrorKind::Io,
                format!("Failed to read configuration file, details: {e}"),
                None,
            ),
            config::ConfigError::Foreign(e) => Self::new(
                ErrorKind::Io,
                format!("Failed to read configuration file, details: {e}"),
                None,
            ),
            _ => Self::new(
                ErrorKind::Io,
                format!("Failed to read configuration file, unknown error"),
                None,
            ),
        }
    }
}

impl From<sqlx::error::Error> for CliError {
    fn from(value: sqlx::error::Error) -> Self {
        match value {
            sqlx::Error::Configuration(e) => CliError::new(
                ErrorKind::Io,
                format!("We failed to parse the database connection url, details: {e}"),
                None,
            ),
            sqlx::Error::InvalidArgument(e) => CliError::new(
                ErrorKind::Io,
                format!(
                    "One or more of the arguments to the called sqlx function was invalid, details: {e}"
                ),
                None,
            ),
            sqlx::Error::Database(e) => CliError::new(
                ErrorKind::Io,
                format!("Database returned an error message: `{e}`"),
                None,
            ),
            sqlx::Error::Io(e) => CliError::new(
                ErrorKind::Io,
                format!("Cannot communicate with the database backend, details: `{e}`"),
                None,
            ),
            sqlx::Error::Tls(e) => CliError::new(
                ErrorKind::Io,
                format!(
                    "Error occurred while attempting to establish a TLS connection to database, details: {e}"
                ),
                None,
            ),
            sqlx::Error::Protocol(e) => CliError::new(
                ErrorKind::Io,
                format!(
                    "Unexpected or invalid data encountered while communicating with the database, details `{e}`"
                ),
                None,
            ),
            _ => unreachable!("你在错误的地方使用了这个 CliError, 这些错误不应该转化为 CliError"),
        }
    }
}
