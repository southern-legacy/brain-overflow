use std::fmt::Display;

use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::CustomError;

#[derive(Serialize, Debug)]
#[serde(rename = "code", rename_all = "camelCase")]
pub struct RequestError {
    #[cfg(feature = "default")]
    #[cfg(not(feature = "more-message"))]
    #[serde(rename = "code")]
    kind: default::RequestErrorKind,
    #[cfg(feature = "more-message")]

    #[serde(rename = "code", flatten)]
    kind: more_message::RequestErrorKind,
}

impl CustomError for RequestError {
    #[cfg(feature = "default")]
    #[cfg(not(feature = "more-message"))]
    type Kind = default::RequestErrorKind;

    #[cfg(feature = "more-message")]
    type Kind = more_message::RequestErrorKind;

    #[inline(always)]
    fn kind(&self) -> &Self::Kind {
        &self.kind
    }

    #[inline(always)]
    fn new(kind: Self::Kind) -> Self {
        Self { kind }
    }
}

#[cfg(feature = "default")]
#[cfg(not(feature = "more-message"))]
impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use default::RequestErrorKind::*;
        match self.kind() {
            JsonDataError => f.write_str("no syntax error, but can't be parsed into target type"),
            JsonSyntaxError => f.write_str("json syntax error"),
            ContentTypeMissing => f.write_str("expected content type: application/json"),
            FailedBufferBody => f.write_str("server failed to buffer the request body"),
            PathFormatError => f.write_str("path format error"),
            PathMissingFields => f.write_str("path missing fields"),
            QueryParamDeserializeFailed => f.write_str("query param can't be deserialized into target type"),
            InvalidParam => f.write_str("invalid params passed in")
        }
    }
}

#[cfg(feature = "more-message")]
impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use more_message::RequestErrorKind::*;
        match self.kind() {
            JsonDataError { msg } => f.write_fmt(format_args!("no syntax error, but can't be parsed into target type, details: {msg}")),
            JsonSyntaxError { msg } => f.write_fmt(format_args!("json syntax error, details: {msg}")),
            ContentTypeMissing { msg } => f.write_fmt(format_args!("expected content type: application/json, details: {msg}")),
            FailedBufferBody { msg } => f.write_fmt(format_args!("server failed to buffer the request body, details: {msg}")),
            PathFormatError { msg } => f.write_fmt(format_args!("path format error, details: {msg}")),
            PathMissingFields { msg } => f.write_fmt(format_args!("path missing fields, details: {msg}")),
            QueryParamDeserializeFailed { msg } => f.write_fmt(format_args!("query param can't be deserialized into target type, details: {msg}")),
            InvalidParam { msg } => f.write_fmt(format_args!("invalid params passed in, details: {msg}"))
        }
    }
}

impl core::error::Error for RequestError {}

#[cfg(feature = "default")]
#[cfg(not(feature = "more-message"))]
mod default {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum RequestErrorKind {
        /// 422 Unprocessable Entity,
        /// 是合法的 JSON 但是无法解析到目标结构
        JsonDataError,

        /// 400 Bad Request,
        /// JSON 格式非法
        JsonSyntaxError,

        /// 400 Bad Request,
        /// Path 格式错误，比如原本应该传数字但是传的字符串
        PathFormatError,

        /// 422 Unprocessable Entity,
        /// Path 缺少了一些参数
        PathMissingFields,

        /// 400
        /// 请求参数无法被转换为目标类型
        QueryParamDeserializeFailed,

        /// 422 Unprocessable Entity
        /// 传过来的结构（不论通过 json 还是 query param）错误
        InvalidParam,

        /// 400 Bad Request,
        /// 请求头中没有添加对应的 Content-Type
        ContentTypeMissing,

        /// 500 Internal Server Error,
        /// 服务器无法缓存请求体
        FailedBufferBody,
    }

    impl IntoResponse for RequestError {
        #[inline(always)]
        fn into_response(self) -> axum::response::Response {
            Response::from(self)
        }
    }

