use std::{collections::HashSet, convert::Infallible, pin::Pin, sync::Arc};

use crate::{app_config, error::api::ApiError, http::ENCODER_TO_CRAB_VAULT};

use axum::response::{IntoResponse, Response};
use crab_vault::auth::{HttpMethod, Jwt, Permission, error::AuthError};
use jsonwebtoken::Header;
use regex::Regex;
use tower::Service;

#[derive(Clone)]
pub struct TokenIssueService {
    inner: Arc<TokenIssueServiceInner>,
}

pub struct TokenIssueServiceInner {
    path_regex: Regex,

    map_fn: fn(Regex, &str) -> Pin<Box<dyn Future<Output = Result<String, Response>> + Send>>,

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
        // Regex 的结构很小，32字节，就不堆分配了
        let regex = self.inner.path_regex.clone();
        let map_fn = self.inner.map_fn;
        let allowed_content_types = self.inner.allowed_content_types.clone();

        let method = <&axum::http::Method as Into<HttpMethod>>::into(request.method());
        let safe_to_issue = self.inner.allowed_methods.contains(&method);

        Box::pin(async move {
            if !safe_to_issue {
                return Ok(ApiError::MethodNotAllowed).map(<ApiError as Into<Response>>::into);
            }

            let mapped_crab_vault_path = match map_fn(regex, request.uri().path()).await {
                Ok(value) => value,
                Err(e) => return Ok(e),
            };

            let permission = Permission::new_minimum()
                .permit_method(vec![method])
                .permit_resource_pattern(mapped_crab_vault_path)
                .restrict_maximum_size_option(max_size)
                .permit_content_type(allowed_content_types);

            let config = app_config::auth().encoder_config_to_crab_vault();
            let jwt = Jwt::new(config.issue_as(), config.audience(), permission)
                .expires_in(config.expire_in())
                .not_valid_in(config.not_valid_in());

            let mut header = Header::new(jsonwebtoken::Algorithm::HS256);
                header.kid = config.kids().first().map(|v| v.clone());

            // TODO! unwrap 纠正
            match ENCODER_TO_CRAB_VAULT.encode(&header, &jwt, config.kids().first().unwrap()) {
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
        fn default_map_fn(
            _: Regex,
            _: &str,
        ) -> Pin<Box<dyn Future<Output = Result<String, Response>> + Send>> {
            Box::pin(async { Ok("".into()) })
        }

        Self {
            path_regex: Regex::new("()").expect("你把正则表达式写错啦"),
            map_fn: default_map_fn,
            allowed_methods: HashSet::new(),
            allowed_content_types: vec![],
            max_size: Some(5 * 1024 * 1024),
        }
    }
}

#[allow(dead_code)]
impl TokenIssueServiceInner {
    #[inline]
    pub fn regex(mut self, regex: &str) -> Self {
        self.path_regex = regex.try_into().expect("你把正则表达式写错啦");
        self
    }

    #[inline]
    pub fn map_fn(
        mut self,
        map_fn: fn(Regex, &str) -> Pin<Box<dyn Future<Output = Result<String, Response>> + Send>>,
    ) -> Self {
        self.map_fn = map_fn;
        self
    }

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
