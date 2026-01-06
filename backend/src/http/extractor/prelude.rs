use crate::error::api::ApiError;
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use axum_valid::HasValidate;
use serde::Deserialize;
use validator::Validate;

#[allow(dead_code)]
pub struct ValidQuery<T>(pub T);

#[allow(dead_code)]
pub struct ValidPath<T>(pub T);

pub struct ValidJson<T>(pub T);

#[derive(FromRequest, FromRequestParts)]
#[from_request(via(axum_valid::Valid), rejection(ApiError))]
pub struct Valid<T>(pub T);

#[allow(dead_code)]
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T);

#[derive(FromRequest)]
#[from_request(via(axum::extract::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

impl<T> HasValidate for Query<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

impl<T> HasValidate for Path<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

impl<T> HasValidate for Json<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

impl<T> ValidJson<T>
where
    T: Validate + for<'de> Deserialize<'de>,
{
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: Validate + for<'de> Deserialize<'de>,
    Valid<Json<T>>: FromRequest<S, Rejection = ApiError>,
{
    type Rejection = ApiError;
    async fn from_request(parts: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(ValidJson(Valid::from_request(parts, state).await?.0.0))
    }
}

impl<S, T> FromRequestParts<S> for ValidPath<T>
where
    S: Send + Sync,
    Valid<Path<T>>: FromRequestParts<S, Rejection = ApiError>,
{
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(ValidPath(
            Valid::from_request_parts(parts, state).await?.0.0,
        ))
    }
}

impl<S, T> FromRequestParts<S> for ValidQuery<T>
where
    S: Send + Sync,
    Valid<Query<T>>: FromRequestParts<S, Rejection = ApiError>,
{
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(ValidQuery(
            Valid::from_request_parts(parts, state).await?.0.0,
        ))
    }
}

