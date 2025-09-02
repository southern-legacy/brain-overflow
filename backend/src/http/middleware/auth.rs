use std::{
    collections::HashSet,
    convert::Infallible,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    http::{
        HeaderMap,
        header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use crab_vault_auth::{error::AuthError, HttpMethod, Jwt, JwtConfig, Permission};
use glob::Pattern;
use tokio::sync::OnceCell;
use tower::{Layer, Service};

use crate::{app_config, error::api::ApiError};

#[derive(Clone)]
pub struct AuthMiddleware<Inner> {
    inner: Inner,
    jwt_config: Arc<JwtConfig>,
    path_rules: Arc<PathRulesCache>,
}

struct PathRulesCache {
    path_rules: OnceCell<Vec<(Pattern, HashSet<HttpMethod>)>>,
}

impl PathRulesCache {
    fn new() -> Self {
        Self {
            path_rules: OnceCell::new(),
        }
    }

    async fn should_not_protect(&self, path: &str, method: HttpMethod) -> bool {
        let path_rules = self
            .path_rules
            .get_or_init(async || app_config::server().auth().get_compiled_path_rules())
            .await;

        for (pattern, allowed_method) in path_rules {
            if pattern.matches(path)
                && (allowed_method.contains(&HttpMethod::All) || allowed_method.contains(&method))
            {
                return true;
            }
        }

        false
    }
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
        let jwt_config = self.jwt_config.clone();
        let path_rules = self.path_rules.clone();

        Box::pin(async move {
            let call_inner_with_req = |req| async move {
                match inner.call(req).await {
                    Ok(val) => Ok(val.into_response()),
                    Err(_) => unreachable!(),
                }
            };

            if path_rules
                .should_not_protect(req.uri().path(), req.method().into())
                .await
            {
                req.extensions_mut().insert(Permission::new_root());
                return call_inner_with_req(req).await;
            }

            match extract_and_validate_token(
                req.headers(),
                req.method().into(),
                req.uri().path(),
                &jwt_config,
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
pub struct AuthLayer(Arc<JwtConfig>, Arc<PathRulesCache>);

impl AuthLayer {
    /// 此函数将在堆上创建一个 [`JwtConfig`] 结构作为这个中间件的配置
    pub fn new() -> Self {
        Self(
            Arc::new(
                app_config::server()
                    .auth()
                    .jwt_config_builder()
                    .clone()
                    .build()
                    .map_err(|e| e.exit_now())
                    .unwrap(),
            ),
            Arc::new(PathRulesCache::new()),
        )
    }
}

impl<Inner> Layer<Inner> for AuthLayer {
    type Service = AuthMiddleware<Inner>;

    fn layer(&self, service: Inner) -> Self::Service {
        AuthMiddleware {
            inner: service,
            jwt_config: self.0.clone(),
            path_rules: self.1.clone(),
        }
    }
}

/// 提取并验证JWT令牌
async fn extract_and_validate_token(
    headers: &HeaderMap,
    method: HttpMethod,
    path: &str,
    jwt_config: &JwtConfig,
) -> Result<Permission, Response> {
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
    let jwt: Jwt<Permission> = Jwt::decode(token, jwt_config)?;

    // 4. 检查 content-length，如果没过这个要求，那更是演都不演了
    let content_length = headers
        .get(CONTENT_LENGTH)
        .ok_or(ApiError::MissingContentLength)?
        .to_str()
        .map_err(|_| ApiError::EncodingError)?
        .parse()
        .map_err(|_| ApiError::ValueParsingError)?;
    if !jwt.payload.check_size(content_length) {
        return Err(ApiError::BodyTooLarge.into());
    }

    // 5. 检查资源路径匹配和请求方法
    if !jwt.payload.can_perform(method) || !jwt.payload.can_access(path) {
        return Err(AuthError::InsufficientPermissions.into());
    }

    // 6. 检查 content-type
    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or(ApiError::MissingContentType)?
        .to_str()
        .map_err(|_| ApiError::InvalidContentType)?;
    if !jwt.payload.check_content_type(content_type) {
        return Err(ApiError::InvalidContentType.into());
    }

    Ok(jwt.payload)
}
