use std::{pin::Pin, sync::LazyLock};
use crate::http::jwt::{DEFAULT_VALIDATION, Jwt};
use crate::http::api::usr::UsrIdent;
use axum::http::HeaderValue;
use axum::response::Response;
use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    response::IntoResponse,
};
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
            dyn Future<Output = Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>>
                + Send,
        >,
    >;

    fn authorize(&mut self, mut request: Request<Body>) -> Self::Future {
        Box::pin(async move {
            let auth_header = request.headers().get(header::AUTHORIZATION);

            if auth_header.is_none() {
                Err((
                    StatusCode::UNAUTHORIZED,
                    "你还未登录!".to_string().into_response(),
                )
                    .into_response())
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
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                format!("Found a character can't be decoded! details:{e}"),
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
                "The value of field Authorization should starts with Bearer!",
            )
                .into_response());
        }
    }
}

fn get_usr_ident(token: &str) -> Result<UsrIdent, Response> {
    match Jwt::<UsrIdent>::decode_with(token, &DEFAULT_VALIDATION) {
        Ok(usr_ident) => Ok(usr_ident),
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                format!("Failed to validate token! details:{e}!"),
            )
                .into_response());
        }
    }
}
