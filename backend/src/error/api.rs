use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "code")]
pub enum ApiError {
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

    // 服务器错误
    InternalServerError,
}

impl ApiError {
    pub fn code(&self) -> StatusCode {
        match self {
            ApiError::MissingContentType
            | ApiError::InvalidContentType
            | ApiError::MissingContentLength
            | ApiError::BodyTooLarge
            | ApiError::EncodingError
            | ApiError::HeaderWithOpaqueBytes
            | ApiError::ValueParsingError => StatusCode::UNPROCESSABLE_ENTITY,

            ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,

            ApiError::BadRequest => StatusCode::BAD_REQUEST,

            ApiError::UriInvalid => StatusCode::NOT_FOUND,

            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.code(), axum::Json(self)).into_response()
    }
}

impl From<ApiError> for Response {
    #[inline(always)]
    fn from(value: ApiError) -> Self {
        value.into_response()
    }
}

impl From<axum::extract::rejection::BytesRejection> for ApiError {
    fn from(_: axum::extract::rejection::BytesRejection) -> Self {
        Self::BodyTooLarge
    }
}

impl From<axum::extract::rejection::QueryRejection> for ApiError {
    fn from(_: axum::extract::rejection::QueryRejection) -> Self {
        Self::BadRequest
    }
}

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(_: axum::extract::rejection::JsonRejection) -> Self {
        Self::BadRequest
    }
}

impl From<axum::extract::rejection::PathRejection> for ApiError {
    fn from(_: axum::extract::rejection::PathRejection) -> Self {
        Self::BadRequest
    }
}

impl From<axum_valid::ValidRejection<ApiError>> for ApiError {
    fn from(value: axum_valid::ValidRejection<ApiError>) -> Self {
        match value {
            axum_valid::ValidationRejection::Valid(_) => todo!(),
            axum_valid::ValidationRejection::Inner(_) => todo!(),
        }
    }
}
