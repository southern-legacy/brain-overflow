mod danger_zone;
mod info;
mod login;
mod refresh;
mod signup;

use ::auth::{Jwt, error::AuthError};
use axum::{
    Extension, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use http::header;
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::{app_config::util::JwtEncoderConfig, http::middleware::auth::Judgement};
use crate::{entity::user::user_info::UserInfo, http::middleware::auth::AuthLayer, server::ServerState};

static ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(argon2::Config::default);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RefreshToken {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(transparent)]
pub struct AccessToken {
    pub id: Uuid,
}

pub(super) fn build_router(
    state: ServerState,
    auth_layer: AuthLayer<AccessToken, impl Judgement<AccessToken>>,
    strict_layer: AuthLayer<RefreshToken, impl Judgement<RefreshToken>>,
) -> Router {
    async fn redirect(state: State<ServerState>, ident: Extension<AccessToken>) -> impl IntoResponse {
        (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, state.prefix_uri(format!("/user/{}", ident.id)))],
        )
    }

    let danger = Router::new()
        .route("/user", routing::delete(danger_zone::delete_account))
        .route("/user", routing::put(danger_zone::change_auth_info))
        .route("/user/refresh", routing::any(refresh::refresh))
        .layer(strict_layer)
        .with_state(state.clone());

    danger.merge(
        Router::new()
            .route("/user/bio", routing::get(redirect))
            .route("/user/bio/{which}", routing::put(info::put))
            .layer(auth_layer)
            .route("/user/login", routing::post(login::login))
            .route("/user/{id}/{code}", routing::get(signup::confirm))
            .route("/user", routing::post(signup::signup))
            .route("/user/{id}", routing::get(info::get))
            .with_state(state),
    )
}

/// 检查密码是否正确
///
/// `val` 中是 使用 argon2 哈希过后的密码，`password` 是明文密码，通过 ssl 保证传输过程中的数据安全
#[tracing::instrument(name = "[user/check password]", skip_all)]
async fn check_password(val: &UserInfo, password: &str) -> Result<(), Response> {
    match argon2::verify_encoded(&val.password_hash, password.as_bytes()) {
        Ok(true) => {
            tracing::info!("Authorization of user (id: {}) successfully.", val.id);
            Ok(())
        }
        Ok(false) => {
            tracing::info!("Authorization of user (id: {}) failed for incorrect password.", val.id);
            Err(StatusCode::UNAUTHORIZED.into_response())
        }
        Err(e) => {
            tracing::error!("Error checking password! {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

#[tracing::instrument(name = "[user/generate password]", skip_all)]
async fn generate_password_hash(password: &str) -> Result<String, Response> {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::now_v7().into_bytes());
    match argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG) {
        Ok(val) => Ok(val),
        Err(e) => {
            tracing::error!("Error occurred while hashing the password! Details: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

impl RefreshToken {
    pub async fn retrieve_self_from<'c, E>(&self, db: E) -> Result<UserInfo, Response>
    where
        E: PgExecutor<'c>,
    {
        match UserInfo::fetch_all_fields_by_id(db, self.id).await {
            Ok(user_info) => Ok(user_info),
            Err(e) => {
                if e.is_not_found() {
                    Err(StatusCode::UNAUTHORIZED.into_response())
                } else {
                    Err(Response::from(e))
                }
            }
        }
    }

    pub fn into_jwt(self, config: &JwtEncoderConfig) -> Result<String, AuthError> {
        let mut jwt = Jwt::new(&config.issue_as, &config.audience, self).not_valid_in(config.not_valid_in);

        if config.never_expires {
            jwt = jwt.never_expires();
        } else {
            jwt = jwt.expires_in(config.expires_in);
        }

        config.encoder.encode_randomly(&jwt)
    }
}

impl From<UserInfo> for RefreshToken {
    fn from(user: UserInfo) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            phone: user.phone,
        }
    }
}

impl AccessToken {
    #[allow(dead_code)]
    pub async fn retrieve_self_from<'c, E>(&self, db: E) -> Result<UserInfo, Response>
    where
        E: PgExecutor<'c>,
    {
        match UserInfo::fetch_all_fields_by_id(db, self.id).await {
            Ok(user_info) => Ok(user_info),
            Err(e) => {
                if e.is_not_found() {
                    Err(StatusCode::UNAUTHORIZED.into_response())
                } else {
                    Err(Response::from(e))
                }
            }
        }
    }

    pub fn into_jwt(self, config: &JwtEncoderConfig) -> Result<String, AuthError> {
        let mut jwt = Jwt::new(&config.issue_as, &config.audience, self).not_valid_in(config.not_valid_in);

        if config.never_expires {
            jwt = jwt.never_expires();
        } else {
            jwt = jwt.expires_in(config.expires_in);
        }

        config.encoder.encode_randomly(&jwt)
    }
}

impl From<Uuid> for AccessToken {
    fn from(id: Uuid) -> Self {
        Self { id }
    }
}

impl From<AccessToken> for Uuid {
    fn from(AccessToken { id }: AccessToken) -> Self {
        id
    }
}

impl AsRef<Uuid> for AccessToken {
    fn as_ref(&self) -> &Uuid {
        &self.id
    }
}