    impl From<RequestError> for Response {
        fn from(value: RequestError) -> Self {
            use axum::http::StatusCode;
            use RequestErrorKind::*;
            let status_code = match &value.kind {
                JsonDataError | PathMissingFields | InvalidParam => StatusCode::UNPROCESSABLE_ENTITY,
                JsonSyntaxError | PathFormatError | QueryParamDeserializeFailed => StatusCode::BAD_REQUEST,
                ContentTypeMissing => StatusCode::BAD_REQUEST,
                FailedBufferBody => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status_code, axum::Json(value)).into_response()
        }
    }

    impl From<axum::extract::rejection::JsonRejection> for RequestError {
        fn from(value: axum::extract::rejection::JsonRejection) -> Self {
            use axum::extract::rejection::JsonRejection::*;
            match value {
                JsonDataError(_e) => RequestError::new(RequestErrorKind::JsonDataError),
                JsonSyntaxError(_e) => RequestError::new(RequestErrorKind::JsonSyntaxError),
                MissingJsonContentType(_e) => RequestError::new(RequestErrorKind::ContentTypeMissing),
                BytesRejection(_e) => RequestError::new(RequestErrorKind::FailedBufferBody),
                _ => todo!(),
            }
        }
    }

    impl From<axum::extract::rejection::PathRejection> for RequestError {
        fn from(value: axum::extract::rejection::PathRejection) -> Self {
            use axum::extract::rejection::PathRejection::*;
            match value {
                FailedToDeserializePathParams(_e) => RequestError::new(RequestErrorKind::PathFormatError),
                MissingPathParams(_e) => RequestError::new(RequestErrorKind::PathMissingFields),
                _ => todo!(),
            }
        }
    }

    impl From<axum::extract::rejection::QueryRejection> for RequestError {
        fn from(value: axum::extract::rejection::QueryRejection) -> Self {
            use axum::extract::rejection::QueryRejection::*;
            match value {
                FailedToDeserializeQueryString(_e) => RequestError::new(RequestErrorKind::QueryParamDeserializeFailed),
                _ => todo!(),
            }
        }
    }

    impl From<axum_valid::ValidRejection<RequestError>> for RequestError {
        fn from(value: axum_valid::ValidRejection<RequestError>) -> Self {
            use axum_valid::ValidationRejection::*;
            match value {
                Valid(_e) => RequestError::new(RequestErrorKind::InvalidParam),
                Inner(e) => e,
            }
        }
    }
}

