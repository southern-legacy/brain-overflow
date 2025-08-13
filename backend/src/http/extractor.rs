use crate::error::request::RequestError;
use axum::{extract::{FromRequest, FromRequestParts, Request}, http::request::Parts};
use axum_valid::HasValidate;

macro_rules! impl_extract {
    ($name: ident, $wrapper: ident, $rejection: ident, FromRequestParts) => {
        impl<S, T> FromRequestParts<S> for $name<T>
        where
            S: Send + Sync,
            Valid<$wrapper<T>>: FromRequestParts<S, Rejection = $rejection>,
        {
            type Rejection = $rejection;

            async fn from_request_parts(
                parts: &mut Parts,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                Ok($name(Valid::from_request_parts(parts, state).await?.0.0))
            }
        }
    };
    ($name: ident, $wrapper: ident, $rejection: ident, FromRequest) => {
        impl<S, T> FromRequest<S> for $name<T>
        where
            S: Send + Sync,
            Valid<$wrapper<T>>: FromRequest<S, Rejection = $rejection>,
        {
            type Rejection = $rejection;

            async fn from_request(parts: Request, state: &S) -> Result<Self, Self::Rejection> {
                Ok($name(Valid::from_request(parts, state).await?.0.0))
            }
        }
    };
}

#[allow(dead_code)]
pub struct ValidQuery<T>(pub T);
#[allow(dead_code)]
pub struct ValidPath<T>(pub T);
pub struct ValidJson<T>(pub T);
impl_extract!(ValidQuery, Query, RequestError, FromRequestParts);
impl_extract!(ValidPath, Path, RequestError, FromRequestParts);
impl_extract!(ValidJson, Json, RequestError, FromRequest);

#[derive(FromRequest, FromRequestParts)]
#[from_request(via(axum_valid::Valid), rejection(RequestError))]
pub struct Valid<T>(pub T);

#[allow(dead_code)]
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(RequestError))]
pub struct Query<T>(pub T);

impl<T> HasValidate for Query<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(RequestError))]
pub struct Path<T>(pub T);

impl<T> HasValidate for Path<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::extract::Json), rejection(RequestError))]
pub struct Json<T>(pub T);

impl<T> HasValidate for Json<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}
