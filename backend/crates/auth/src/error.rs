use std::{string::FromUtf8Error, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::DecodeError;
use jsonwebtoken::Algorithm;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Clone, Error)]
#[serde(rename_all = "camelCase", tag = "code")]
pub enum AuthError {
    #[error("missing authorization header")]
    MissingAuthHeader,

    #[error("key of algorithm `{0:?}` is undefined")]
    InvalidAlgorithm(Algorithm),

    #[error("invalid authorization format: expected 'Bearer <token>'")]
    InvalidAuthFormat,

    #[error("no key id fed")]
    InvalidKeyId,

    #[error("Some of the text was invalid UTF-8: {0}")]
    InvalidUtf8(
        #[from]
        #[serde(skip)]
        FromUtf8Error,
    ),

    #[error("An error happened while serializing/deserializing JSON: {0}")]
    InvalidJson(
        #[from]
        #[serde(skip)]
        Arc<serde_json::Error>,
    ),

    #[error("An error happened when decoding some base64 text: {0}")]
    InvalidBase64(
        #[from]
        #[serde(skip)]
        DecodeError,
    ),

    #[error("token is invalid")]
    InvalidToken,

    #[error("token has expired")]
    TokenExpired,

    #[error("token is not yet valid")]
    TokenNotYetValid,

    #[error("invalid signature")]
    InvalidSignature,

    #[error("untrusted issuer")]
    InvalidIssuer,

    #[error("invalid audience")]
    InvalidAudience,

    #[error("invalid subject")]
    InvalidSubject,

    #[error("required claim `{0}` missing")]
    MissingClaim(String),

    #[error("token has been revoked")]
    TokenRevoked,

    #[error("internal server error during authentication, details: {0}")]
    InternalError(#[serde(skip)] String),
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind::*;

        match value.into_kind() {
            ExpiredSignature => AuthError::TokenExpired,
            InvalidSignature => AuthError::InvalidSignature,
            InvalidIssuer => AuthError::InvalidIssuer,
            InvalidAudience => AuthError::InvalidAudience,
            InvalidSubject => AuthError::InvalidSubject,
            MissingRequiredClaim(claim) => AuthError::MissingClaim(claim),
            ImmatureSignature => AuthError::TokenNotYetValid,
            InvalidToken => AuthError::InvalidToken,
            Base64(e) => AuthError::InvalidBase64(e),
            Utf8(e) => AuthError::InvalidUtf8(e),
            Json(e) => AuthError::InvalidJson(e),

            InvalidEcdsaKey => AuthError::InternalError("the secret given is not a valid ECDSA key".into()),
            InvalidRsaKey(_) => AuthError::InternalError("the secret given is not a valid RSA key".into()),
            RsaFailedSigning => AuthError::InternalError("could not sign with the given key".into()),
            InvalidAlgorithmName => AuthError::InternalError("cannot parse algorithm from str".into()),
            InvalidKeyFormat => AuthError::InternalError("a key is provided with an invalid format".into()),
            InvalidAlgorithm => AuthError::InternalError("the algorithm in the header doesn't match the one passed to decode or the encoding/decoding key used doesn't match the alg requested".to_string()),
            MissingAlgorithm => AuthError::InternalError("the Validation struct does not contain at least 1 algorithm".into()),
            Crypto(e) => AuthError::InternalError(format!("Something unspecified went wrong with crypto: {e}")),
            _ => todo!()
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AuthError::MissingAuthHeader
            | AuthError::InvalidKeyId
            | AuthError::InvalidAuthFormat
            | AuthError::InvalidToken
            | AuthError::TokenExpired
            | AuthError::TokenNotYetValid
            | AuthError::InvalidAlgorithm(_)
            | AuthError::InvalidSignature
            | AuthError::InvalidIssuer
            | AuthError::InvalidAudience
            | AuthError::InvalidSubject
            | AuthError::MissingClaim(_)
            | AuthError::InvalidUtf8(_)
            | AuthError::InvalidJson(_)
            | AuthError::InvalidBase64(_)
            | AuthError::TokenRevoked => StatusCode::UNAUTHORIZED,

            // AuthError::InsufficientPermissions => StatusCode::FORBIDDEN,

            AuthError::InternalError(_) => StatusCode::UNAUTHORIZED,
        };

        status_code.into_response()
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidJson(Arc::new(value))
    }
}

impl From<AuthError> for Response {
    #[inline(always)]
    fn from(val: AuthError) -> Response {
        val.into_response()
    }
}
