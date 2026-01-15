use crate::error::api::ApiError;
use axum::{
    extract::{self, Request},
    http::request::Parts,
};
use serde::{Deserialize, de::DeserializeOwned};
use validator::Validate;

pub mod utils;

/// Basically same as [`Path`](axum::extract::Path),
/// but rejection casted into [`ApiError`]
#[derive(extract::FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T);

/// Basically same as [`Query`](axum::extract::Query),
/// but rejection casted into [`ApiError`]
#[allow(dead_code)]
#[derive(extract::FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

/// Basically same as [`Json`](axum::extract::Json),
///
/// but rejection casted into [`ApiError`]
#[allow(dead_code)]
#[derive(extract::FromRequest)]
#[from_request(via(axum::extract::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

#[allow(dead_code)]
/// Validated [`path`](axum::extract::Path), rejection casted into [`ApiError`]
pub struct ValidPath<T>(pub T)
where
    T: Validate + for<'de> Deserialize<'de> + Send;

#[allow(dead_code)]
/// Validated [`query`](axum::extract::Query), rejection casted into [`ApiError`]
pub struct ValidQuery<T>(pub T)
where
    T: Validate + for<'de> Deserialize<'de>;

/// Validated [`json`](axum::extract::Json), rejection casted into [`ApiError`].
/// Can be also wrapped with [`Option`], behavior:
/// - If no content in http request body, then [`None`]
/// - If there is content:
///     - Failed in deserialization: [`Err`]
///     - Failed in validation: [`Err`]
///     - Suceeded finally: [`Ok`]
///
///     > wrapped in [`Some`] of course
pub struct ValidJson<T>(pub T)
where
    T: Validate + DeserializeOwned;

impl<S, T> extract::OptionalFromRequest<S> for Json<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        axum::Json::from_request(req, state)
            .await
            .map_err(ApiError::from)
            .map(|opt| opt.map(|axum::Json(v)| Json(v)))
    }
}

impl<S, T> extract::FromRequestParts<S> for ValidPath<T>
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

impl<S, T> extract::FromRequestParts<S> for ValidQuery<T>
where
    S: Send + Sync,
    T: Validate + for<'de> Deserialize<'de>,
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

impl<S, T> extract::FromRequest<S> for ValidJson<T>
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

impl<S, T> extract::OptionalFromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
{
    type Rejection = ApiError;
    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        let json: Option<axum::Json<T>> = axum::Json::from_request(req, state).await?;
        match json {
            Some(axum::Json(data)) => data
                .validate()
                .map(|_| Some(ValidJson(data)))
                .map_err(ApiError::from),
            None => Ok(None),
        }
    }
}
