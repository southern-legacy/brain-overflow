use crate::error::api::ApiError;
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use serde::{Deserialize, de::DeserializeOwned};
use validator::Validate;

pub mod utils;

/// Basically same as [`Query`](axum::extract::Query),
/// but rejection casted into [`ApiError`]
#[allow(dead_code)]
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

/// Basically same as [`Path`](axum::extract::Path),
/// but rejection casted into [`ApiError`]
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T);

/// Basically same as [`Json`](axum::extract::Json),
///
/// but rejection casted into [`ApiError`]
#[allow(dead_code)]
#[derive(FromRequest)]
#[from_request(via(axum::extract::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

#[allow(dead_code)]
/// Validated [query](axum::extract::Query), rejection casted into [`ApiError`]
pub struct ValidQuery<T>(pub T)
where
    T: Validate + for<'de> Deserialize<'de> + Send;

#[allow(dead_code)]
/// Validated [path](axum::extract::Path), rejection casted into [`ApiError`]
pub struct ValidPath<T>(pub T)
where
    T: Validate + for<'de> Deserialize<'de> + Send;

/// Validated [json](axum::extract::Json), rejection casted into [`ApiError`]
pub struct ValidJson<T>(pub T)
where
    T: Validate + DeserializeOwned;

impl<S, T> FromRequestParts<S> for ValidPath<T>
where
    S: Send + Sync,
    T: Validate + for<'de> Deserialize<'de> + Send,
{
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::extract;
        let extract::Path::<T>(path) = extract::Path::from_request_parts(parts, state).await?;

        path.validate()
            .map(|_| ValidPath(path))
            .map_err(ApiError::from)
    }
}

impl<S, T> FromRequestParts<S> for ValidQuery<T>
where
    S: Send + Sync,
    T: Validate + for<'de> Deserialize<'de> + Send,
{
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::extract;

        let extract::Query::<T>(que) = extract::Query::from_request_parts(parts, state).await?;
        que.validate()
            .map(|_| ValidQuery(que))
            .map_err(ApiError::from)
    }
}

impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json::<T>(json) = axum::Json::from_request(req, state).await?;

        json.validate()
            .map(|_| ValidJson(json))
            .map_err(ApiError::from)
    }
}
