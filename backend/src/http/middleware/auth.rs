use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    http::{
        HeaderMap,
        header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use crab_vault::auth::{HttpMethod, Jwt, JwtDecoder, Permission, error::AuthError};
use tower::{Layer, Service};

use crate::{app_config, error::api::ApiError};


#[derive(Clone)]
pub struct Auth<Inner> {
    inner: Inner,
    jwt_config: &'static JwtDecoder,
}

#[derive(Clone)]
pub struct AuthLayer(&'static JwtDecoder);

// 在 Inner 是一个 Service 的情况下，可以为 Auth<Inner> 实现 Service
// 这个 Auth 和 Inner 使用同样的请求参数，axum::http::Request<ReqBody>
impl<Inner, ReqBody> Service<axum::http::Request<ReqBody>> for Auth<Inner>
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
        let jwt_config = self.jwt_config;
        let mut inner = std::mem::replace(&mut self.inner, cloned);

        Box::pin(async move {
            let call_inner_with_req = |req| async move {
                match inner.call(req).await {
                    Ok(val) => Ok(val.into_response()),
                    Err(_) => unreachable!(),
                }
            };

            if should_not_protect(req.uri().path(), req.method().into()).await {
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


impl AuthLayer {
    /// 此函数将在堆上创建一个 [`JwtConfig`] 结构作为这个中间件的配置
    pub fn new(jwt_decoder: &'static JwtDecoder) -> Self {
        Self(jwt_decoder)
    }
}

impl<Inner> Layer<Inner> for AuthLayer {
    type Service = Auth<Inner>;

    fn layer(&self, inner: Inner) -> Self::Service {
        let Self(jwt_config) = self.clone();
        
        Auth { inner, jwt_config }
    }
}

async fn should_not_protect(path: &str, method: HttpMethod) -> bool {
    for (pattern, allowed_method) in &app_config::auth().path_rules {
        if pattern.matches(path)
            && (allowed_method.contains(&HttpMethod::All)
                || allowed_method.contains(&method)
                || (allowed_method.contains(&HttpMethod::Safe) && method.safe())
                || (allowed_method.contains(&HttpMethod::Unsafe) && !method.safe()))
        {
            return true;
        }
    }

    false
}

/// 提取并验证JWT令牌
async fn extract_and_validate_token(
    headers: &HeaderMap,
    method: HttpMethod,
    path: &str,
    decoder: &JwtDecoder,
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
    let jwt: Jwt<Permission> = decoder.decode(token)?;

    // 4. 检查 content-length，如果没过这个要求，那更是演都不演了
    // 当然，如果访问的是一个 bucket (只有一个) 那就不用检查
    if path.split('/').filter(|v| !v.is_empty()).count() <= 1 {
        return Ok(jwt.load);
    }

    let content_length = headers
        .get(CONTENT_LENGTH)
        .ok_or(ApiError::MissingContentLength)?
        .to_str()
        .map_err(|_| ApiError::HeaderWithOpaqueBytes)?
        .parse()
        .map_err(|_| ApiError::ValueParsingError)?;

    let perm = jwt.load.clone().compile();
    if !perm.check_size(content_length) {
        return Err(ApiError::BodyTooLarge.into());
    }

    // 5. 检查资源路径匹配和请求方法
    if !perm.can_perform_method(method) || !perm.can_access(path) {
        return Err(AuthError::InsufficientPermissions.into());
    }

    // 6. 检查 content-type
    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or(ApiError::MissingContentType)?
        .to_str()
        .map_err(|_| ApiError::InvalidContentType)?;
    if !perm.check_content_type(content_type) {
        return Err(ApiError::InvalidContentType.into());
    }

    Ok(jwt.load)
}
