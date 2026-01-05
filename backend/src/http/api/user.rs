mod danger_zone;
mod info;
mod login;
mod signup;
use crab_vault::auth::Jwt;
use crab_vault::auth::error::AuthError;
use http::header;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::app_config::AppConfig;
use crate::app_config::util::JwtEncoderConfig;
use crate::{
    entity::user::user_info::UserInfo, http::middleware::auth::AuthLayer, server::ServerState,
};
use axum::{
    Extension, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

static ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(argon2::Config::default);

pub(super) fn build_router(config: &AppConfig) -> Router<ServerState> {
    let router = Router::new();
    let auth_layer = AuthLayer::new(
        config.auth.decoder_config.decoder.clone(),
        |_, _, _, token: Jwt<UserIdent>| Box::pin(async move { Ok(token.load) }),
    );

    async fn redirect(ident: Extension<UserIdent>) -> impl IntoResponse {
        (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, format!("/user/{}", ident.id))],
        )
    }

    router
        .route("/user", routing::delete(danger_zone::delete_account))
        .route("/user", routing::put(danger_zone::change_auth_info))
        .route("/user/bio", routing::get(redirect))
        .route("/user/bio/{which}", routing::put(info::put))
        .layer(auth_layer)
        .route("/user/{id}", routing::get(info::get))
        .route("/user", routing::post(signup::signup))
        .route("/user/login", routing::post(login::login))
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserIdent {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl UserIdent {
    pub async fn retrieve_self_from_db<'c, E>(&self, db: E) -> Result<UserInfo, Response>
    where
        E: Executor<'c, Database = Postgres>,
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
        config.encoder.encode_randomly(
            &Jwt::new(&config.issue_as, &config.audience, self)
                .expires_in(config.expires_in)
                .not_valid_in(config.not_valid_in),
        )
    }
}

impl From<UserInfo> for UserIdent {
    fn from(user: UserInfo) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            phone: user.phone,
        }
    }
}

/// 检查密码是否正确
///
/// `val` 中是 使用 argon2 哈希过后的密码，`password` 是明文密码
#[tracing::instrument(name = "[user/check password]", skip_all)]
async fn check_password(val: &UserInfo, password: &str) -> Result<(), Response> {
    match argon2::verify_encoded(&val.password_hash, password.as_bytes()) {
        Ok(true) => {
            tracing::info!("Authorization of user (id: {}) successfully.", val.id);
            Ok(())
        }
        Ok(false) => {
            tracing::info!(
                "Authorization of user (id: {}) failed for incorrect password.",
                val.id
            );
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
