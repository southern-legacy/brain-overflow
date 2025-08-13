use std::fmt::Display;

use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::CustomError;

#[derive(Serialize, Debug)]
#[serde(rename = "code", rename_all = "camelCase")]
pub struct RequestError {
    #[serde(rename = "code")]
    kind: RequestErrorKind,
}

impl CustomError for RequestError {
    type Kind = RequestErrorKind;

    #[inline(always)]
    fn kind(&self) -> &Self::Kind {
        &self.kind
    }

    #[inline(always)]
    fn new(kind: Self::Kind) -> Self {
        Self { kind }
    }
}

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

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RequestErrorKind::*;
        match self.kind() {
            JsonDataError => f.write_str("there's no syntax error, but this json can't be parsed into target type"),
            JsonSyntaxError => f.write_str("json syntax error"),
            ContentTypeMissing => f.write_str("expected content type: application/json"),
            FailedBufferBody => f.write_str("server failed to buffer the request body"),
            PathFormatError => f.write_str("path format error"),
            PathMissingFields => f.write_str("path missin fields"),
            QueryParamDeserializeFailed => f.write_str("query param can't be deserialized into target type"),
            InvalidParam => f.write_str("invalid params passed in")
        }
    }
}

impl core::error::Error for RequestError {}


#[cfg(test)]
mod test {
    use crate::error::request::*;

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