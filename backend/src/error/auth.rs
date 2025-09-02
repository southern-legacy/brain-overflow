use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::error::CustomError;

#[derive(Debug, Serialize)]
pub struct AuthError {
    #[serde(rename = "code")]
    kind: AuthErrorKind,
}

impl CustomError for AuthError {
    type Kind = AuthErrorKind;

    #[inline(always)]
    fn kind(&self) -> &AuthErrorKind {
        &self.kind
    }

    #[inline(always)]
    fn new(kind: Self::Kind) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum AuthErrorKind {
    TokenInvalid,
    TokenExpired,
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        use AuthErrorKind::*;
        use jsonwebtoken::errors::ErrorKind::*;
        match value.into_kind() {
            ExpiredSignature => AuthError::new(TokenExpired),
            _ => AuthError::new(TokenInvalid),
        }
    }
}

impl IntoResponse for AuthError {
    #[inline(always)]
    fn into_response(self) -> Response {
        Response::from(self)
    }
}

impl From<AuthError> for Response {
    fn from(val: AuthError) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, axum::Json(val)).into_response()
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            AuthErrorKind::TokenInvalid => f.write_str("jwt error: this is invalid"),
            AuthErrorKind::TokenExpired => f.write_str("jwt error: this token has been expired"),
        }
    }
}

impl std::error::Error for AuthError {}
