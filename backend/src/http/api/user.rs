mod danger_zone;
mod info;
mod login;
mod signup;

use crab_vault::auth::Jwt;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::app_config;
use crate::entity::user::user_info::UserInfo;
use crate::http::middleware::auth::AuthLayer;
use crate::server::ServerState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Router, routing};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

static ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(argon2::Config::default);

pub(super) fn build_router() -> Router<ServerState> {
    let router = Router::new();

    router
        .route("/", routing::delete(danger_zone::delete_account))
        .route("/", routing::put(danger_zone::change_auth_info))
        .layer(AuthLayer::new(
            &app_config::auth().decoder.decoder,
            |_, _, _, token: Jwt<UserIdent>| Box::pin(async move { Ok(token.load) }),
        ))
        .route("/{id}", routing::get(info::info))
        .route("/", routing::post(signup::signup))
        .route("/login", routing::post(login::login))
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserIdent {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl UserIdent {
    pub async fn retrieve_self_from_db(&self, db: &PgPool) -> Result<UserInfo, Response> {
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

    pub fn issue_as_jwt(self) -> Response {
        let config = &app_config::auth().encoder;

        (
            StatusCode::OK,
            config.encoder.encode_randomly(
                &Jwt::new(&config.issue_as, &config.audience, self)
                    .expires_in(config.expires_in)
                    .not_valid_in(config.not_valid_in),
            ),
        )
            .into_response()
    }
}

impl From<UserInfo> for UserIdent {
    fn from(user: UserInfo) -> Self {
        Self {
            email: user.email,
            phone: user.phone,
            id: user.id,
            name: user.name,
        }
    }
}

#[tracing::instrument(name = "[user/check password]", skip_all)]
async fn check_passwd(val: &UserInfo, passwd: &str) -> Result<(), Response> {
    match argon2::verify_encoded(&val.passwd_hash, passwd.as_bytes()) {
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
async fn generate_passwd_hash(passwd: &str) -> Result<String, Response> {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::now_v7().into_bytes());
    match argon2::hash_encoded(passwd.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG) {
        Ok(val) => Ok(val),
        Err(e) => {
            tracing::error!("Error occurred while hashing the password! Details: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
