use std::{collections::HashSet, convert::Infallible, pin::Pin, sync::Arc};

use crate::{app_config, error::api::ApiError};

use axum::response::{IntoResponse, Response};
use crab_vault::auth::{HttpMethod, Jwt, Permission};
use tower::Service;

#[derive(Clone)]
pub struct TokenIssueService {
    inner: Arc<TokenIssueServiceInner>,
}

pub struct TokenIssueServiceInner {
    // 条件限制
    allowed_methods: HashSet<HttpMethod>,
    allowed_content_types: Vec<String>,
    max_size: Option<usize>,
}

impl<R: std::marker::Send + 'static> Service<axum::http::request::Request<R>>
    for TokenIssueService
{
    type Response = Response;

    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // 负载均衡，但是如何评判这个 service 到了瓶颈呢？
        // 而且这个 Service 不涉及磁盘 IO，应该比较快吧
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: axum::http::request::Request<R>) -> Self::Future {
        let max_size = self.inner.max_size;
        let allowed_content_types = self.inner.allowed_content_types.clone();

        let path = request.uri().path().to_string();
        let method = <&axum::http::Method as Into<HttpMethod>>::into(request.method());
        let safe_to_issue = self.inner.allowed_methods.contains(&method);

        Box::pin(async move {
            if !safe_to_issue {
                return Ok(<ApiError as Into<Response>>::into(
                    ApiError::MethodNotAllowed,
                ));
            }

            let permission = Permission::new_minimum()
                .permit_method(vec![method])
                .permit_resource_pattern(path)
                .restrict_maximum_size_option(max_size)
                .permit_content_type(allowed_content_types);

            let config = &app_config::crab_vault().encoder;
            let jwt = Jwt::new(&config.issue_as, &config.audience, permission)
                .expires_in(config.expires_in)
                .not_valid_in(config.not_valid_in);

            match config.encoder.encode_randomly(&jwt) {
                Ok(v) => Ok(v.into_response()),
                Err(e) => Ok(e.into_response()),
            }
        })
    }
}

impl TokenIssueService {
    #[inline]
    pub fn new(inner: TokenIssueServiceInner) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl Default for TokenIssueServiceInner {
    fn default() -> Self {
        Self {
            allowed_methods: HashSet::new(),
            allowed_content_types: vec![],
            max_size: Some(5 * 1024 * 1024),
        }
    }
}

#[allow(dead_code)]
impl TokenIssueServiceInner {
    #[inline]
    pub fn allowed_methods(mut self, methods: &[HttpMethod]) -> Self {
        self.allowed_methods = methods.iter().copied().collect();
        self
    }

    #[inline]
    pub fn allowed_content_types(mut self, content_types: Vec<String>) -> Self {
        self.allowed_content_types = content_types;
        self
    }

    #[inline]
    pub fn max_size_option(mut self, max_size: Option<usize>) -> Self {
        self.max_size = max_size;
        self
    }
}
