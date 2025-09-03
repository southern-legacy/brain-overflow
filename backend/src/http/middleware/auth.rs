use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    http::{
        HeaderMap,
        header::AUTHORIZATION,
    },
    response::{IntoResponse, Response},
};
use crab_vault_auth::{Jwt, JwtConfig, error::AuthError};
use tower::{Layer, Service};

use crate::{app_config, http::api::usr::UsrIdent};

#[derive(Clone)]
pub struct AuthMiddleware<Inner> {
    inner: Inner,
}

impl<Inner, ReqBody> Service<axum::http::Request<ReqBody>> for AuthMiddleware<Inner>
where
    Inner: Service<axum::http::Request<ReqBody>> + Send + Clone + 'static,
    ReqBody: 'static + Send,
    Inner::Error: std::error::Error,
    Inner::Response: IntoResponse,
    Inner::Future: 'static + Send,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| unreachable!())
    }

    fn call(&mut self, mut req: axum::http::Request<ReqBody>) -> Self::Future {
        let cloned = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, cloned);

        Box::pin(async move {
            let call_inner_with_req = |req| async move {
                match inner.call(req).await {
                    Ok(val) => Ok(val.into_response()),
                    Err(_) => unreachable!(),
                }
            };

            match extract_and_validate_token(
                req.headers(),
                app_config::server().auth().jwt_config().await,
            )
            .await
            {
                Ok(permission) => {
                    req.extensions_mut().insert(permission);
                    call_inner_with_req(req).await
                }
                Err(e) => Ok(e),
            }
        })
    }
}

#[derive(Clone)]
pub struct AuthLayer;

impl AuthLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<Inner> Layer<Inner> for AuthLayer {
    type Service = AuthMiddleware<Inner>;

    fn layer(&self, service: Inner) -> Self::Service {
        AuthMiddleware { inner: service }
    }
}

/// 提取并验证JWT令牌
async fn extract_and_validate_token(
    headers: &HeaderMap,
    jwt_config: &JwtConfig,
) -> Result<UsrIdent, Response> {
    // 1. 提取Authorization头
    let auth_header = headers
        .get(AUTHORIZATION)
        .ok_or(AuthError::MissingAuthHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidAuthFormat)?;

    // 2. 验证Bearer格式并提取令牌
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidAuthFormat)?;

    // 3. 解码并验证JWT
    let jwt: Jwt<UsrIdent> = Jwt::decode(token, jwt_config)?;

    Ok(jwt.payload)
}