#[cfg(feature = "more-message")]
mod more_message {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename = "code", rename_all = "camelCase", tag = "code")]
    pub enum RequestErrorKind {
        /// 422 Unprocessable Entity,
        /// 是合法的 JSON 但是无法解析到目标结构
        JsonDataError { msg: String },

        /// 400 Bad Request,
        /// JSON 格式非法
        JsonSyntaxError { msg: String },

        /// 400 Bad Request,
        /// Path 格式错误，比如原本应该传数字但是传的字符串
        PathFormatError { msg: String },

        /// 422 Unprocessable Entity,
        /// Path 缺少了一些参数
        PathMissingFields { msg: String },

        /// 400
        /// 请求参数无法被转换为目标类型
        QueryParamDeserializeFailed { msg: String },

        /// 422 Unprocessable Entity
        /// 传过来的结构（不论通过 json 还是 query param）错误
        InvalidParam { msg: String },

        /// 400 Bad Request,
        /// 请求头中没有添加对应的 Content-Type
        ContentTypeMissing { msg: String },

        /// 500 Internal Server Error,
        /// 服务器无法缓存请求体
        FailedBufferBody { msg: String },
    }

    impl IntoResponse for RequestError {
        #[inline(always)]
        fn into_response(self) -> axum::response::Response {
            Response::from(self)
        }
    }

    impl From<RequestError> for Response {
        fn from(value: RequestError) -> Self {
            use axum::http::StatusCode;
            use RequestErrorKind::*;
            let status_code = match value.kind() {
                PathMissingFields{ msg: _ } | InvalidParam{ msg: _ } |  JsonDataError { msg: _ } => StatusCode::UNPROCESSABLE_ENTITY,
                PathFormatError{ msg: _ } | QueryParamDeserializeFailed{ msg: _ } | JsonSyntaxError { msg: _ } | ContentTypeMissing { msg: _ } => StatusCode::BAD_REQUEST,
                FailedBufferBody { msg: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status_code, axum::Json(value)).into_response()
        }
    }

    impl From<axum::extract::rejection::JsonRejection> for RequestError {
        fn from(value: axum::extract::rejection::JsonRejection) -> Self {
            use axum::extract::rejection::JsonRejection::*;
            use RequestErrorKind as Kind;
            use RequestError as Error;
            match value {
                JsonDataError(e) => Error::new(Kind::JsonDataError { msg: e.to_string() } ),
                JsonSyntaxError(e) => Error::new(Kind::JsonSyntaxError { msg: e.to_string() }),
                MissingJsonContentType(e) => Error::new(Kind::ContentTypeMissing { msg: e.to_string() }),
                BytesRejection(e) => Error::new(Kind::FailedBufferBody { msg: e.to_string() }),
                _ => todo!(),
            }
        }
    }

    impl From<axum::extract::rejection::PathRejection> for RequestError {
        fn from(value: axum::extract::rejection::PathRejection) -> Self {
            use axum::extract::rejection::PathRejection::*;
            use RequestErrorKind as Kind;
            use RequestError as Error;
            match value {
                FailedToDeserializePathParams(e) => Error::new(Kind::PathFormatError { msg: e.to_string() }),
                MissingPathParams(e) => Error::new(Kind::PathMissingFields { msg: e.to_string() }),
                _ => todo!(),
            }
        }
    }

    impl From<axum::extract::rejection::QueryRejection> for RequestError {
        fn from(value: axum::extract::rejection::QueryRejection) -> Self {
            use axum::extract::rejection::QueryRejection::*;
            use RequestErrorKind as Kind;
            use RequestError as Error;
            match value {
                FailedToDeserializeQueryString(e) => Error::new(Kind::QueryParamDeserializeFailed { msg: e.to_string() }),
                _ => todo!(),
            }
        }
    }

    impl From<axum_valid::ValidRejection<RequestError>> for RequestError {
        fn from(value: axum_valid::ValidRejection<RequestError>) -> Self {
            use axum_valid::ValidationRejection::*;
            use RequestErrorKind as Kind;
            use RequestError as Error;
            match value {
                Valid(e) => Error::new(Kind::InvalidParam { msg: e.to_string() }),
                Inner(e) => e,
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "default")]
#[cfg(not(feature = "more-message"))]
mod test {
    use crate::error::request::*;
    use super::default::*;

    #[test]
    fn test() {
        helper(RequestErrorKind::JsonDataError);
        helper(RequestErrorKind::JsonSyntaxError);
        helper(RequestErrorKind::PathFormatError);
        helper(RequestErrorKind::PathMissingFields);
        helper(RequestErrorKind::QueryParamDeserializeFailed);
        helper(RequestErrorKind::ContentTypeMissing);
        helper(RequestErrorKind::FailedBufferBody);
        helper(RequestErrorKind::InvalidParam);
    }

    fn helper(err: RequestErrorKind) {
        let err = RequestError::new(err);
        println!("{}", serde_json::to_string(&err).unwrap());
    }
}

#[cfg(test)]
#[cfg(feature = "more-message")]
mod test {
    use crate::error::request::*;
    use super::more_message::*;

    #[test]
    fn test() {
        helper(RequestErrorKind::JsonDataError { msg: "data error".to_string() });
        helper(RequestErrorKind::JsonSyntaxError { msg: "data error".to_string() } );
        helper(RequestErrorKind::PathFormatError { msg: "data error".to_string() } );
        helper(RequestErrorKind::PathMissingFields { msg: "data error".to_string() } );
        helper(RequestErrorKind::QueryParamDeserializeFailed { msg: "data error".to_string() } );
        helper(RequestErrorKind::ContentTypeMissing { msg: "data error".to_string() } );
        helper(RequestErrorKind::FailedBufferBody { msg: "data error".to_string() } );
        helper(RequestErrorKind::InvalidParam { msg: "data error".to_string() } );
    }

    fn helper(err: RequestErrorKind) {
        let err = RequestError::new(err);
        println!("{}", serde_json::to_string(&err).unwrap());
    }
}