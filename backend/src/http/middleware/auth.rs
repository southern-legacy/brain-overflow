use std::{
    convert::Infallible,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    extract::Request,
    http::{HeaderMap, header::AUTHORIZATION},
    response::{IntoResponse, Response},
};
use crab_vault_auth::{HttpMethod, Jwt, JwtDecoder, error::AuthError};
use http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use tower::{Layer, Service};

type PinBox<T> = Pin<Box<T>>;

#[derive(Clone)]
pub struct Auth<Inner, T, F>
where
    F: 'static
        + Clone
        + Send
        + Fn(
            &HeaderMap,
            HttpMethod,
            &str,
            Jwt<T>,
        ) -> PinBox<dyn Future<Output = Result<T, Response>> + Send>,
    T: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
    inner_service: Inner,
    decoder: Arc<JwtDecoder>,
    validator: F,
    _p: PhantomData<T>,
}

#[derive(Clone)]
pub struct AuthLayer<T, F>
where
    F: 'static
        + Clone
        + Send
        + Fn(
            &HeaderMap,
            HttpMethod,
            &str,
            Jwt<T>,
        ) -> PinBox<dyn Future<Output = Result<T, Response>> + Send>,
    T: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
    decoder: JwtDecoder,
    validator: F,
    _p: PhantomData<T>,
}

impl<T, F> AuthLayer<T, F>
where
    F: 'static
        + Clone
        + Send
        + Fn(
            &HeaderMap,
            HttpMethod,
            &str,
            Jwt<T>,
        ) -> PinBox<dyn Future<Output = Result<T, Response>> + Send>,
    T: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
    /// # 此函数将在堆上创建一个 [`JwtConfig`] 结构作为这个中间件的配置
    ///
    /// ## 参数说明
    ///
    /// - `decoder`：解码 Jwt 的结构
    /// - `validator`：验证 token 上下文的
    ///
    /// > `validator` 接受 (&HeaderMap, HttpMethod, &str, Jwt<T>) 返回一个 [`Pin`] 住的 [`Box`]<[`Future`]>，
    /// >
    /// > 这个 [`Future`] 可以返回一个 [`Result`]
    /// >
    /// > - `Ok(_)` 时，表示里面的 token 合法，现在将这个校验后的 token 给到 `Inner` 服务
    /// > - `Err(response)` 时，表示 token 不合法，直接给客户端返回相应的错误
    pub fn new(decoder: JwtDecoder, validator: F) -> Self {
        Self {
            decoder,
            validator,
            _p: PhantomData,
        }
    }
}

// 在 Inner 是一个 Service 的情况下，可以为 Auth<Inner> 实现 Service
// 这个 Auth 和 Inner 使用同样的请求参数，axum::http::Request<ReqBody>
impl<Inner, Token, Valid> Service<Request> for Auth<Inner, Token, Valid>
where
    Inner: Service<Request> + Send + Clone + 'static,
    Inner::Error: std::error::Error,
    Inner::Response: IntoResponse,
    Inner::Future: 'static + Send,
    Valid: 'static
        + Clone
        + Send
        + Fn(
            &HeaderMap,
            HttpMethod,
            &str,
            Jwt<Token>,
        ) -> PinBox<dyn Future<Output = Result<Token, Response>> + Send>,
    Token: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner_service.poll_ready(cx).map_err(|_| unreachable!())
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let cloned = self.inner_service.clone();
        let validate_token = self.validator.clone();
        let decoder = self.decoder.clone();
        let mut inner_service = std::mem::replace(&mut self.inner_service, cloned);

        Box::pin(async move {
            let (headers, method, path) = (req.headers(), req.method().into(), req.uri().path());

            match extract_token::<Token>(headers, &decoder).await {
                Ok(token) => match validate_token(headers, method, path, token).await {
                    Ok(ext) => {
                        req.extensions_mut().insert(ext);
                        match inner_service.call(req).await {
                            Ok(response) => Ok(response.into_response()),
                            Err(_) => unreachable!(),
                        }
                    }
                    Err(e) => Ok(e),
                },
                Err(e) => match e {
                    AuthError::TokenExpired => Ok((
                        StatusCode::UNAUTHORIZED,
                        axum::Json(json!({
                            "reason": "tokenExpired"
                        })),
                    )
                        .into_response()),
                    _ => Ok(e.into_response()),
                },
            }
        })
    }
}

impl<Inner, T, F> Layer<Inner> for AuthLayer<T, F>
where
    F: 'static
        + Clone
        + Send
        + Fn(
            &HeaderMap,
            HttpMethod,
            &str,
            Jwt<T>,
        ) -> PinBox<dyn Future<Output = Result<T, Response>> + Send>,
    T: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
    type Service = Auth<Inner, T, F>;

    fn layer(&self, inner: Inner) -> Self::Service {
        let Self {
            decoder,
            validator,
            _p,
        } = self.clone();

        Auth {
            inner_service: inner,
            validator: validator.clone(),
            decoder: Arc::new(decoder),
            _p: PhantomData,
        }
    }
}

/// 提取并验证JWT令牌
async fn extract_token<T>(headers: &HeaderMap, decoder: &JwtDecoder) -> Result<Jwt<T>, AuthError>
where
    T: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
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

    decoder.decode(token)
}
