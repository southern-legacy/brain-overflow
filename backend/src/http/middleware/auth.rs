use std::{
    convert::Infallible,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use ::auth::{Jwt, JwtDecoder, error::AuthError};
use axum::{
    extract::Request,
    http::{HeaderMap, header::AUTHORIZATION},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use tower::{Layer, Service};

use crate::server::ServerState;

pub type PinBoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub trait Judgement<T>
where
    Self: 'static + Clone + Send + Sync + Fn(ServerState, &Request, Jwt<T>) -> PinBoxFuture<Result<T, Response>>,
{
}

impl<T, Arbitrary> Judgement<T> for Arbitrary where
    Arbitrary: 'static + Clone + Send + Sync + Fn(ServerState, &Request, Jwt<T>) -> PinBoxFuture<Result<T, Response>>
{
}

pub trait TokenContent
where
    Self: 'static + Clone + Sync + Send + for<'de> Deserialize<'de>,
{
}

impl<Arbitrary> TokenContent for Arbitrary where Arbitrary: 'static + Clone + Sync + Send + for<'de> Deserialize<'de> {}

#[derive(Clone)]
pub struct Auth<Inner, T, F>
where
    F: Judgement<T>,
    T: TokenContent,
{
    inner: Inner,
    state: ServerState,
    judge: F,
    _p: PhantomData<T>,
}

#[derive(Clone)]
pub struct AuthLayer<T, F>
where
    F: Judgement<T>,
    T: TokenContent,
{
    judge: F,
    state: ServerState,
    _p: PhantomData<T>,
}

impl<T, F> AuthLayer<T, F>
where
    F: Judgement<T>,
    T: TokenContent,
{
    /// # 此函数将在堆上创建一个 [`JwtConfig`] 结构作为这个中间件的配置
    ///
    /// ## 参数说明
    ///
    /// - [`ServerState`]：可能需要的全局资源，比如数据库连接
    /// - [`Request`]：请求本身
    /// - [`Jwt<T>`]：目前看来没有问题的 token
    ///
    /// 你应该返回：
    ///
    /// [`Result<T, Response>`]
    ///
    /// - `Ok(_)` 时，表示里面的 token 合法，现在将这个校验后的 `T` 给到 `Inner` 服务
    /// - `Err(response)` 时，表示 token 不合法，直接给客户端返回相应的错误
    pub fn new(state: ServerState, judge: F) -> Self {
        Self {
            state,
            judge,
            _p: PhantomData,
        }
    }
}

// 在 Inner 是一个 Service 的情况下，可以为 Auth<Inner> 实现 Service
// 这个 Auth 和 Inner 使用同样的请求参数，axum::http::Request<ReqBody>
impl<Inner, T, F> Service<Request> for Auth<Inner, T, F>
where
    Inner: Service<Request> + Send + Clone + 'static,
    Inner::Error: std::error::Error,
    Inner::Response: IntoResponse,
    Inner::Future: 'static + Send,
    F: Judgement<T>,
    T: TokenContent,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| unreachable!())
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let cloned = self.inner.clone();
        let judge = self.judge.clone();
        let state = self.state.clone();
        let mut inner_service = std::mem::replace(&mut self.inner, cloned);

        Box::pin(async move {
            match extract_token::<T>(req.headers(), &state.config().auth.decoder).await {
                Ok(token) => match judge(state, &req, token).await {
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
    F: Judgement<T>,
    T: TokenContent,
{
    type Service = Auth<Inner, T, F>;

    fn layer(&self, inner: Inner) -> Self::Service {
        let Self { state, judge, _p } = self.clone();

        Auth {
            inner,
            state,
            judge,
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
