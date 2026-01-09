use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use validator::ValidationErrors;

use crate::error::CustomError;

#[derive(Serialize, Debug)]
pub struct ApiError {
    kind: ApiErrorKind,
    context: Vec<serde_json::Value>,
}

#[derive(Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub enum ApiErrorKind {
    // 客户端错误
    MissingContentType,
    InvalidContentType,

    MissingContentLength,
    BodyTooLarge,

    UriInvalid,

    EncodingError,
    ValueParsingError,

    BadRequest,

    MethodNotAllowed,
    HeaderWithOpaqueBytes,

    Unauthorized,

    // 服务器错误
    InternalServerError,
}

impl ApiErrorKind {
    pub fn code(&self) -> StatusCode {
        match self {
            ApiErrorKind::MissingContentType
            | ApiErrorKind::InvalidContentType
            | ApiErrorKind::MissingContentLength
            | ApiErrorKind::BodyTooLarge
            | ApiErrorKind::EncodingError
            | ApiErrorKind::HeaderWithOpaqueBytes
            | ApiErrorKind::ValueParsingError => StatusCode::UNPROCESSABLE_ENTITY,

            ApiErrorKind::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,

            ApiErrorKind::BadRequest => StatusCode::BAD_REQUEST,

            ApiErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,

            ApiErrorKind::UriInvalid => StatusCode::NOT_FOUND,

            ApiErrorKind::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ApiError {
    pub fn with_context<T: Serialize>(mut self, error: T) -> Self {
        self.context.insert(0, serde_json::json!(error));
        self
    }
}

impl CustomError for ApiError {
    type Kind = ApiErrorKind;

    fn kind(&self) -> &ApiErrorKind {
        &self.kind
    }

    fn new(kind: ApiErrorKind) -> Self {
        Self {
            kind,
            context: vec![],
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(v) => (self.kind.code(), v).into_response(),
            Err(_) => (self.kind.code()).into_response(),
        }
    }
}

impl From<ApiError> for Response {
    fn from(value: ApiError) -> Self {
        value.into_response()
    }
}

impl From<axum::extract::rejection::BytesRejection> for ApiError {
    fn from(_: axum::extract::rejection::BytesRejection) -> Self {
        Self::new(ApiErrorKind::BodyTooLarge)
    }
}

impl From<axum::extract::rejection::QueryRejection> for ApiError {
    fn from(_: axum::extract::rejection::QueryRejection) -> Self {
        Self::new(ApiErrorKind::BadRequest)
    }
}

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(_: axum::extract::rejection::JsonRejection) -> Self {
        Self::new(ApiErrorKind::BadRequest)
    }
}

impl From<axum::extract::rejection::PathRejection> for ApiError {
    fn from(_: axum::extract::rejection::PathRejection) -> Self {
        Self::new(ApiErrorKind::BadRequest)
    }
}

impl From<axum_valid::ValidRejection<ApiError>> for ApiError {
    fn from(value: axum_valid::ValidRejection<ApiError>) -> Self {
        match value {
            axum_valid::ValidationRejection::Valid(ValidationErrors(e)) => Self {
                kind: ApiErrorKind::BadRequest,
                context: e
                    .into_iter()
                    .map(|(v, e)| serde_json::json!({ v: e }))
                    .collect(),
            },
            axum_valid::ValidationRejection::Inner(e) => e,
        }
    }
}

impl From<crab_vault::auth::error::AuthError> for ApiError {
    fn from(_: crab_vault::auth::error::AuthError) -> Self {
        Self::new(ApiErrorKind::Unauthorized)
    }
}
