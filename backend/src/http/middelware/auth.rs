use crate::http::api::usr::UsrIdent;
use crate::http::jwt::{DEFAULT_VALIDATION, Jwt};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    response::IntoResponse,
};
use std::{pin::Pin, sync::LazyLock};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

pub static AUTH_LAYER: LazyLock<AsyncRequireAuthorizationLayer<Auth>> =
    LazyLock::new(|| AsyncRequireAuthorizationLayer::new(Auth));

#[derive(Clone)]
pub struct Auth;

impl AsyncAuthorizeRequest<Body> for Auth {
    type RequestBody = Body;
    type ResponseBody = Body;
    type Future = Pin<
        Box<
            dyn Future<Output = Result<Request, Response>> + Send,
        >,
    >;

    fn authorize(&mut self, mut request: Request) -> Self::Future {
        Box::pin(async move {
            let auth_header = request.headers().get(header::AUTHORIZATION);

            if auth_header.is_none() {
                Err(StatusCode::UNAUTHORIZED.into_response())
            } else {
                let auth_header = decode_header(auth_header.unwrap())?;
                let token = strip_prefix_bearer(auth_header)?;
                let usr_ident = get_usr_ident(token)?;

                request.extensions_mut().insert(usr_ident);
                Ok(request)
            }
        })
    }
}

fn decode_header(token: &HeaderValue) -> Result<&str, Response> {
    match token.to_str() {
        Ok(h) => Ok(h),
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                r#"{"code":"token_invalid"}"#,
            )
                .into_response());
        }
    }
}

fn strip_prefix_bearer(field: &str) -> Result<&str, Response> {
    match field.strip_prefix("Bearer ") {
        Some(token) => Ok(token),
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                r#"{"code":"token_invalid"}"#,
            )
                .into_response());
        }
    }
}

fn get_usr_ident(token: &str) -> Result<UsrIdent, Response> {
    match Jwt::<UsrIdent>::decode_with(token, &DEFAULT_VALIDATION) {
        Ok(usr_ident) => Ok(usr_ident),
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                r#"{"code":"token_invalid"}"#,
            )
                .into_response());
        }
    }
}
